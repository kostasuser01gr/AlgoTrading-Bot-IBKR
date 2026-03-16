#!/usr/bin/env bash
set -euo pipefail

if [ ! -d .github ]; then
  echo "no-ci-files"
  exit 0
fi

find .github -type f | sort

if grep -R -nE '@(main|master|latest)\b' .github; then
  echo "unpinned CI action reference detected"
  exit 1
fi

workflow_count=$(find .github/workflows -type f \( -name '*.yml' -o -name '*.yaml' \) 2>/dev/null | wc -l | tr -d ' ')
if [ "${workflow_count:-0}" -gt 0 ]; then
  missing_permissions=$(find .github/workflows -type f \( -name '*.yml' -o -name '*.yaml' \) -exec grep -L '^permissions:' {} + || true)
  if [ -n "$missing_permissions" ]; then
    echo "workflow missing explicit permissions block"
    printf '%s\n' "$missing_permissions"
    exit 1
  fi
fi

echo "ci-files-reviewed"
