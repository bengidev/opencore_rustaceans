# Architecture

OpenCore Rustaceans is a single Cargo package with **internal module**
boundaries — modules stay inside the crate until a real external consumer
appears.

## Composition root

`src/main.rs` is the application composition root. It chooses persistence
implementations, runs onboarding when needed, and launches the **app shell**
(`src/app/`). No feature logic accumulates there.

## Internal modules

```text
src/
├── main.rs                                # composition root
├── lib.rs                                 # test / embedder surface
├── app/                                   # welcome ↔ workspace router
│   ├── mod.rs                             # Facade: run(), ShellApp
│   ├── app_messages.rs                    # ShellMessage
│   ├── app_state.rs                       # ActiveScreen + shell reducer
│   └── app_effects.rs                     # dialogs, AI stream, persist-on-close
├── features/
│   ├── mod.rs                             # feature registry
│   ├── onboarding/
│   │   └── …                              # first-run flow (see onboarding CONTEXT)
│   ├── welcome/
│   │   └── …                              # home screen (see welcome CONTEXT)
│   └── workspace/
│       ├── mod.rs                         # Facade
│       ├── workspace_model.rs             # ChatThread composite
│       ├── workspace_messages.rs          # Command messages
│       ├── workspace_outcome.rs           # routing outcomes
│       ├── workspace_state.rs             # State reducer
│       ├── workspace_view.rs              # chat UI
│       ├── workspace_overlay.rs           # API key + close-project overlays
│       ├── workspace_session.rs           # session Strategy trait
│       ├── workspace_file_session.rs      # JSON persistence
│       ├── workspace_memory_session.rs    # in-memory test double
│       ├── workspace_credential_store.rs  # credential Strategy trait
│       ├── workspace_keychain_store.rs    # macOS Keychain adapter
│       ├── workspace_memory_credential.rs # in-memory test double
│       ├── workspace_ai_provider.rs       # AiProvider Strategy trait
│       ├── workspace_openrouter_provider.rs
│       └── workspace_sse.rs               # SSE line parser
└── shared/
    └── design/                            # tokens + theme
```

## Design patterns (GoF)

| Pattern | Where | Role |
|---------|-------|------|
| **Facade** | `onboarding/mod.rs`, `welcome/mod.rs`, `workspace/mod.rs`, `app/mod.rs` | Hide prefixed siblings; expose `run`, `view`, state types |
| **Composite** | `welcome_model.rs`, `workspace_model.rs` | Screen / thread content trees |
| **Strategy** | `WelcomeHistory`, `WorkspaceSession`, `WorkspaceCredentialStore`, `AiProvider` | Swaps file vs in-memory vs keychain backends |
| **Adapter** | `workspace_openrouter_provider.rs`, `workspace_sse.rs` | OpenRouter SSE → domain stream events |
| **Command** | `WelcomeMessage`, `WorkspaceMessage`, `ShellMessage` | Encodes user intents decoupled from widgets |
| **State** | `*State::update`, `AppState::update` | Pure transitions + routing outcomes |
| **Factory Method** | persistence constructors, `OpenCoreTheme::from_mode` | Pick concrete backends at composition time |
| **Template Method** | iced `Program` impls | Shared draw lifecycle for canvas widgets |

## TDD workflow

Tests are colocated in each feature module (`onboarding_*.rs`, `welcome_*.rs`,
`workspace_*.rs`, `app_*.rs`). Typical cycle:

1. **Red** — add a failing `#[test]` beside the reducer or strategy.
2. **Green** — implement the smallest change in the sibling module.
3. **Refactor** — keep the facade thin; run targeted `cargo test` filters.

Run the suite:

```bash
cargo test onboarding
cargo test welcome
cargo test workspace
cargo test app
cargo test
```

## Boundary rules

- Keep `main.rs` thin: compose modules and launch.
- Share cross-feature primitives through `shared`, not feature-to-feature imports.
- Screen routing and cross-feature side effects belong in `app/`, not in feature modules.
- Use private modules by default; expose only composition-facing APIs.
