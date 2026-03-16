#!/usr/bin/env bash
set -euo pipefail

python3 <<'PY'
import json
from pathlib import Path
from datetime import datetime, timezone

root = Path(".")
report_path = root / "BLACK_VAULT_HARDENING_REPORT.md"
compliance_path = root / ".black-vault" / "ComplianceLedger.json"

findings = json.loads((root / ".black-vault" / "FindingsRegister.json").read_text())
scan_ledger = json.loads((root / ".black-vault" / "ScanLedger.json").read_text())
artifacts = json.loads((root / ".black-vault" / "ArtifactsLedger.json").read_text())

complete_scans = all(item.get("status") == "COMPLETE" for item in scan_ledger) and bool(scan_ledger)
open_findings = sum(1 for item in findings if item.get("status") == "OPEN")
all_gates_green = bool(artifacts) and artifacts[-1].get("all_gates_status") == "PASS"

checks = [
    ("Inventory Complete", (root / ".black-vault" / "Inventory.tsv").exists()),
    ("ScanLedger 100% Coverage", complete_scans),
    ("FindingsRegister EMPTY", open_findings == 0),
    ("All Gates PASS", all_gates_green),
    ("Secrets Scan Zero", all_gates_green),
    ("Supply Chain Locked", (root / "Cargo.lock").exists() and (root / "pnpm-lock.yaml").exists()),
]

lines = [
    "# Black Vault Hardening Report",
    "",
    f"Generated: {datetime.now(timezone.utc).isoformat()}",
    "",
    "| Gate | Status |",
    "|------|--------|",
]
for name, status in checks:
    lines.append(f"| {name} | {'PASS' if status else 'FAIL'} |")
lines.extend(
    [
        "",
        "## Findings Summary",
        f"- Total: {len(findings)}",
        f"- Open: {open_findings}",
        f"- Verified: {sum(1 for item in findings if item.get('status') == 'VERIFIED')}",
    ]
)

report_path.write_text("\n".join(lines) + "\n")

compliance = json.loads(compliance_path.read_text()) if compliance_path.exists() else []
compliance.append(
    {
        "audit_id": f"AUD-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "checks": [{"gate": name, "status": "PASS" if status else "FAIL"} for name, status in checks],
        "compliance_score": round(sum(1 for _, status in checks if status) / len(checks) * 100, 2),
    }
)
compliance_path.write_text(json.dumps(compliance, indent=2) + "\n")
PY
