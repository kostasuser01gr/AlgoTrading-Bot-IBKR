#!/usr/bin/env bash
set -euo pipefail

FILE=${1:?usage: scan-file.sh <file>}

python3 - "$FILE" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

file_path = Path(sys.argv[1])
if not file_path.exists():
    raise SystemExit(f"missing file: {file_path}")

total_lines = sum(1 for _ in file_path.open())
block_size = 250
ranges = []
for start in range(1, total_lines + 1, block_size):
    end = min(start + block_size - 1, total_lines)
    ranges.append(
        {
            "start": start,
            "end": end,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "notes": "pending-review",
            "findings": [],
        }
    )

ledger_path = Path(".black-vault/ScanLedger.json")
ledger = json.loads(ledger_path.read_text()) if ledger_path.exists() else []
ledger = [item for item in ledger if item.get("file") != str(file_path)]
ledger.append(
    {
        "file": str(file_path),
        "total_lines": total_lines,
        "ranges_scanned": ranges,
        "status": "IN_PROGRESS",
        "re_scans": [],
    }
)
ledger_path.write_text(json.dumps(ledger, indent=2) + "\n")
PY
