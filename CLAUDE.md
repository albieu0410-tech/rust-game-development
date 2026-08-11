# CLAUDE.md

Guidance for Claude or other coding agents working in this repository.

## Project Intent

DEDUCED is a Rust workspace for a reusable deduction game engine plus multiple clients. The first useful deliverable was a playable CLI that proves the core loop; a Bevy client (`deduced-game`) now also exists as a windowed, phone-aspect-ratio (390x844) demo built on the same `deduced-core`/`deduced-content` APIs. This pulled Bevy work ahead of the `docs/phases.md` Phase 4 schedule at explicit user request — the CLI is still the fastest place to play-test new rules before wiring them into Bevy screens.

## Key Constraint

`deduced-core` is a pure rules crate. Do not introduce dependencies or concepts from UI, storage, network, accounts, ads, or platform-specific code.

## Crate Responsibilities

- `deduced-core`: answers, attributes, categories, comparison, rounds, guesses, scoring.
- `deduced-content`: load and validate JSON content.
- `deduced-save`: profile, stats, and storage traits.
- `deduced-bot`: bot policies using the public core API.
- `deduced-cli`: terminal prototype for rapid play-testing.
- `deduced-game`: Bevy client (menu/playing/result screens in `src/screens/`); reads the same `content/` JSON and drives rounds through `deduced-core::Round`.

## Before Editing

- Inspect existing module patterns before adding abstractions.
- Keep changes scoped to the requested milestone.
- Do not rewrite unrelated files.
- Do not hard-code category-specific logic unless a phase document explicitly calls for a prototype exception.

## Verification

Run:

```bash
cargo fmt --all
cargo test --workspace
```

For game-loop changes, also run:

```bash
cargo run -p deduced-cli
```

For `deduced-game` UI changes, also run and click through it:

```bash
cargo run -p deduced-game
```
