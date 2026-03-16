use std::collections::BTreeMap;

use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{AlertRecord, HealthSnapshot, OperationalMode};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,alerts_service=debug").init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/publish", post(publish))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7007").await?;
    info!("alerts-service listening on 127.0.0.1:7007");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "alerts-service".to_string(),
        mode: OperationalMode::Research,
        healthy: true,
        details: BTreeMap::from([("channels".to_string(), "desktop,email,webhook".to_string())]),
        checked_at: Utc::now(),
    })
}

async fn publish(Json(alert): Json<AlertRecord>) -> Json<AlertRecord> {
    Json(AlertRecord { created_at: Utc::now(), ..alert })
}
