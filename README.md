# Adaptive Multi-Model Trading Operator

[![CI](https://github.com/kostasuser01gr/AlgoTrading-Bot-IBKR/actions/workflows/ci-build.yml/badge.svg)](https://github.com/kostasuser01gr/AlgoTrading-Bot-IBKR/actions/workflows/ci-build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue)](https://www.typescriptlang.org)
[![Python](https://img.shields.io/badge/Python-3.11+-blue)](https://python.org)

Production-grade starter monorepo for a native desktop market operator built with Tauri, Rust, Python, and TypeScript.

## Workspace Layout

- `apps/desktop`: native operator console built with React and Tauri.
- `crates/operator-core`: shared Rust domain, orchestration, fusion, risk, scheduler, connector, and audit primitives.
- `services/*`: local control-plane services for orchestration, routing, risk, ingestion, execution, alerts, and backtesting.
- `workers/quant`: Python research, feature, and backtest package.
- `packages/*`: shared TypeScript schemas, UI primitives, and SDK clients.
- `docs`: architecture, security, and operations guidance.
- `infra`: observability bootstrap configuration.
- `scripts`: repo utilities and verification helpers.

## Design Principles

- Production-first
- Auditable by default
- Execution only after policy and risk approval
- Mode separation for research, backtest, paper, and live
- No hidden autonomy
- Reversible adaptation

## Quickstart

```bash
pnpm install
uv sync --project workers/quant
cargo test --workspace
uv run --project workers/quant pytest workers/quant/tests
```

## Development Entry Points

```bash
pnpm dev:desktop
cargo run -p orchestrator-service
cargo run -p model-router-service
cargo run -p risk-engine-service
```

