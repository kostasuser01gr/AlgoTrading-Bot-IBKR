# Security Policy

## Supported Scope

This repository is maintained as a production-oriented trading operator workspace. Security fixes should prioritize:

- secret exposure and credential handling
- dependency vulnerabilities rated HIGH or CRITICAL
- unsafe workflow automation or CI bootstrap behavior
- runtime paths that bypass risk approval or audit logging

## Reporting

Do not open a public issue for a live secret, token, or actively exploitable flaw.

Report sensitive findings privately to the repository owner with:

- affected file or component
- reproduction steps
- impact assessment
- proposed mitigation if known

## Local Security Checks

Preferred local checks:

```bash
bash scripts/run-all-gates.sh
gitleaks detect --no-git --source . --redact
trivy fs --severity HIGH,CRITICAL --exit-code 1 --skip-dirs target --skip-dirs apps/desktop/dist --skip-dirs node_modules --skip-dirs workers/quant/.venv .
semgrep --config auto .
```

## Secret Handling

- Use environment variables for credentials and live-trading toggles.
- Keep examples sanitized.
- Never store production secrets in tracked files, workflow logs, or issue bodies.
