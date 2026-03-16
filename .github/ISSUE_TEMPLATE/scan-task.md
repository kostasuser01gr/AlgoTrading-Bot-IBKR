---
name: Scan Task
about: Assign a file scan task
labels: type:scan-task
---

## Scan Task

**File**: `path/to/file`  
**Lines**: `NNN`  
**Risk Level**: `P0 | P1 | P2`

## Scan Blocks

- [ ] Block 1

## Instructions

1. Run `bash scripts/hardening/scan-file.sh <file>`
2. Review with the 7-lens protocol
3. Update `.black-vault/ScanLedger.json`
4. Log findings in `.black-vault/FindingsRegister.json`
5. Run `bash scripts/run-all-gates.sh`
