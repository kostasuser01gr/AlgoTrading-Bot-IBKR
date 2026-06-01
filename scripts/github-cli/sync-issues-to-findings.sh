#!/usr/bin/env bash
set -euo pipefail

REPO_ARG=${1:-}
if [[ -n "${GITHUB_REPOSITORY:-}" && ( -z "$REPO_ARG" || "$REPO_ARG" == "." ) ]]; then
  REPO_SLUG="$GITHUB_REPOSITORY"
else
  REPO=${REPO_ARG:-.}
  OWNER=$(gh repo view "$REPO" --json owner -q '.owner.login')
  NAME=$(gh repo view "$REPO" --json name -q '.name')
  REPO_SLUG="$OWNER/$NAME"
fi

tmp_file=$(mktemp)
gh issue list --repo "$REPO_SLUG" --label "type:finding" --state all --json number,title,body,labels,state,createdAt >"$tmp_file"

python3 - "$tmp_file" <<'PY'
import json
import re
import sys
from pathlib import Path

issues = json.loads(Path(sys.argv[1]).read_text())
path = Path(".black-vault/FindingsRegister.json")
findings = json.loads(path.read_text()) if path.exists() else []
by_id = {item["id"]: item for item in findings if "id" in item}

for issue in issues:
    title = issue["title"]
    match = re.match(r"(F\d+):", title)
    finding_id = match.group(1) if match else f"F{issue['number']:04d}"
    labels = {label["name"] for label in issue.get("labels", [])}
    severity = "P2"
    for candidate in ("security:P0", "security:P1", "security:P2"):
      if candidate in labels:
        severity = candidate.split(":")[1]
        break
    status = "VERIFIED" if issue["state"] == "CLOSED" else "OPEN"
    current = by_id.get(finding_id, {"id": finding_id})
    current.update(
        {
            "id": finding_id,
            "severity": severity,
            "title": title,
            "github_issue": issue["number"],
            "status": status,
            "created_at": issue.get("createdAt", ""),
        }
    )
    by_id[finding_id] = current

path.write_text(json.dumps(list(by_id.values()), indent=2) + "\n")
PY

rm -f "$tmp_file"
echo "synced findings"
