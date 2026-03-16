# Module Breakdown

## Core Runtime Modules

- `operator-core/domain`: shared contracts for theses, risk decisions, audit events, memory items, and health snapshots.
- `operator-core/model_router`: model adapter contract plus heuristic starter adapters.
- `operator-core/fusion`: regime-aware weighted voting and disagreement scoring.
- `operator-core/risk`: approval pipeline and rejection logic.
- `operator-core/connectors`: capability registry and approved source crawler abstraction.
- `operator-core/scheduler`: background jobs with explicit intervals.
- `operator-core/audit`: tamper-evident JSONL audit chain.
- `operator-core/command_bus`: command ingress abstraction for operator intents.

## Desktop Modules

- `src/lib/commandBus.ts`: chat request pipeline and command lifecycle events.
- `src/components/MissionControlShell.tsx`: premium operator console layout.
- `packages/sdk`: HTTP SDK to the local orchestrator.
- `packages/shared-types`: shared Zod schemas and TypeScript contracts.

