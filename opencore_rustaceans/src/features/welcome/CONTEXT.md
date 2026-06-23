# Welcome context

Home screen shown after onboarding: centered welcome header, get-started
actions, and recent-project shortcuts. GUI-only for now — row presses emit
[`WelcomeOutcome::ActionRequested`] for the host to wire later.

## Glossary

| Term | Meaning |
|------|---------|
| **Welcome** | Post-onboarding home screen with action rows and recent projects |
| **Screen** | [`WelcomeScreen`] — root Composite of sections |
| **Section** | Titled group (`GET STARTED`, `RECENT PROJECTS`) |
| **Item** | One actionable row with icon, label, optional shortcut |
| **Reducer** | `WelcomeState::update` — pure message → state + outcome |
| **Outcome** | `WelcomeOutcome` — routing hint for the composition root |

## Public surface (Facade)

Callers outside the module import only from `features::welcome`:

- `run` — standalone Iced application entry
- `view` — embeddable view for multi-window hosts
- `WelcomeState`, `WelcomeMessage`, `WelcomeOutcome`
- `WelcomeItemId`, `WelcomeScreen`, `default_screen`

## Branding

Uses **OpenCore** naming and the shared monochrome design tokens.
