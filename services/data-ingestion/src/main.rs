use std::collections::BTreeMap;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{
    ApprovedSourceCrawler, ConnectorCapability, ConnectorCapabilityRegistry, ConnectorClass,
    HealthSnapshot, OperationalMode, StaticSourceCrawler, connectors::CrawlRequest,
};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    crawler: StaticSourceCrawler,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,data_ingestion_service=debug").init();

    let mut registry = ConnectorCapabilityRegistry::default();
    registry.register(ConnectorCapability {
        connector_id: "approved-http".to_string(),
        class: ConnectorClass::ReadOnlyResearch,
        scopes: vec!["news.read".to_string(), "filings.read".to_string()],
        rate_limit_per_minute: 30,
        dry_run_supported: true,
        session_isolation: "read-only-profile".to_string(),
    });

    let state =
        AppState { crawler: StaticSourceCrawler::new(["example.com", "www.sec.gov"], registry) };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/crawl", post(crawl))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7004").await?;
    info!("data-ingestion-service listening on 127.0.0.1:7004");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "data-ingestion-service".to_string(),
        mode: OperationalMode::Research,
        healthy: true,
        details: BTreeMap::from([("crawler".to_string(), "approved-hosts-loaded".to_string())]),
        checked_at: Utc::now(),
    })
}

async fn crawl(
    State(state): State<AppState>,
    Json(request): Json<CrawlRequest>,
) -> Result<Json<operator_core::connectors::CrawledSource>, String> {
    let response = state.crawler.crawl(request).await.map_err(|error| error.to_string())?;
    Ok(Json(response))
}
