#!/usr/bin/env bash
set -euo pipefail

STACK=${1:-auto}

if [ "$STACK" = "auto" ]; then
  STACK="mixed"
fi

export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"
mkdir -p "$HOME/.local/bin"

detect_os() {
  case "$(uname -s)" in
    Darwin) echo "darwin" ;;
    Linux) echo "linux" ;;
    *)
      echo "unsupported operating system: $(uname -s)" >&2
      return 1
      ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    arm64|aarch64) echo "arm64" ;;
    x86_64|amd64) echo "x64" ;;
    *)
      echo "unsupported architecture: $(uname -m)" >&2
      return 1
      ;;
  esac
}

install_from_tarball_release() {
  local repo=$1
  local asset=$2
  local binary=$3
  local scratch

  scratch=$(mktemp -d)
  gh release download --repo "$repo" --pattern "$asset" --dir "$scratch"
  tar -xzf "$scratch/$asset" -C "$scratch"
  install -m 0755 "$scratch/$binary" "$HOME/.local/bin/$binary"
  rm -rf "$scratch"
}

install_gitleaks() {
  local os arch version asset

  if command -v brew >/dev/null 2>&1; then
    brew install gitleaks
    return 0
  fi

  os=$(detect_os)
  arch=$(detect_arch)
  version=$(gh release view --repo gitleaks/gitleaks --json tagName -q '.tagName')
  asset="gitleaks_${version#v}_${os}_${arch}.tar.gz"
  install_from_tarball_release gitleaks/gitleaks "$asset" gitleaks
}

install_trivy() {
  local os arch version asset

  if command -v brew >/dev/null 2>&1; then
    brew install trivy
    return 0
  fi

  os=$(detect_os)
  arch=$(detect_arch)
  version=$(gh release view --repo aquasecurity/trivy --json tagName -q '.tagName')

  case "${os}:${arch}" in
    linux:x64) asset="trivy_${version#v}_Linux-64bit.tar.gz" ;;
    linux:arm64) asset="trivy_${version#v}_Linux-ARM64.tar.gz" ;;
    darwin:x64) asset="trivy_${version#v}_macOS-64bit.tar.gz" ;;
    darwin:arm64) asset="trivy_${version#v}_macOS-ARM64.tar.gz" ;;
    *)
      echo "unsupported platform for trivy: ${os}:${arch}" >&2
      return 1
      ;;
  esac

  install_from_tarball_release aquasecurity/trivy "$asset" trivy
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
  install_gitleaks
fi

if ! command -v trivy >/dev/null 2>&1; then
  install_trivy
fi

rustup component add llvm-tools-preview
pnpm install --frozen-lockfile
uv sync --project workers/quant --extra dev
