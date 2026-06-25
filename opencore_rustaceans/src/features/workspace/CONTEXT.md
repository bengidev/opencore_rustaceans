# Workspace context

Project shell for an open repo: OpenRouter model catalog, API-key settings,
session persistence, close-project flow, and chat panel composition.

## Glossary

| Term | Meaning |
|------|---------|
| **Workspace** | Screen bound to one open project path |
| **Chat** | [`crate::features::chat`] — thread, composer, streaming |
| **Reducer** | `WorkspaceState::update` — delegates chat to `ChatState` |
| **Outcome** | `WorkspaceOutcome` — routing hint for the app shell |
| **Overlay** | OpenRouter API key settings, model picker, or close-project confirmation |
| **Model catalog** | Fetched from OpenRouter when an API key is stored |
| **Session** | [`WorkspaceSession`] Strategy — JSON snapshot of open project + chat |
| **Credential store** | [`WorkspaceCredentialStore`] Strategy — Keychain + file composite (`PersistedWorkspaceCredentialStore`) |
| **AI provider** | [`AiProvider`] Strategy — OpenRouter SSE streaming (default model `openai/gpt-4o-mini`) |

## Actions

| Action | Behavior |
|--------|----------|
| Send message | Requires OpenRouter API key and loaded model; streams assistant reply |
| Missing key hint | Opens OpenRouter API key overlay |
| Model chip | Shows `Not Available` without key; fetches and lists OpenRouter models with key |
| Configure OpenRouter | Opens API key settings overlay |
| Close project | Confirms → clears session → returns to welcome |
| Window close / Cmd+Q | App shell persists full workspace session |

Provider ID for API keys: `"openrouter"`.

## Public surface (Facade)

Callers outside the module import only from `features::workspace`:

- `view` — workspace shell composing chat
- `WorkspaceState`, `WorkspaceMessage`, `WorkspaceOutcome`
- `ChatThread`, `ChatMessage`, `ChatRole` (re-exported from `features::chat`)
- `WorkspaceSession`, `FileWorkspaceSession`, `InMemoryWorkspaceSession`, `WorkspaceSessionData`
- `WorkspaceCredentialStore`, `PersistedWorkspaceCredentialStore`, `InMemoryWorkspaceCredentialStore`
- `AiProvider`, `OpenRouterProvider`, `CannedAiProvider`, `ChatRequest`, `ChatStreamEvent`
- `fetch_openrouter_models`, `ModelOption`
- `DEFAULT_MODEL`, `OPENROUTER_PROVIDER_ID`

Routing and persistence side effects live in `app/` — not in this module.

## Branding

OpenRouter provider logo and copy; shared monochrome design tokens throughout.
