#!/usr/bin/env bash
set -euo pipefail

STACK=${1:-auto}

if [ "$STACK" = "auto" ]; then
  STACK="mixed"
fi

export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"

install_with_fallback() {
  local tool=$1
  local install_script=$2
  if command -v brew >/dev/null 2>&1; then
    brew install "$tool"
    return 0
  fi
  curl -sSfL "$install_script" | sh -s -- -b "$HOME/.local/bin"
}

if ! command -v node >/dev/null 2>&1; then
  echo "node is required"
  exit 1
fi

if ! command -v pnpm >/dev/null 2>&1; then
  npm install -g pnpm
fi

if ! command -v uv >/dev/null 2>&1; then
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="$HOME/.local/bin:$PATH"
fi

if ! command -v cargo-audit >/dev/null 2>&1; then
  cargo install cargo-audit
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo install cargo-llvm-cov
fi

if ! command -v cargo-mutants >/dev/null 2>&1; then
  cargo install cargo-mutants
fi

if ! command -v semgrep >/dev/null 2>&1; then
  python3 -m pip install --user semgrep
fi

if ! command -v gitleaks >/dev/null 2>&1; then
  install_with_fallback gitleaks https://raw.githubusercontent.com/gitleaks/gitleaks/master/install.sh
fi

if ! command -v trivy >/dev/null 2>&1; then
  install_with_fallback trivy https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh
fi

rustup component add llvm-tools-preview
pnpm install --frozen-lockfile
uv sync --project workers/quant --extra dev
