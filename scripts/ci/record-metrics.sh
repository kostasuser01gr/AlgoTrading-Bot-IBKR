#!/usr/bin/env bash
set -euo pipefail

python3 <<'PY'
import json
from pathlib import Path

runs_root = Path("artifacts/black-vault/runs")
runs = sorted(p for p in runs_root.glob("GR-*") if p.is_dir())
metrics_path = Path(".black-vault/MetricsLedger.json")

if metrics_path.exists():
    metrics = json.loads(metrics_path.read_text())
else:
    metrics = []

if not runs:
    raise SystemExit("no gate runs found")

latest = runs[-1]
metadata = json.loads((latest / "metadata.json").read_text())

entry = {
    "timestamp": metadata["timestamp"],
    "gate_run_id": metadata["gate_run_id"],
    "passed": metadata["passed"],
    "failed": metadata["failed"],
    "artifact_dir": metadata["artifact_dir"],
}

metrics.append(entry)
metrics_path.write_text(json.dumps(metrics, indent=2) + "\n")
PY
