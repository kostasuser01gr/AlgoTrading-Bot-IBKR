use std::collections::BTreeMap;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{
    FusedThesis, HealthSnapshot, OperationalMode, PortfolioSnapshot, RiskEngine, RiskPolicyConfig,
};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    engine: RiskEngine,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EvaluateRiskRequest {
    thesis: FusedThesis,
    portfolio: PortfolioSnapshot,
    mode: OperationalMode,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,risk_engine_service=debug").init();

    let state = AppState { engine: RiskEngine::new(RiskPolicyConfig::default()) };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/evaluate", post(evaluate))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7003").await?;
    info!("risk-engine-service listening on 127.0.0.1:7003");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "risk-engine-service".to_string(),
        mode: OperationalMode::Research,
        healthy: true,
        details: BTreeMap::from([("kill_switch".to_string(), "armed".to_string())]),
        checked_at: Utc::now(),
    })
}

async fn evaluate(
    State(state): State<AppState>,
    Json(request): Json<EvaluateRiskRequest>,
) -> Json<operator_core::RiskDecision> {
    Json(state.engine.evaluate(&request.thesis, &request.portfolio, request.mode))
}
