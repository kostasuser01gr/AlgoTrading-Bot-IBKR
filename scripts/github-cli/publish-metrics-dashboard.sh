#!/usr/bin/env bash
set -euo pipefail

python3 <<'PY'
import json
from datetime import datetime, timezone
from pathlib import Path

metrics_path = Path(".black-vault/MetricsLedger.json")
metrics = json.loads(metrics_path.read_text()) if metrics_path.exists() else []
report = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "runs": len(metrics),
    "successful_runs": sum(1 for item in metrics if item.get("failed", 0) == 0),
}
print(json.dumps(report, indent=2))
PY
