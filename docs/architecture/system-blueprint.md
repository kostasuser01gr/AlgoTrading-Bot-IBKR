# System Blueprint

## Recommended Architecture

1. Native desktop shell in Tauri for the operator console.
2. Local Rust control plane for orchestration, policy enforcement, risk gating, audit, and connector supervision.
3. Python quant worker package for research, feature engineering, and backtesting.
4. Optional remote infrastructure for model training, heavy simulations, object storage, and fleet observability.

## Service Boundaries

- `apps/desktop`: operator UX only. No direct broker credentials.
- `services/orchestrator`: workflow assembly, task fan-out, final response synthesis.
- `services/model-router`: model invocation and fusion.
- `services/risk-engine`: authoritative pre-trade gate.
- `services/data-ingestion`: approved sources only, provenance required.
- `services/execution`: order staging and mode-aware submission path.
- `services/backtest`: deterministic simulations.
- `services/alerts`: alert fan-out and escalation.

## Trust Boundaries

- External content is untrusted until normalized and tagged.
- Models are advisory until risk and policy approve downstream actions.
- Execution requires a signed approved action.
- Audit records are append-only and tamper-evident.

