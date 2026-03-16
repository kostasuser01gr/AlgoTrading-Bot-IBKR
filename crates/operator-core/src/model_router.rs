use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{ActionKind, Direction, EvidenceItem, FusedThesis, ModelOutput, ModelRole};
use crate::fusion::FusionEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisRequest {
    pub market: String,
    pub question: String,
    pub regime: String,
    pub time_horizon: String,
    pub source_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingResult {
    pub outputs: Vec<ModelOutput>,
    pub fused: FusedThesis,
}

#[derive(Debug, Error)]
pub enum ModelRouterError {
    #[error("model adapter failure: {0}")]
    Adapter(String),
}

#[async_trait]
pub trait ModelAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn role(&self) -> ModelRole;
    async fn infer(&self, request: &AnalysisRequest) -> Result<ModelOutput, ModelRouterError>;
}

#[derive(Debug, Clone)]
pub struct HeuristicModelAdapter {
    name: &'static str,
    role: ModelRole,
}

impl HeuristicModelAdapter {
    pub fn new(name: &'static str, role: ModelRole) -> Self {
        Self { name, role }
    }

    fn directional_view(&self, question: &str) -> Direction {
        let lowered = question.to_lowercase();
        if lowered.contains("breakout")
            || lowered.contains("bull")
            || lowered.contains("long")
            || lowered.contains("strength")
        {
            Direction::Long
        } else if lowered.contains("bear")
            || lowered.contains("short")
            || lowered.contains("sell")
            || lowered.contains("weakness")
        {
            Direction::Short
        } else {
            match self.role {
                ModelRole::QuantPredictive => Direction::Long,
                ModelRole::Risk => Direction::Neutral,
                _ => Direction::Neutral,
            }
        }
    }
}

#[async_trait]
impl ModelAdapter for HeuristicModelAdapter {
    fn name(&self) -> &'static str {
        self.name
    }

    fn role(&self) -> ModelRole {
        self.role.clone()
    }

    async fn infer(&self, request: &AnalysisRequest) -> Result<ModelOutput, ModelRouterError> {
        let direction = self.directional_view(&request.question);
        let confidence = match self.role {
            ModelRole::QuantPredictive => 0.74,
            ModelRole::MarketStructure => 0.70,
            ModelRole::SentimentNews => 0.66,
            ModelRole::Reasoning => 0.68,
            ModelRole::Risk => 0.63,
            ModelRole::Fusion => 0.60,
        };
        let recommended_action = match direction {
            Direction::Long => ActionKind::OpenLong,
            Direction::Short => ActionKind::OpenShort,
            Direction::Neutral => ActionKind::Watch,
        };
        let recommended_size = match self.role {
            ModelRole::Risk => 0.01,
            ModelRole::QuantPredictive => 0.02,
            _ => 0.015,
        };
        let abstain = matches!(self.role, ModelRole::Risk) && direction == Direction::Neutral;

        Ok(ModelOutput {
            id: Uuid::new_v4(),
            model_name: self.name.to_string(),
            role: self.role.clone(),
            thesis: format!(
                "{} sees {:?} conditions for {} over {}",
                self.name, direction, request.market, request.time_horizon
            ),
            direction,
            time_horizon: request.time_horizon.clone(),
            confidence,
            supporting_evidence: vec![EvidenceItem {
                summary: format!(
                    "{} analyzed {} approved sources",
                    self.name,
                    request.source_ids.len()
                ),
                source_id: request
                    .source_ids
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
                weight: confidence,
            }],
            invalidation_conditions: vec![
                "break of session support/resistance".to_string(),
                "regime flip detected by volatility monitor".to_string(),
            ],
            risk_notes: vec!["size should be scaled by regime volatility".to_string()],
            recommended_action,
            recommended_size,
            abstain,
            latency_ms: 12,
            cost_usd: if matches!(self.role, ModelRole::Reasoning) { 0.04 } else { 0.0 },
            source_refs: request.source_ids.clone(),
            generated_at: Utc::now(),
        })
    }
}

#[derive(Clone)]
pub struct ModelRouter {
    adapters: Vec<Arc<dyn ModelAdapter>>,
    fusion_engine: FusionEngine,
}

impl ModelRouter {
    pub fn new(adapters: Vec<Arc<dyn ModelAdapter>>, fusion_engine: FusionEngine) -> Self {
        Self { adapters, fusion_engine }
    }

    pub fn default_router() -> Self {
        Self::new(
            vec![
                Arc::new(HeuristicModelAdapter::new("reasoning-core", ModelRole::Reasoning)),
                Arc::new(HeuristicModelAdapter::new(
                    "market-structure-core",
                    ModelRole::MarketStructure,
                )),
                Arc::new(HeuristicModelAdapter::new("sentiment-core", ModelRole::SentimentNews)),
                Arc::new(HeuristicModelAdapter::new("quant-core", ModelRole::QuantPredictive)),
                Arc::new(HeuristicModelAdapter::new("risk-context", ModelRole::Risk)),
            ],
            FusionEngine::default(),
        )
    }

    pub async fn analyze(
        &self,
        request: AnalysisRequest,
    ) -> Result<RoutingResult, ModelRouterError> {
        let mut outputs = Vec::with_capacity(self.adapters.len());
        for adapter in &self.adapters {
            outputs.push(adapter.infer(&request).await?);
        }
        let fused = self.fusion_engine.fuse(&request, outputs.clone());
        Ok(RoutingResult { outputs, fused })
    }
}
