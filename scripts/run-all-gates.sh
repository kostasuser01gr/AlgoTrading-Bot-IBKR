#!/usr/bin/env bash
set -euo pipefail

GATE_RUN_ID="GR-$(date -u +%Y-%m-%d-%H%M%S)"
ARTIFACT_DIR="artifacts/black-vault/runs/$GATE_RUN_ID"
mkdir -p "$ARTIFACT_DIR"
mkdir -p .black-vault

GATES_PASSED=0
GATES_FAILED=0
GATES_WARNED=0

run_gate() {
  local gate_id=$1
  local command=$2
  local description=$3

  echo "[$(date -u +%H:%M:%S)] Running $gate_id: $description..."

  if eval "$command" >"$ARTIFACT_DIR/$gate_id.log" 2>&1; then
    echo "PASS $gate_id"
    GATES_PASSED=$((GATES_PASSED + 1))
    return 0
  fi

  echo "FAIL $gate_id (see $ARTIFACT_DIR/$gate_id.log)"
  GATES_FAILED=$((GATES_FAILED + 1))
  return 1
}

run_advisory_gate() {
  local gate_id=$1
  local command=$2
  local description=$3

  echo "[$(date -u +%H:%M:%S)] Running $gate_id: $description..."

  if eval "$command" >"$ARTIFACT_DIR/$gate_id.log" 2>&1; then
    echo "PASS $gate_id"
    GATES_PASSED=$((GATES_PASSED + 1))
    return 0
  fi

  echo "WARN $gate_id (see $ARTIFACT_DIR/$gate_id.log)"
  GATES_WARNED=$((GATES_WARNED + 1))
  return 0
}

gate_integration() {
  local pid
  local status=0

  cargo run -p orchestrator-service >"$ARTIFACT_DIR/orchestrator-service.log" 2>&1 &
  pid=$!

  for _ in $(seq 1 30); do
    if curl -sf http://127.0.0.1:7001/health >"$ARTIFACT_DIR/G5-health.json"; then
      break
    fi
    sleep 1
  done

  curl -sf -X POST http://127.0.0.1:7001/v1/chat/request \
    -H "content-type: application/json" \
    -d @tests/contracts/chat-request.example.json >"$ARTIFACT_DIR/G5-chat-response.json" || status=$?

  kill "$pid" >/dev/null 2>&1 || true
  wait "$pid" >/dev/null 2>&1 || true
  return "$status"
}

gate_runtime() {
  gate_integration
  cp "$ARTIFACT_DIR/G5-health.json" "$ARTIFACT_DIR/G13-health.json"
  cp "$ARTIFACT_DIR/G5-chat-response.json" "$ARTIFACT_DIR/G13-chat-response.json"
}

gate_perf() {
  local pid
  local status=0

  cargo run -p orchestrator-service >"$ARTIFACT_DIR/orchestrator-perf.log" 2>&1 &
  pid=$!

  for _ in $(seq 1 30); do
    if curl -sf http://127.0.0.1:7001/health >/dev/null; then
      break
    fi
    sleep 1
  done

  curl -sf -X POST http://127.0.0.1:7001/v1/chat/request \
    -H "content-type: application/json" \
    -d @tests/contracts/chat-request.example.json >/dev/null

  python3 - <<'PY' >"$ARTIFACT_DIR/G14-latency.txt"
import json
import time
import urllib.request

payload = open("tests/contracts/chat-request.example.json", "rb").read()
latencies = []

for _ in range(5):
    req = urllib.request.Request(
        "http://127.0.0.1:7001/v1/chat/request",
        data=payload,
        headers={"content-type": "application/json"},
    )
    start = time.perf_counter()
    with urllib.request.urlopen(req) as response:
        response.read()
    latencies.append((time.perf_counter() - start) * 1000)

print("avg_ms=%.3f" % (sum(latencies) / len(latencies)))
print("max_ms=%.3f" % max(latencies))
for value in latencies:
    print(f"{value:.3f}")
PY

  python3 - "$ARTIFACT_DIR/G14-latency.txt" <<'PY'
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    lines = [line.strip() for line in handle if line.strip()]

avg_ms = float(lines[0].split("=", 1)[1])
max_ms = float(lines[1].split("=", 1)[1])

if avg_ms > 50.0 or max_ms > 100.0:
    raise SystemExit(1)
PY
  status=$?

  kill "$pid" >/dev/null 2>&1 || true
  wait "$pid" >/dev/null 2>&1 || true
  return "$status"
}

