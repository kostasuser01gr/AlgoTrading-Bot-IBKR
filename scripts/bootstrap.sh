#!/usr/bin/env bash
set -euo pipefail

pnpm install
uv sync --project workers/quant --extra dev
cargo test --workspace
uv run --project workers/quant pytest workers/quant/tests

