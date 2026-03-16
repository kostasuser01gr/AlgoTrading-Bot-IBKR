# Stack Decision

## Primary Stack

1. `Tauri + React + TypeScript` for the native desktop console.
2. `Rust` for always-on orchestration, connectors, policy, risk, and execution control.
3. `Python` for quant research, feature engineering, model evaluation, and backtesting.

## Why This Split

- Rust is used where uptime, concurrency, memory safety, and explicit failure handling matter most.
- Python is kept in the research lane where the ecosystem for quant workflows is strongest.
- TypeScript is used for the operator console, schemas, and desktop-side command pipeline.

## Alternative

- `.NET/C#` is defensible if FIX-heavy execution, Windows-first deployment, or existing broker SDK dependencies dominate the roadmap.
- For this repo, Rust remains the better core runtime because it enforces process isolation and resource discipline more cleanly in a desktop-local control plane.

