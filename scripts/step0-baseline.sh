#!/usr/bin/env bash
set -euo pipefail

echo "=== STEP 0: BASELINE GATE COMMANDS ==="
echo "Stack: mixed-rust-nodejs-python"
echo ""
echo "G1 Build: cargo build --workspace && pnpm --filter @adaptive/desktop build"
echo "G2 Lint: cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings"
echo "G3 Typecheck: pnpm check:ts"
echo "G4 Tests: cargo test --workspace && uv run --project workers/quant pytest workers/quant/tests"
echo "G5 Integration: orchestrator /health + /v1/chat/request smoke"
echo "G6 Coverage: cargo llvm-cov ... && coverage.py"
echo "G7 Mutation: cargo mutants -p operator-core --timeout 60 --no-shuffle"
echo "G8 Audit: cargo audit && pnpm audit --audit-level high && trivy fs ..."
echo "G9 SAST: cargo clippy ... && semgrep --config auto ."
echo "G10 Secrets: gitleaks detect --no-git --source . --redact"
echo "G11 CI: bash scripts/ci-audit.sh"
echo "G12 IaC: trivy config --severity HIGH,CRITICAL --exit-code 1 infra"
echo "G13 Runtime: orchestrator smoke"
echo "G14 Perf: 5 local /v1/chat/request measurements"
echo ""
echo "=== RUN: Full gate suite ==="
bash scripts/run-all-gates.sh
