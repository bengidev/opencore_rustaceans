# Welcome context

Home screen shown after onboarding: centered welcome header, get-started
actions, and recent-project shortcuts. Row presses and keyboard shortcuts
run real workflows (file/folder pickers, git clone, command palette) and
update persisted recent-project history.

## Glossary

| Term | Meaning |
|------|---------|
| **Welcome** | Post-onboarding home screen with action rows and recent projects |
| **Screen** | [`WelcomeScreen`] — root Composite of sections |
| **Section** | Titled group (`GET STARTED`, `RECENT PROJECTS`) |
| **Item** | One actionable row with icon, label, optional shortcut |
| **Recent history** | Project paths on `WelcomeState`; empty history hides the recent section |
| **History** | [`WelcomeHistory`] Strategy — file or in-memory recent-path store |
| **Reducer** | `WelcomeState::update` — pure message → state + outcome |
| **Outcome** | `WelcomeOutcome` — routing hint for the composition root |
| **Overlay** | Command palette or clone-repository modal above the home canvas |

## Actions

| Item | Behavior |
|------|----------|
| New File | Save dialog → creates empty file → status banner |
| Open Project | Folder picker → validates path → history bump → `WorkspaceOpened` outcome |
| Clone Repository | Modal URL → `git clone` into `~/OpenCore/repositories` |
| Open Command Palette | Double-shift or row press → searchable command list |
| Recent project | Validates stored path → history bump → `WorkspaceOpened` outcome |

`WorkspaceOpened` is the composition-root hook for a future workspace shell; the welcome screen shows a status banner until that host exists.

Keyboard: `⌘N`, `⌘O`, `⌘1`–`⌘5`, double `⇧`, `Esc` dismisses overlays.

## Public surface (Facade)

Callers outside the module import only from `features::welcome`:

- `run` / `run_with_history` — Iced application entry
- `view` — embeddable view for multi-window hosts
- `subscription` — keyboard shortcuts for embeddable hosts (`⌘N`, `⌘O`, `⌘1`–`⌘5`, double `⇧`, `Esc`)
- `WelcomeState`, `WelcomeMessage`, `WelcomeOutcome`
- `WelcomeItemId`, `WelcomeScreen`, `build_screen`, `default_screen`
- `WelcomeHistory`, `FileWelcomeHistory`, `InMemoryWelcomeHistory`

## Branding

Uses **OpenCore** naming and the shared monochrome design tokens.
