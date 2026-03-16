---
name: Finding Report
about: Report a security, reliability, or compliance finding
labels: type:finding,status:OPEN
---

## Finding Details

**ID**: `F###`  
**Severity**: `P0 | P1 | P2`  
**Category**: `Security | Reliability | Performance | Maintainability`  
**File**: `path/to/file`  
**Line**: `NNN`

## Description

Describe the issue clearly and concretely.

## Impact

Describe exploitability or operator impact.

## Evidence

Include the proof artifact, failing test, or log snippet.

## Reproduction

```bash
# exact steps
```

## Solution

Describe the minimal patch.

## Acceptance Criteria

- [ ] Fix implemented
- [ ] Regression test added
- [ ] Gate suite passes
- [ ] ScanLedger updated
- [ ] Status set to VERIFIED
