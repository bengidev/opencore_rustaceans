# opencore_rustaceans

OpenCore Rustaceans — Rust desktop shell with internal modular boundaries.

## Onboarding module

First-run onboarding structured as an **internal module** with GoF design patterns and **TDD** colocated tests.

```bash
cd opencore_rustaceans
cargo test onboarding   # 18 unit tests
cargo run               # launch onboarding window
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for module layout, patterns, and boundary rules.

## Project layout

```text
opencore_rustaceans/     # Cargo package
docs/ARCHITECTURE.md
src/
├── main.rs              # composition root
├── lib.rs
├── features/onboarding/
└── shared/design/
```
