# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Layout

**Multi-context** — this repo uses `CONTEXT-MAP.md` at the root to point at one `CONTEXT.md` per context. System-wide decisions live in `docs/adr/`; context-specific decisions live under each context's `docs/adr/`.

## Before exploring, read these

- **`CONTEXT-MAP.md`** at the repo root — identifies each context and where its `CONTEXT.md` lives. Read every `CONTEXT.md` relevant to the topic.
- **`docs/adr/`** — system-wide ADRs. Also check `src/<context>/docs/adr/` (or the path listed in the map) for context-scoped decisions.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The producer skill (`/real-engineer-grill-with-docs`) creates them lazily when terms or decisions actually get resolved.

## File structure

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← system-wide decisions
└── src/
    ├── <context-a>/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← context-specific decisions
    └── <context-b>/
        ├── CONTEXT.md
        └── docs/adr/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in the relevant `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/real-engineer-grill-with-docs`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders) — but worth reopening because…_
