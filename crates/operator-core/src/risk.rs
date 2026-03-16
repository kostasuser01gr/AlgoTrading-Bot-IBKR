use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    DecisionStatus, FusedThesis, OperationalMode, PortfolioSnapshot, RiskDecision, RiskPolicyConfig,
};

#[derive(Debug, Clone)]
pub struct RiskEngine {
    policy: RiskPolicyConfig,
    kill_switch_armed: bool,
}

impl RiskEngine {
    pub fn new(policy: RiskPolicyConfig) -> Self {
        Self { policy, kill_switch_armed: true }
    }

    pub fn evaluate(
        &self,
        thesis: &FusedThesis,
        portfolio: &PortfolioSnapshot,
        mode: OperationalMode,
    ) -> RiskDecision {
        let mut reasons = Vec::new();
        let mut status = DecisionStatus::Approved;
        let mut capped_size = thesis.recommended_size.min(self.policy.max_position_size);
        let mut required_approvals = vec!["risk_engine".to_string()];

        if thesis.abstain {
            reasons.push("fusion layer abstained".to_string());
        }
        if thesis.confidence < self.policy.confidence_floor {
            reasons.push(format!(
                "confidence {:.2} is below floor {:.2}",
                thesis.confidence, self.policy.confidence_floor
            ));
        }
        if portfolio.open_positions >= self.policy.max_open_positions {
            reasons.push("open position limit reached".to_string());
        }
        if portfolio.realized_daily_pnl <= -self.policy.max_daily_loss {
            reasons.push("daily loss threshold breached".to_string());
        }
        if portfolio.realized_weekly_pnl <= -self.policy.max_weekly_drawdown {
            reasons.push("weekly drawdown threshold breached".to_string());
        }
        if portfolio.correlated_exposure >= self.policy.correlated_exposure_cap {
            reasons.push("correlated exposure cap breached".to_string());
        }
        if self.policy.blackout_markets.iter().any(|market| market == &thesis.market) {
            reasons.push("market is in blackout window".to_string());
        }

        if !reasons.is_empty() {
            status = DecisionStatus::Rejected;
            capped_size = 0.0;
        } else if matches!(mode, OperationalMode::Live) && self.policy.live_requires_human_approval
        {
            status = DecisionStatus::RequiresHumanApproval;
            required_approvals.push("human_operator".to_string());
        }

        RiskDecision {
            id: Uuid::new_v4(),
            thesis_id: thesis.id,
            approved: matches!(status, DecisionStatus::Approved),
            status,
            reasons,
            capped_size,
            required_approvals,
            kill_switch_armed: self.kill_switch_armed,
            evaluated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::{
        ActionKind, Direction, FusedThesis, OperationalMode, PortfolioSnapshot, RiskPolicyConfig,
    };

    use super::RiskEngine;

    fn sample_thesis() -> FusedThesis {
        FusedThesis {
            id: Uuid::new_v4(),
            market: "BTC-USD".to_string(),
            regime: "base".to_string(),
            thesis: "Long BTC".to_string(),
            direction: Direction::Long,
            time_horizon: "1d".to_string(),
            confidence: 0.74,
            disagreement_score: 0.22,
            overconfidence_flag: false,
            recommended_action: ActionKind::OpenLong,
            recommended_size: 0.03,
            abstain: false,
            supporting_evidence: vec![],
            dissenting_views: vec![],
            why_not: vec![],
            model_outputs: vec![],
            source_refs: vec![],
            generated_at: Utc::now(),
        }
    }

    fn sample_portfolio() -> PortfolioSnapshot {
        PortfolioSnapshot {
            account_id: "paper".to_string(),
            cash: 100_000.0,
            equity: 100_000.0,
            realized_daily_pnl: 0.0,
            realized_weekly_pnl: 0.0,
            open_positions: 1,
            gross_exposure: 0.1,
            net_exposure: 0.1,
            correlated_exposure: 0.1,
        }
    }

    #[test]
    fn risk_engine_requires_human_approval_in_live_mode() {
        let engine = RiskEngine::new(RiskPolicyConfig::default());
        let decision =
            engine.evaluate(&sample_thesis(), &sample_portfolio(), OperationalMode::Live);
        assert!(!decision.approved);
        assert!(matches!(decision.status, crate::domain::DecisionStatus::RequiresHumanApproval));
        assert!(decision.required_approvals.contains(&"human_operator".to_string()));
    }
}
