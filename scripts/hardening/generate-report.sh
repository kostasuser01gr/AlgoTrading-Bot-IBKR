#!/usr/bin/env bash
set -euo pipefail

exec bash scripts/github-cli/generate-compliance-report.sh "$@"
