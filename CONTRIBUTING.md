# Contributing

## Local Setup

```bash
pnpm install --frozen-lockfile
uv sync --project workers/quant --extra dev
cargo build --workspace
```

## Validation

Run the fast local matrix before pushing:

```bash
pnpm check:ts
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
uv run --project workers/quant pytest workers/quant/tests
```

Run the full hardening gates when changing shared logic, CI, or release tooling:

```bash
bash scripts/run-all-gates.sh
```

## GitHub Workflows

- `Gates` validates build, lint, type checks, tests, audit, SAST, and secrets on PRs and automation branch pushes.
- `Hardening Nightly` and `Compliance Audit` are for broader scheduled verification and reporting.

## Change Expectations

- Keep fixes small and reversible.
- Do not commit secrets, tokens, or local machine paths.
- Update `README.md`, `SECURITY.md`, or workflow docs when the operator workflow changes.
