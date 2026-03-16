#!/usr/bin/env bash
set -euo pipefail

GATE=${1:-all}

if [ "$GATE" = "all" ]; then
  exec bash scripts/run-all-gates.sh
fi

ARTIFACT_DIR="artifacts/black-vault/ci-single-gate"
mkdir -p "$ARTIFACT_DIR"

run_or_fail() {
  local name=$1
  local command=$2
  echo "running $name"
  eval "$command" >"$ARTIFACT_DIR/$name.log" 2>&1
}

case "$GATE" in
  G1) run_or_fail "G1" "cargo build --workspace && pnpm --filter @adaptive/desktop build" ;;
  G2) run_or_fail "G2" "cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings" ;;
  G3) run_or_fail "G3" "pnpm check:ts" ;;
  G4) run_or_fail "G4" "cargo test --workspace && uv run --project workers/quant pytest workers/quant/tests" ;;
  G6) run_or_fail "G6" "cargo llvm-cov --workspace --json --output-path \"$ARTIFACT_DIR/G6-rust-coverage.json\" && uv run --project workers/quant --with coverage coverage run --branch -m pytest workers/quant/tests && uv run --project workers/quant --with coverage coverage json -o \"$ARTIFACT_DIR/G6-python-coverage.json\"" ;;
  G8) run_or_fail "G8" "cargo audit && pnpm audit --audit-level high && trivy fs --severity HIGH,CRITICAL --exit-code 1 --skip-dirs target --skip-dirs apps/desktop/dist --skip-dirs node_modules --skip-dirs workers/quant/.venv ." ;;
  G9) run_or_fail "G9" "cargo clippy --workspace --all-targets --all-features -- -D warnings && semgrep --config auto ." ;;
  G10) run_or_fail "G10" "gitleaks detect --no-git --source . --redact" ;;
  *)
    echo "unsupported gate: $GATE"
    exit 1
    ;;
esac
