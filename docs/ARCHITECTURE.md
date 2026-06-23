# Architecture

OpenCore Rustaceans is a single Cargo package with **internal module**
boundaries — modules stay inside the crate until a real external consumer
appears.

## Composition root

`src/main.rs` is the application composition root. It chooses persistence
implementations and launches the onboarding Iced runtime. No feature logic
accumulates there.

## Internal modules

```text
src/
├── main.rs                                # composition root
├── lib.rs                                 # test / embedder surface
├── features/
│   ├── mod.rs                             # feature registry
│   └── onboarding/
│       ├── mod.rs                         # feature facade (GoF Facade)
│       ├── onboarding_messages.rs         # Command messages
│       ├── onboarding_state.rs            # State reducer
│       ├── onboarding_dynamics.rs         # orb animation dynamics
│       ├── onboarding_feature_card_dynamics.rs
│       ├── onboarding_outcome.rs          # routing outcomes
│       ├── onboarding_persistence.rs      # Strategy trait
│       ├── onboarding_file_persistence.rs # filesystem Strategy
│       ├── onboarding_memory_persistence.rs
│       ├── onboarding_view.rs             # Iced view
│       ├── onboarding_feature_card_icon.rs
│       ├── onboarding_galaxy_orb.rs       # Template Method canvas
│       └── onboarding_scene_backdrop.rs
│   └── welcome/
│       ├── mod.rs                         # feature facade (GoF Facade)
│       ├── welcome_model.rs               # Composite screen catalog
│       ├── welcome_messages.rs            # Command messages
│       ├── welcome_outcome.rs             # routing outcomes
│       ├── welcome_state.rs               # State reducer
│       └── welcome_view.rs                # Iced view
└── shared/
    ├── mod.rs
    └── design/
        ├── design_palette.rs
        ├── design_tokens.rs
        └── design_theme.rs                # Factory Method resolver
```

## Design patterns (GoF)

| Pattern | Where | Role |
|---------|-------|------|
| **Facade** | `onboarding/mod.rs`, `welcome/mod.rs` | Hides prefixed siblings; exposes `run`, `view`, state types |
| **Composite** | `welcome_model.rs` | Screen → sections → items content tree |
| **Strategy** | `OnboardingPersistence` | Swaps file vs in-memory backends |
| **Command** | `OnboardingMessage` | Encodes user intents decoupled from widgets |
| **State** | `OnboardingState::update` | Pure transitions + `OnboardingOutcome` routing |
| **Factory Method** | `OpenCoreTheme::from_mode`, persistence constructors | Pick concrete products |
| **Template Method** | iced `Program` impls | Shared draw lifecycle for canvas widgets |

## TDD workflow

Tests are colocated in each `onboarding_*.rs` module. Typical cycle:

1. **Red** — add a failing `#[test]` beside the reducer or strategy.
2. **Green** — implement the smallest change in the sibling module.
3. **Refactor** — keep the facade thin; run `cargo test onboarding`.

Run the suite:

```bash
cargo test onboarding
```

## Boundary rules

- Keep `main.rs` thin: compose modules and launch.
- Share cross-feature primitives through `shared`, not feature-to-feature imports.
- Use private modules by default; expose only composition-facing APIs.
