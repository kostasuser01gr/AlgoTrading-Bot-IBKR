#!/usr/bin/env bash
set -euo pipefail

resolve_repo_slug() {
  local repo=${1:-}

  if [ -z "$repo" ] || [ "$repo" = "." ]; then
    gh repo view --json owner,name -q '.owner.login + "/" + .name'
    return
  fi

  gh repo view "$repo" --json owner,name -q '.owner.login + "/" + .name'
}
