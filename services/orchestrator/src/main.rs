use std::collections::BTreeMap;
use std::env;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{
    AnalysisRequest, AuditEvent, BackgroundScheduler, ChatRequest, ChatResponse, CommandBus,
    HealthSnapshot, ModelRouter, OperationalMode, PortfolioSnapshot, RiskEngine, RiskPolicyConfig,
    TamperEvidentAuditWriter,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    model_router: ModelRouter,
    risk_engine: RiskEngine,
    audit_writer: TamperEvidentAuditWriter,
    command_bus: CommandBus,
    portfolio_state: PortfolioStateResolver,
    live_mode_enabled: bool,
}

#[derive(Clone)]
struct PortfolioStateResolver {
    research: PortfolioSnapshot,
    backtest: PortfolioSnapshot,
    paper: PortfolioSnapshot,
    live: Option<PortfolioSnapshot>,
}

impl PortfolioStateResolver {
    fn from_env() -> Result<Self, String> {
        let live = match env::var("OPERATOR_LIVE_PORTFOLIO_SNAPSHOT_JSON") {
            Ok(raw) => Some(serde_json::from_str(&raw).map_err(|error| {
                format!("invalid OPERATOR_LIVE_PORTFOLIO_SNAPSHOT_JSON: {error}")
            })?),
            Err(env::VarError::NotPresent) => None,
            Err(error) => {
                return Err(format!(
                    "failed to read OPERATOR_LIVE_PORTFOLIO_SNAPSHOT_JSON: {error}"
                ));
            }
        };

        Ok(Self {
            research: bootstrap_portfolio("research", 0.12, 0.08, 0.10, 2),
            backtest: bootstrap_portfolio("backtest", 0.08, 0.04, 0.05, 1),
            paper: bootstrap_portfolio("paper", 0.10, 0.06, 0.08, 2),
            live,
        })
    }

    fn resolve(&self, mode: &OperationalMode) -> Result<PortfolioSnapshot, String> {
        match mode {
            OperationalMode::Research => Ok(self.research.clone()),
            OperationalMode::Backtest => Ok(self.backtest.clone()),
            OperationalMode::Paper => Ok(self.paper.clone()),
            OperationalMode::Live => self.live.clone().ok_or_else(|| {
                "trusted live portfolio state is unavailable; set OPERATOR_LIVE_PORTFOLIO_SNAPSHOT_JSON"
                    .to_string()
            }),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,orchestrator_service=debug").init();

    let audit_writer = TamperEvidentAuditWriter::new("runtime-orchestrator.audit.jsonl");
    let model_router = ModelRouter::default_router();
    let risk_engine = RiskEngine::new(RiskPolicyConfig::default());
    let (command_bus, mut receiver) = CommandBus::new(512);
    let portfolio_state = PortfolioStateResolver::from_env().map_err(std::io::Error::other)?;
    let live_mode_enabled =
        matches!(env::var("OPERATOR_ENABLE_LIVE_TRADING").as_deref(), Ok("true"));

    tokio::spawn(async move {
        while let Some(intent) = receiver.recv().await {
            info!(intent_id = %intent.id, actor = %intent.actor, market = %intent.market, "operator intent received");
        }
    });

    let scheduler = BackgroundScheduler::new();
    scheduler
        .spawn_job("source-freshness", Duration::from_secs(60), || async { Ok::<(), String>(()) })
        .await;
    scheduler
        .spawn_job("model-score-refresh", Duration::from_secs(300), || async {
            Ok::<(), String>(())
        })
        .await;

    let state = AppState {
        model_router,
        risk_engine,
        audit_writer,
        command_bus,
        portfolio_state,
        live_mode_enabled,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat/request", post(chat_request))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7001").await?;
    info!("orchestrator-service listening on 127.0.0.1:7001");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "orchestrator-service".to_string(),
        mode: OperationalMode::Research,
        healthy: true,
        details: BTreeMap::from([
            ("scheduler".to_string(), "running".to_string()),
            ("audit".to_string(), "enabled".to_string()),
            ("live_mode".to_string(), "explicit-enable-required".to_string()),
        ]),
        checked_at: Utc::now(),
    })
}

async fn chat_request(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    if matches!(request.mode, OperationalMode::Live) && !state.live_mode_enabled {
        return Err((
            StatusCode::FORBIDDEN,
            "live mode requires OPERATOR_ENABLE_LIVE_TRADING=true and verified model adapters"
                .to_string(),
        ));
    }

    let correlation_id = state
        .command_bus
        .publish(
            request.actor.clone(),
            request.mode.clone(),
            request.market.clone(),
            request.message.clone(),
            vec!["research.read".to_string()],
        )
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let portfolio = state
        .portfolio_state
        .resolve(&request.mode)
        .map_err(|error| (StatusCode::FAILED_DEPENDENCY, error))?;

    let routing = state
        .model_router
        .analyze(AnalysisRequest {
            market: request.market.clone(),
            question: request.message.clone(),
            regime: if request.message.to_lowercase().contains("volatility") {
                "volatile".to_string()
            } else {
                "base".to_string()
            },
            time_horizon: "1d".to_string(),
            source_ids: vec!["market-feed".to_string(), "news-wire".to_string()],
        })
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    let risk = state.risk_engine.evaluate(&routing.fused, &portfolio, request.mode.clone());

    let response = ChatResponse {
        narrative: format!(
            "Market operator processed '{}' for {}. Consensus direction is {:?} with confidence {:.2}.",
            request.message, request.market, routing.fused.direction, routing.fused.confidence
        ),
        decision_summary: if risk.approved {
            format!("Risk approved {} at {:.2}% sizing.", request.market, risk.capped_size * 100.0)
        } else {
            format!("Execution blocked: {}", risk.reasons.join("; "))
        },
        citations: routing.fused.source_refs.clone(),
        machine_payload: json!({
            "correlation_id": correlation_id,
            "watchlist": request.watchlist,
            "mode": request.mode,
            "disagreement": routing.fused.disagreement_score,
        }),
        thesis: routing.fused.clone(),
        risk: risk.clone(),
    };

    state
        .audit_writer
        .write(AuditEvent {
            id: Uuid::new_v4(),
            actor: request.actor,
            action: "chat_request".to_string(),
            mode: request.mode,
            status: if risk.approved { "approved".to_string() } else { "blocked".to_string() },
            correlation_id,
            timestamp: Utc::now(),
            details: json!({
                "market": request.market,
                "decision_summary": response.decision_summary,
                "thesis_id": response.thesis.id,
                "risk_decision_id": response.risk.id
            }),
        })
        .await
        .map_err(|error| {
            error!(error = %error, "failed to write audit event");
            (StatusCode::INTERNAL_SERVER_ERROR, format!("audit persistence failure: {error}"))
        })?;

    Ok(Json(response))
}

