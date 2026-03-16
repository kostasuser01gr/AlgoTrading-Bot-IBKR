#!/usr/bin/env bash
set -euo pipefail

python3 <<'PY'
import json
from pathlib import Path

findings_path = Path(".black-vault/FindingsRegister.json")
state_path = Path(".black-vault/StateSnapshot.json")

findings = json.loads(findings_path.read_text()) if findings_path.exists() else []
open_count = sum(1 for item in findings if item.get("status") == "OPEN")
verified_count = sum(1 for item in findings if item.get("status") == "VERIFIED")

state = json.loads(state_path.read_text()) if state_path.exists() else {}
state["findings_open"] = open_count
state["findings_verified"] = verified_count
state_path.write_text(json.dumps(state, indent=2) + "\n")

print(f"open={open_count} verified={verified_count}")
PY
