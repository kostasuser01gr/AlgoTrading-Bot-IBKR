# Security Model

## Core Controls

1. Separate credentials for research, paper, and live modes.
2. Capability-scoped connectors with rate limits and session isolation.
3. Tamper-evident audit log for every operator action and workflow outcome.
4. Read-only mode for research connectors and browser sessions by default.
5. Kill switch and risk approval are mandatory before execution.

## Prompt and Tool Safety

- Treat every external source as untrusted input.
- Strip secrets and disallowed context before sending prompts to models.
- Never grant a model unrestricted tool access.
- Maintain explicit allowlists for tools, models, and destinations.

