#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/repo-slug.sh"

REPO=${1:-}
BRANCH=${2:-main}
REPO_SLUG=$(resolve_repo_slug "$REPO")

gh api "repos/$REPO_SLUG/branches/$BRANCH/protection" \
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

echo "branch protection enforced"
