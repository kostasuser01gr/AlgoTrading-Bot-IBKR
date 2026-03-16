#!/usr/bin/env bash
set -euo pipefail

REPO=${1:-.}
OWNER=$(gh repo view "$REPO" --json owner -q '.owner.login')
NAME=$(gh repo view "$REPO" --json name -q '.name')
REPO_SLUG="$OWNER/$NAME"

gh project create --owner "$OWNER" --title "Hardening Sprint" >/dev/null 2>&1 || true

for milestone in "Phase 0: Baseline" "Phase 1: Core Scan" "Phase 2: Hardening" "Phase 3: Release"; do
  gh api "repos/$REPO_SLUG/milestones" -f title="$milestone" >/dev/null 2>&1 || true
done

gh api "repos/$REPO_SLUG/branches/main/protection" \
  --method PUT \
  -H "Accept: application/vnd.github+json" \
  --input - <<'JSON' >/dev/null
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["gates"]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true,
    "required_approving_review_count": 1
  },
  "restrictions": null
}
JSON

for label in "security:P0" "security:P1" "security:P2" "type:finding" "type:scan-task" "status:OPEN" "status:VERIFIED"; do
  gh label create "$label" --repo "$REPO_SLUG" --force >/dev/null 2>&1 || true
done

echo "configured $REPO_SLUG"