fn bootstrap_portfolio(
    account_id: &str,
    gross_exposure: f64,
    net_exposure: f64,
    correlated_exposure: f64,
    open_positions: usize,
) -> PortfolioSnapshot {
    PortfolioSnapshot {
        account_id: account_id.to_string(),
        cash: 250_000.0,
        equity: 250_000.0,
        realized_daily_pnl: 0.0,
        realized_weekly_pnl: 0.0,
        open_positions,
        gross_exposure,
        net_exposure,
        correlated_exposure,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use axum::{Json, extract::State, http::StatusCode};
    use uuid::Uuid;

    use super::{AppState, PortfolioStateResolver, bootstrap_portfolio, chat_request};
    use operator_core::{
        ChatRequest, CommandBus, ModelRouter, OperationalMode, RiskEngine, RiskPolicyConfig,
        TamperEvidentAuditWriter,
    };

    fn build_state(
        audit_path: &std::path::Path,
        live_mode_enabled: bool,
        live: Option<operator_core::PortfolioSnapshot>,
    ) -> AppState {
        let (command_bus, mut receiver) = CommandBus::new(8);
        tokio::spawn(async move { while receiver.recv().await.is_some() {} });
        AppState {
            model_router: ModelRouter::default_router(),
            risk_engine: RiskEngine::new(RiskPolicyConfig::default()),
            audit_writer: TamperEvidentAuditWriter::new(audit_path),
            command_bus,
            portfolio_state: PortfolioStateResolver {
                research: bootstrap_portfolio("research", 0.12, 0.08, 0.10, 2),
                backtest: bootstrap_portfolio("backtest", 0.08, 0.04, 0.05, 1),
                paper: bootstrap_portfolio("paper", 0.10, 0.06, 0.08, 2),
                live,
            },
            live_mode_enabled,
        }
    }

    fn request(mode: OperationalMode) -> ChatRequest {
        ChatRequest {
            actor: "desktop-operator".to_string(),
            mode,
            market: "BTC-USD".to_string(),
            message: "Generate a 1-day thesis with dissent and risk constraints.".to_string(),
            watchlist: vec!["BTC-USD".to_string()],
        }
    }

    #[tokio::test]
    async fn chat_request_rejects_live_mode_without_explicit_enablement() {
        let audit_path = std::env::temp_dir().join(format!("audit-{}.jsonl", Uuid::new_v4()));
        let state =
            build_state(&audit_path, false, Some(bootstrap_portfolio("live", 0.04, 0.02, 0.01, 1)));

        let error = chat_request(State(state), Json(request(OperationalMode::Live)))
            .await
            .expect_err("live mode should be blocked");

        assert_eq!(error.0, StatusCode::FORBIDDEN);
        let _ = fs::remove_file(audit_path);
    }

    #[tokio::test]
    async fn chat_request_requires_trusted_live_portfolio_state() {
        let audit_path = std::env::temp_dir().join(format!("audit-{}.jsonl", Uuid::new_v4()));
        let state = build_state(&audit_path, true, None);

        let error = chat_request(State(state), Json(request(OperationalMode::Live)))
            .await
            .expect_err("live mode should fail without trusted state");

        assert_eq!(error.0, StatusCode::FAILED_DEPENDENCY);
        let _ = fs::remove_file(audit_path);
    }

    #[tokio::test]
    async fn chat_request_fails_closed_on_audit_error() {
        let audit_dir = std::env::temp_dir().join(format!("audit-dir-{}", Uuid::new_v4()));
        fs::create_dir(&audit_dir).expect("create temp audit dir");
        let state = build_state(&audit_dir, false, None);

        let error = chat_request(State(state), Json(request(OperationalMode::Research)))
            .await
            .expect_err("audit failure should block the response");

        assert_eq!(error.0, StatusCode::INTERNAL_SERVER_ERROR);
        let _ = fs::remove_dir_all(audit_dir);
    }

    #[tokio::test]
    async fn chat_request_succeeds_for_research_with_bootstrap_state() {
        let audit_path = std::env::temp_dir().join(format!("audit-{}.jsonl", Uuid::new_v4()));
        let state = build_state(&audit_path, false, None);

        let response = chat_request(State(state), Json(request(OperationalMode::Research)))
            .await
            .expect("research mode should succeed")
            .0;

        assert!(response.risk.approved);
        assert_ne!(
            response.machine_payload["correlation_id"].as_str(),
            Some("00000000-0000-0000-0000-000000000000")
        );
        let _ = fs::remove_file(audit_path);
    }
}