gate_iac() {
  local output

  output="$(trivy config --severity HIGH,CRITICAL --exit-code 1 infra 2>&1)" || {
    printf '%s\n' "$output"
    return 1
  }

  printf '%s\n' "$output"

  if printf '%s\n' "$output" | grep -q 'Detected config files[[:space:]]*num=0'; then
    return 1
  fi

  if printf '%s\n' "$output" | grep -q 'Supported files for scanner(s) not found'; then
    return 1
  fi
}

BUILD_CMD="cargo build --workspace && pnpm --filter @adaptive/desktop build"
LINT_CMD="cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings"
TYPECHECK_CMD="pnpm check:ts"
TEST_CMD="cargo test --workspace && uv run --project workers/quant pytest workers/quant/tests"
INTEGRATION_CMD="gate_integration"
COVERAGE_CMD='cargo llvm-cov --workspace --json --output-path "$ARTIFACT_DIR/G6-rust-coverage.json" && uv run --project workers/quant --with coverage coverage run --branch -m pytest workers/quant/tests && uv run --project workers/quant --with coverage coverage json -o "$ARTIFACT_DIR/G6-python-coverage.json"'
MUTATION_CMD="cargo mutants -p operator-core --timeout 60 --no-shuffle"
AUDIT_CMD="cargo audit && pnpm audit --audit-level high && trivy fs --severity HIGH,CRITICAL --exit-code 1 --skip-dirs target --skip-dirs apps/desktop/dist --skip-dirs node_modules --skip-dirs workers/quant/.venv ."
SAST_CMD="cargo clippy --workspace --all-targets --all-features -- -D warnings && semgrep --config auto ."
SECRETS_CMD="gitleaks detect --no-git --source . --redact"
CONTAINER_CMD="gate_iac"
RUNTIME_CMD="gate_runtime"
PERF_CMD="gate_perf"

run_gate "G1" "$BUILD_CMD" "Build/Compile"
run_gate "G2" "$LINT_CMD" "Format/Lint"
run_gate "G3" "$TYPECHECK_CMD" "Typecheck"
run_gate "G4" "$TEST_CMD" "Unit Tests"
run_gate "G5" "$INTEGRATION_CMD" "Integration Tests"
run_advisory_gate "G6" "$COVERAGE_CMD" "Coverage Report"
run_advisory_gate "G7" "$MUTATION_CMD" "Mutation Testing"
run_gate "G8" "$AUDIT_CMD" "Dependency Audit"
run_gate "G9" "$SAST_CMD" "SAST"
run_gate "G10" "$SECRETS_CMD" "Secrets Scan"
run_gate "G11" "bash scripts/ci-audit.sh" "Config/CI Audit"
run_advisory_gate "G12" "$CONTAINER_CMD" "Container/IaC Scan"
run_gate "G13" "$RUNTIME_CMD" "Runtime Sanity"
run_gate "G14" "$PERF_CMD" "Performance Regression"

cat >"$ARTIFACT_DIR/metadata.json" <<EOF
{
  "gate_run_id": "$GATE_RUN_ID",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "passed": $GATES_PASSED,
  "failed": $GATES_FAILED,
  "warned": $GATES_WARNED,
  "artifact_dir": "$ARTIFACT_DIR"
}
EOF

python3 - "$ARTIFACT_DIR/metadata.json" <<'PY'
import json
import sys
from pathlib import Path

metadata = json.loads(Path(sys.argv[1]).read_text())
ledger_path = Path(".black-vault/ArtifactsLedger.json")

if ledger_path.exists():
    ledger = json.loads(ledger_path.read_text())
else:
    ledger = []

ledger.append(
    {
        "gate_run_id": metadata["gate_run_id"],
        "timestamp": metadata["timestamp"],
        "all_gates_status": "PASS" if metadata["failed"] == 0 else "FAIL",
        "artifact_dir": metadata["artifact_dir"],
    }
)

ledger_path.write_text(json.dumps(ledger, indent=2) + "\n")
PY

echo ""
echo "========================================="
echo "Gate Run: $GATE_RUN_ID"
echo "Passed: $GATES_PASSED"
echo "Failed: $GATES_FAILED"
echo "Warned: $GATES_WARNED"
echo "Artifacts: $ARTIFACT_DIR"
echo "========================================="

if [ "$GATES_FAILED" -gt 0 ]; then
  echo "FAIL-STOP: $GATES_FAILED gates failed. Aborting work until fixed."
  exit 1
fi

echo "All gates PASS"
