#!/usr/bin/env bash
set -euo pipefail

python3 <<'PY'
import json
from pathlib import Path

path = Path(".black-vault/FindingsRegister.json")
findings = json.loads(path.read_text()) if path.exists() else []

for finding in findings:
    severity = finding.get("severity", "P2")
    finding["priority_label"] = f"priority:{severity}"

path.write_text(json.dumps(findings, indent=2) + "\n")
PY
