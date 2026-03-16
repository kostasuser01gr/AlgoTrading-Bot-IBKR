#!/usr/bin/env bash
set -euo pipefail

echo "Start local services in separate terminals:"
echo "  cargo run -p orchestrator-service"
echo "  cargo run -p model-router-service"
echo "  cargo run -p risk-engine-service"
echo "  cargo run -p data-ingestion-service"
echo "  cargo run -p execution-service"
echo "  cargo run -p backtest-service"
echo "  cargo run -p alerts-service"
echo "  pnpm --filter @adaptive/desktop dev"

