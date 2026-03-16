use std::collections::BTreeMap;

use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use operator_core::{BacktestResult, HealthSnapshot, OperationalMode};
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BacktestRequest {
    strategy_name: String,
    market: String,
    timeframe: String,
    returns: Vec<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info,backtest_service=debug").init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/run", post(run_backtest))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7006").await?;
    info!("backtest-service listening on 127.0.0.1:7006");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthSnapshot> {
    Json(HealthSnapshot {
        service: "backtest-service".to_string(),
        mode: OperationalMode::Backtest,
        healthy: true,
        details: BTreeMap::from([("simulator".to_string(), "ready".to_string())]),
        checked_at: Utc::now(),
    })
}

async fn run_backtest(Json(request): Json<BacktestRequest>) -> Json<BacktestResult> {
    let total_return = request.returns.iter().sum::<f64>();
    let trade_count = request.returns.len();
    let wins = request.returns.iter().filter(|value| **value > 0.0).count() as f64;
    let max_drawdown = request
        .returns
        .iter()
        .scan(0.0, |equity, point| {
            *equity += point;
            Some(*equity)
        })
        .fold((0.0_f64, 0.0_f64), |(peak, drawdown), equity| {
            let next_peak = peak.max(equity);
            let next_drawdown = drawdown.min(equity - next_peak);
            (next_peak, next_drawdown)
        })
        .1;
    let mean = if trade_count == 0 { 0.0 } else { total_return / trade_count as f64 };
    let variance = if trade_count <= 1 {
        0.0
    } else {
        request.returns.iter().map(|value| (value - mean).powi(2)).sum::<f64>()
            / (trade_count - 1) as f64
    };
    let sharpe = if variance <= f64::EPSILON { 0.0 } else { mean / variance.sqrt() };

    Json(BacktestResult {
        id: Uuid::new_v4(),
        strategy_name: request.strategy_name,
        market: request.market,
        timeframe: request.timeframe,
        total_return,
        max_drawdown,
        sharpe,
        win_rate: if trade_count == 0 { 0.0 } else { wins / trade_count as f64 },
        trade_count,
        generated_at: Utc::now(),
    })
}
