#!/usr/bin/env bash
set -euo pipefail

python3 <<'PY'
import json
from datetime import datetime, timedelta, timezone
from pathlib import Path

path = Path(".black-vault/FindingsRegister.json")
findings = json.loads(path.read_text()) if path.exists() else []
threshold = datetime.now(timezone.utc) - timedelta(days=3)
stuck = []

for finding in findings:
    if finding.get("status") != "OPEN":
        continue
    created_at = finding.get("created_at")
    if not created_at:
        continue
    created = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
    if created < threshold:
        stuck.append(finding["id"])

print(json.dumps({"stuck_findings": stuck}))
PY
