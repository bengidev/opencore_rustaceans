# Workspace context

AI chat screen for an open project: streaming OpenRouter completions,
API-key settings (iOS parity), session persistence, and close-project flow.

## Glossary

| Term | Meaning |
|------|---------|
| **Workspace** | Chat screen bound to one open project path |
| **Thread** | [`ChatThread`] — append-only user/assistant messages |
| **Reducer** | `WorkspaceState::update` — pure message → state + outcome |
| **Outcome** | `WorkspaceOutcome` — routing hint for the app shell |
| **Overlay** | API key settings or close-project confirmation modal |
| **Session** | [`WorkspaceSession`] Strategy — JSON snapshot of open project + chat |
| **Credential store** | [`WorkspaceCredentialStore`] Strategy — keyring or in-memory API keys |
| **AI provider** | [`AiProvider`] Strategy — OpenRouter SSE streaming (default model `openai/gpt-4o-mini`) |

## Actions

| Action | Behavior |
|--------|----------|
| Send message | Requires API key; streams assistant reply via OpenRouter |
| Missing key hint | Opens API key settings overlay |
| Configure actions | Opens API key settings (phase-1 stub for custom actions) |
| Close project | Confirms → clears session → returns to welcome |
| Window close / Cmd+Q | App shell persists full workspace session |

Provider ID for API keys: `"openrouter"`.

## Public surface (Facade)

Callers outside the module import only from `features::workspace`:

- `view` — embeddable chat view
- `WorkspaceState`, `WorkspaceMessage`, `WorkspaceOutcome`
- `ChatThread`, `ChatMessage`, `ChatRole`
- `WorkspaceSession`, `FileWorkspaceSession`, `InMemoryWorkspaceSession`, `WorkspaceSessionData`
- `WorkspaceCredentialStore`, `KeychainWorkspaceCredentialStore`, `InMemoryWorkspaceCredentialStore`
- `AiProvider`, `OpenRouterProvider`, `CannedAiProvider`, `ChatRequest`, `ChatStreamEvent`
- `DEFAULT_MODEL`, `OPENROUTER_PROVIDER_ID`

Routing and persistence side effects live in `app/` — not in this module.

## Branding

Uses **OpenCore** naming and shared monochrome design tokens.
