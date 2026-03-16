use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    ActionKind, Direction, DissentingView, EvidenceItem, FusedThesis, ModelOutput, ModelRole,
};
use crate::model_router::AnalysisRequest;

#[derive(Debug, Clone)]
pub struct FusionEngine {
    regime_weights: BTreeMap<String, BTreeMap<ModelRole, f32>>,
}

impl Default for FusionEngine {
    fn default() -> Self {
        let default_weights = BTreeMap::from([
            (ModelRole::Reasoning, 1.0),
            (ModelRole::MarketStructure, 1.2),
            (ModelRole::SentimentNews, 0.9),
            (ModelRole::QuantPredictive, 1.3),
            (ModelRole::Risk, 0.8),
        ]);
        let volatile_weights = BTreeMap::from([
            (ModelRole::Reasoning, 0.9),
            (ModelRole::MarketStructure, 1.1),
            (ModelRole::SentimentNews, 1.3),
            (ModelRole::QuantPredictive, 1.0),
            (ModelRole::Risk, 1.1),
        ]);
        Self {
            regime_weights: BTreeMap::from([
                ("base".to_string(), default_weights),
                ("volatile".to_string(), volatile_weights),
            ]),
        }
    }
}

impl FusionEngine {
    pub fn fuse(&self, request: &AnalysisRequest, outputs: Vec<ModelOutput>) -> FusedThesis {
        let weights =
            self.regime_weights.get(&request.regime).or_else(|| self.regime_weights.get("base"));

        let active_outputs: Vec<&ModelOutput> =
            outputs.iter().filter(|output| !output.abstain).collect();
        let total_weight = active_outputs
            .iter()
            .map(|output| self.weight_for(weights, &output.role))
            .sum::<f32>()
            .max(1.0);

        let directional_score = active_outputs
            .iter()
            .map(|output| self.weight_for(weights, &output.role) * output.direction.score())
            .sum::<f32>()
            / total_weight;

        let consensus_direction = if directional_score > 0.2 {
            Direction::Long
        } else if directional_score < -0.2 {
            Direction::Short
        } else {
            Direction::Neutral
        };

        let confidence = active_outputs
            .iter()
            .map(|output| self.weight_for(weights, &output.role) * output.confidence)
            .sum::<f32>()
            / total_weight;

        let disagreement_score = active_outputs
            .iter()
            .map(|output| {
                (output.direction.score() - directional_score).abs()
                    * self.weight_for(weights, &output.role)
            })
            .sum::<f32>()
            / total_weight;

        let overconfidence_flag = confidence > 0.78 && disagreement_score > 0.35;
        let abstain = active_outputs.is_empty()
            || confidence < 0.55
            || disagreement_score > 0.60
            || overconfidence_flag;
        let recommended_action = if abstain {
            ActionKind::Watch
        } else {
            match consensus_direction {
                Direction::Long => ActionKind::OpenLong,
                Direction::Short => ActionKind::OpenShort,
                Direction::Neutral => ActionKind::Hold,
            }
        };

        let recommended_size = if abstain {
            0.0
        } else {
            (active_outputs
                .iter()
                .map(|output| self.weight_for(weights, &output.role) * output.recommended_size)
                .sum::<f32>()
                / total_weight)
                * (1.0 - disagreement_score.clamp(0.0, 0.8))
        };

        let supporting_evidence = outputs
            .iter()
            .flat_map(|output| output.supporting_evidence.clone())
            .take(8)
            .collect::<Vec<EvidenceItem>>();
        let source_refs = outputs
            .iter()
            .flat_map(|output| output.source_refs.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let dissenting_views = outputs
            .iter()
            .filter(|output| !output.abstain && output.direction != consensus_direction)
            .map(|output| DissentingView {
                model_name: output.model_name.clone(),
                reason: output
                    .risk_notes
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "model diverged from consensus".to_string()),
                direction: output.direction.clone(),
                confidence: output.confidence,
            })
            .collect::<Vec<_>>();
        let why_not = outputs
            .iter()
            .filter(|output| output.abstain)
            .map(|output| format!("{} abstained due to low conviction", output.model_name))
            .chain(overconfidence_flag.then_some(
                "ensemble confidence is high while disagreement remains elevated".to_string(),
            ))
            .collect::<Vec<_>>();

        FusedThesis {
            id: Uuid::new_v4(),
            market: request.market.clone(),
            regime: request.regime.clone(),
            thesis: if abstain {
                format!(
                    "Abstain on {} until model disagreement narrows or fresher evidence arrives",
                    request.market
                )
            } else {
                format!(
                    "Consensus leans {:?} on {} over {} with regime-aware weighted evidence",
                    consensus_direction, request.market, request.time_horizon
                )
            },
            direction: consensus_direction,
            time_horizon: request.time_horizon.clone(),
            confidence,
            disagreement_score,
            overconfidence_flag,
            recommended_action,
            recommended_size,
            abstain,
            supporting_evidence,
            dissenting_views,
            why_not,
            model_outputs: outputs.clone(),
            source_refs,
            generated_at: Utc::now(),
        }
    }

    fn weight_for(&self, weights: Option<&BTreeMap<ModelRole, f32>>, role: &ModelRole) -> f32 {
        weights.and_then(|weights| weights.get(role)).copied().unwrap_or(1.0)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::{Direction, EvidenceItem, ModelOutput, ModelRole};
    use crate::model_router::AnalysisRequest;

    use super::FusionEngine;

    #[test]
    fn fusion_abstains_when_disagreement_is_high() {
        let engine = FusionEngine::default();
        let request = AnalysisRequest {
            market: "BTC-USD".to_string(),
            question: "Assess BTC".to_string(),
            regime: "volatile".to_string(),
            time_horizon: "1d".to_string(),
            source_ids: vec!["src-1".to_string()],
        };
        let outputs = vec![
            ModelOutput {
                id: Uuid::new_v4(),
                model_name: "quant".to_string(),
                role: ModelRole::QuantPredictive,
                thesis: "Bullish".to_string(),
                direction: Direction::Long,
                time_horizon: "1d".to_string(),
                confidence: 0.82,
                supporting_evidence: vec![EvidenceItem {
                    summary: "momentum".to_string(),
                    source_id: "src-1".to_string(),
                    weight: 0.8,
                }],
                invalidation_conditions: vec![],
                risk_notes: vec!["volatility elevated".to_string()],
                recommended_action: crate::domain::ActionKind::OpenLong,
                recommended_size: 0.02,
                abstain: false,
                latency_ms: 10,
                cost_usd: 0.01,
                source_refs: vec!["src-1".to_string()],
                generated_at: Utc::now(),
            },
            ModelOutput {
                id: Uuid::new_v4(),
                model_name: "sentiment".to_string(),
                role: ModelRole::SentimentNews,
                thesis: "Bearish".to_string(),
                direction: Direction::Short,
                time_horizon: "1d".to_string(),
                confidence: 0.81,
                supporting_evidence: vec![],
                invalidation_conditions: vec![],
                risk_notes: vec!["macro headlines negative".to_string()],
                recommended_action: crate::domain::ActionKind::OpenShort,
                recommended_size: 0.02,
                abstain: false,
                latency_ms: 12,
                cost_usd: 0.01,
                source_refs: vec!["src-1".to_string()],
                generated_at: Utc::now(),
            },
        ];

        let fused = engine.fuse(&request, outputs);
        assert!(fused.abstain);
        assert!(fused.disagreement_score > 0.6);
    }
}
