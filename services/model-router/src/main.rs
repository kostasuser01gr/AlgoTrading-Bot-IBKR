use std::collections::BTreeMap;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{AnalysisRequest, HealthSnapshot, ModelRouter, OperationalMode};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    model_router: ModelRouter,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,model_router_service=debug").init();

    let state = AppState { model_router: ModelRouter::default_router() };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/analyze", post(analyze))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7002").await?;
    info!("model-router-service listening on 127.0.0.1:7002");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "model-router-service".to_string(),
        mode: OperationalMode::Research,
        healthy: true,
        details: BTreeMap::from([("router".to_string(), "ready".to_string())]),
        checked_at: Utc::now(),
    })
}

async fn analyze(
    State(state): State<AppState>,
    Json(request): Json<AnalysisRequest>,
) -> Result<Json<operator_core::model_router::RoutingResult>, String> {
    let result = state.model_router.analyze(request).await.map_err(|error| error.to_string())?;
    Ok(Json(result))
}
