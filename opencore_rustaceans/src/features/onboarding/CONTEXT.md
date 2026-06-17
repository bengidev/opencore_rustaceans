# Onboarding context

First-run onboarding flow: immersive landing with hold-to-zoom galaxy orb,
theme toggle, and primary CTA to enter the app.

## Glossary

| Term | Meaning |
|------|---------|
| **Onboarding** | Single-page first-run experience shown before the main workspace |
| **Sentinel** | Filesystem flag (`onboarding-completed.flag`) marking completion |
| **Reducer** | `OnboardingState::update` — pure message → state + outcome |
| **Outcome** | `OnboardingOutcome` — routing hint for the composition root |
| **Persistence** | `OnboardingPersistence` strategy — query and mark completion |

## Public surface (Facade)

Callers outside the module import only from `features::onboarding`:

- `run` — standalone Iced application entry
- `view` — embeddable view for multi-window hosts
- `OnboardingState`, `OnboardingMessage`, `OnboardingOutcome`
- `OnboardingPersistence`, `mark_completed`

## Branding

Uses **OpenCore** naming (`OpenCoreTheme`, `com.opencore.opencore` data directory qualifier).
