# Chat context

Thread, composer, and streaming state for the workspace chat panel.

## Glossary

| Term | Meaning |
|------|---------|
| **Thread** | [`ChatThread`] — append-only user/assistant messages |
| **Chat state** | [`ChatState`] — draft, thread, streaming flags |
| **Chat event** | [`ChatEvent`] — composer and thread UI messages |
| **Outcome** | [`ChatOutcome`] — routing hint for the workspace reducer |

## Public surface (Facade)

Callers outside the module import from `features::chat`:

- `ChatThread`, `ChatMessage`, `ChatRole`
- `ChatState`, `ChatEvent`, `ChatOutcome`
- `body`, `composer` — embeddable chat widgets
- `text_input_style`, `control_radius`, `chip_button_style` — shared control styling

The workspace module composes chat with project header, model picker, and overlays.
