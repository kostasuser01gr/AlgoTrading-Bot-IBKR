use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationalMode {
    Research,
    Backtest,
    Paper,
    Live,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Low,
    Medium,
    High,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestionMethod {
    Polling,
    Streaming,
    ScheduledCrawl,
    EventTrigger,
    BrowserAutomation,
    McpConnector,
    LocalImport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Long,
    Short,
    Neutral,
}

impl Direction {
    pub fn score(&self) -> f32 {
        match self {
            Self::Long => 1.0,
            Self::Short => -1.0,
            Self::Neutral => 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    Reasoning,
    MarketStructure,
    SentimentNews,
    QuantPredictive,
    Risk,
    Fusion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Hold,
    Watch,
    OpenLong,
    OpenShort,
    Reduce,
    Close,
    Hedge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Approved,
    Rejected,
    RequiresHumanApproval,
    Staged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorClass {
    ReadOnlyResearch,
    ExternalAction,
    TradingExecution,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceItem {
    pub summary: String,
    pub source_id: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceRecord {
    pub id: String,
    pub connector_id: String,
    pub source_uri: String,
    pub source_name: String,
    pub timestamp: DateTime<Utc>,
    pub trust_level: TrustLevel,
    pub ingestion_method: IngestionMethod,
    pub content_hash: String,
    pub freshness_score: f32,
    pub entity_tags: Vec<String>,
    pub market_tags: Vec<String>,
    pub strategy_relevance_tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelOutput {
    pub id: Uuid,
    pub model_name: String,
    pub role: ModelRole,
    pub thesis: String,
    pub direction: Direction,
    pub time_horizon: String,
    pub confidence: f32,
    pub supporting_evidence: Vec<EvidenceItem>,
    pub invalidation_conditions: Vec<String>,
    pub risk_notes: Vec<String>,
    pub recommended_action: ActionKind,
    pub recommended_size: f32,
    pub abstain: bool,
    pub latency_ms: u64,
    pub cost_usd: f32,
    pub source_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DissentingView {
    pub model_name: String,
    pub reason: String,
    pub direction: Direction,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FusedThesis {
    pub id: Uuid,
    pub market: String,
    pub regime: String,
    pub thesis: String,
    pub direction: Direction,
    pub time_horizon: String,
    pub confidence: f32,
    pub disagreement_score: f32,
    pub overconfidence_flag: bool,
    pub recommended_action: ActionKind,
    pub recommended_size: f32,
    pub abstain: bool,
    pub supporting_evidence: Vec<EvidenceItem>,
    pub dissenting_views: Vec<DissentingView>,
    pub why_not: Vec<String>,
    pub model_outputs: Vec<ModelOutput>,
    pub source_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSnapshot {
    pub account_id: String,
    pub cash: f64,
    pub equity: f64,
    pub realized_daily_pnl: f64,
    pub realized_weekly_pnl: f64,
    pub open_positions: usize,
    pub gross_exposure: f64,
    pub net_exposure: f64,
    pub correlated_exposure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RiskPolicyConfig {
    pub max_position_size: f32,
    pub confidence_floor: f32,
    pub max_open_positions: usize,
    pub max_daily_loss: f64,
    pub max_weekly_drawdown: f64,
    pub correlated_exposure_cap: f64,
    pub live_requires_human_approval: bool,
    pub blackout_markets: Vec<String>,
}

impl Default for RiskPolicyConfig {
    fn default() -> Self {
        Self {
            max_position_size: 0.02,
            confidence_floor: 0.58,
            max_open_positions: 8,
            max_daily_loss: 5_000.0,
            max_weekly_drawdown: 15_000.0,
            correlated_exposure_cap: 0.35,
            live_requires_human_approval: true,
            blackout_markets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RiskDecision {
    pub id: Uuid,
    pub thesis_id: Uuid,
    pub approved: bool,
    pub status: DecisionStatus,
    pub reasons: Vec<String>,
    pub capped_size: f32,
    pub required_approvals: Vec<String>,
    pub kill_switch_armed: bool,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRun {
    pub id: Uuid,
    pub strategy_name: String,
    pub mode: OperationalMode,
    pub market: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub thesis_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BacktestResult {
    pub id: Uuid,
    pub strategy_name: String,
    pub market: String,
    pub timeframe: String,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub sharpe: f64,
    pub win_rate: f64,
    pub trade_count: usize,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaperTrade {
    pub id: Uuid,
    pub thesis_id: Uuid,
    pub market: String,
    pub side: Direction,
    pub size: f32,
    pub entry_price: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiveOrder {
    pub id: Uuid,
    pub thesis_id: Uuid,
    pub broker_account: String,
    pub market: String,
    pub side: Direction,
    pub size: f32,
    pub limit_price: Option<f64>,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub id: Uuid,
    pub market: String,
    pub side: Direction,
    pub size: f32,
    pub average_price: f64,
    pub unrealized_pnl: f64,
    pub opened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlertRecord {
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub correlation_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: Uuid,
    pub actor: String,
    pub action: String,
    pub mode: OperationalMode,
    pub status: String,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorCapability {
    pub connector_id: String,
    pub class: ConnectorClass,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub dry_run_supported: bool,
    pub session_isolation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemoryItem {
    pub id: Uuid,
    pub namespace: String,
    pub key: String,
    pub value: serde_json::Value,
    pub confidence: f32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelScorecard {
    pub model_name: String,
    pub role: ModelRole,
    pub regime: String,
    pub hit_rate: f32,
    pub calibration_error: f32,
    pub average_latency_ms: u64,
    pub average_cost_usd: f32,
    pub sample_size: usize,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegimeSnapshot {
    pub market: String,
    pub regime: String,
    pub volatility_bucket: String,
    pub trend_bucket: String,
    pub liquidity_bucket: String,
    pub confidence: f32,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIntent {
    pub id: Uuid,
    pub actor: String,
    pub mode: OperationalMode,
    pub content: String,
    pub market: String,
    pub requested_capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub actor: String,
    pub mode: OperationalMode,
    pub market: String,
    pub message: String,
    pub watchlist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub narrative: String,
    pub decision_summary: String,
    pub thesis: FusedThesis,
    pub risk: RiskDecision,
    pub citations: Vec<String>,
    pub machine_payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovedAction {
    pub thesis_id: Uuid,
    pub action: ActionKind,
    pub market: String,
    pub size: f32,
    pub mode: OperationalMode,
    pub risk_decision_id: Uuid,
    pub approved_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshot {
    pub service: String,
    pub mode: OperationalMode,
    pub healthy: bool,
    pub details: BTreeMap<String, String>,
    pub checked_at: DateTime<Utc>,
}
