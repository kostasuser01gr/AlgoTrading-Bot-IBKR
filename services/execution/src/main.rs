use std::collections::BTreeMap;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{
    ApprovedAction, Direction, HealthSnapshot, LiveOrder, OperationalMode, PaperTrade,
};
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Default)]
struct AppState;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StageOrderRequest {
    approved_action: ApprovedAction,
    account: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum StageOrderResponse {
    Paper(PaperTrade),
    Live(LiveOrder),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,execution_service=debug").init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/stage", post(stage))
        .with_state(AppState)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7005").await?;
    info!("execution-service listening on 127.0.0.1:7005");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "execution-service".to_string(),
        mode: OperationalMode::Paper,
        healthy: true,
        details: BTreeMap::from([("kill_switch".to_string(), "armed".to_string())]),
        checked_at: Utc::now(),
    })
}

async fn stage(
    State(_state): State<AppState>,
    Json(request): Json<StageOrderRequest>,
) -> Result<Json<StageOrderResponse>, String> {
    if !request.approved_action.approved_by.iter().any(|approval| approval == "risk_engine") {
        return Err("risk_engine approval missing".to_string());
    }

    let side = match request.approved_action.action {
        operator_core::ActionKind::OpenLong => Direction::Long,
        operator_core::ActionKind::OpenShort => Direction::Short,
        _ => Direction::Neutral,
    };

    let response = match request.approved_action.mode {
        OperationalMode::Paper => StageOrderResponse::Paper(PaperTrade {
            id: Uuid::new_v4(),
            thesis_id: request.approved_action.thesis_id,
            market: request.approved_action.market,
            side,
            size: request.approved_action.size,
            entry_price: 0.0,
            status: "staged".to_string(),
            created_at: Utc::now(),
        }),
        OperationalMode::Live => StageOrderResponse::Live(LiveOrder {
            id: Uuid::new_v4(),
            thesis_id: request.approved_action.thesis_id,
            broker_account: request.account,
            market: request.approved_action.market,
            side,
            size: request.approved_action.size,
            limit_price: None,
            status: "awaiting_submission".to_string(),
            submitted_at: Utc::now(),
        }),
        _ => return Err("execution staging only supports paper or live mode".to_string()),
    };
    Ok(Json(response))
}
