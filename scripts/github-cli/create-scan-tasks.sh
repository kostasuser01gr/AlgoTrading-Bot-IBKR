#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/repo-slug.sh"

REPO=${1:-}
REPO_SLUG=$(resolve_repo_slug "$REPO")

python3 <<'PY' | while IFS=$'\t' read -r file total risk blocks; do
import csv
import json
from pathlib import Path

inventory = Path(".black-vault/Inventory.tsv")
ledger = Path(".black-vault/ScanLedger.json")
scanned = set()

if ledger.exists():
    for item in json.loads(ledger.read_text()):
        scanned.add(item.get("file"))

with inventory.open() as handle:
    reader = csv.DictReader(handle, delimiter="\t")
    for row in reader:
        if row["classification"] == "generated" or row["exclusion_reason"]:
            continue
        if row["filepath"] in scanned:
            continue
        lines = int(row["total_lines"] or 0)
        block_size = 50 if row["risk_level"] == "P0" else 250
        blocks = (lines + block_size - 1) // block_size if lines else 0
        print(f"{row['filepath']}\t{lines}\t{row['risk_level']}\t{blocks}")
PY
  title="SCAN: ${file} (${blocks} blocks)"
  body=$(
    cat <<EOF
## Scan Task

**File**: \`${file}\`
**Lines**: ${total}
**Risk Level**: ${risk}
**Blocks**: ${blocks}

1. Run \`bash scripts/hardening/scan-file.sh ${file}\`
2. Update \`.black-vault/ScanLedger.json\`
3. Log findings in \`.black-vault/FindingsRegister.json\`
4. Run \`bash scripts/run-all-gates.sh\`
EOF
  )
  gh issue create --repo "$REPO_SLUG" --title "$title" --body "$body" --label "type:scan-task" >/dev/null 2>&1 || true
done

echo "scan tasks created"
