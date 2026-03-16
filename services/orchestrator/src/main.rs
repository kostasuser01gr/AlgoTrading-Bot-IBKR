use std::collections::BTreeMap;
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,orchestrator_service=debug").init();

    let audit_writer = TamperEvidentAuditWriter::new("runtime-orchestrator.audit.jsonl");
    let model_router = ModelRouter::default_router();
    let risk_engine = RiskEngine::new(RiskPolicyConfig::default());
    let (command_bus, mut receiver) = CommandBus::new(512);

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

    let state = AppState { model_router, risk_engine, audit_writer, command_bus };

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
        ]),
        checked_at: Utc::now(),
    })
}

async fn chat_request(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
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

    let risk = state.risk_engine.evaluate(
        &routing.fused,
        &default_portfolio(&request.mode),
        request.mode.clone(),
    );

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

    if let Err(error) = state
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
    {
        error!(error = %error, "failed to write audit event");
    }

    Ok(Json(response))
}

fn default_portfolio(mode: &OperationalMode) -> PortfolioSnapshot {
    let account_id = match mode {
        OperationalMode::Research => "research",
        OperationalMode::Backtest => "backtest",
        OperationalMode::Paper => "paper",
        OperationalMode::Live => "live",
    };
    PortfolioSnapshot {
        account_id: account_id.to_string(),
        cash: 250_000.0,
        equity: 250_000.0,
        realized_daily_pnl: 0.0,
        realized_weekly_pnl: 0.0,
        open_positions: 2,
        gross_exposure: 0.12,
        net_exposure: 0.08,
        correlated_exposure: 0.10,
    }
}
