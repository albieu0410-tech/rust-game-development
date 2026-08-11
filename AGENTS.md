# Agent Guide

This repo is organized around a strict separation between the reusable deduction engine and every client or service that may use it.

## Non-Negotiable Boundaries

- Keep `crates/deduced-core` free of Bevy, filesystem, networking, database, time, platform, and UI dependencies.
- Add category behavior through data where possible, not category-specific Rust structs.
- Preserve deterministic round creation from `category + seed + content_version`.
- Prefer small, testable pure functions in core.
- Keep app-specific rendering, input, assets, and persistence outside core.

## Preferred Workflow

1. Change `deduced-core` first when modifying game rules.
2. Add or update focused tests for deduction/scoring behavior.
3. Update content validation when the JSON shape changes.
4. Keep `deduced-cli` as the fastest way to play-test the loop.
5. Prefer proving new rules in the CLI or core tests before wiring them into `deduced-game` screens (`apps/deduced-game/src/screens/`).

## Commands

```bash
cargo fmt --all
cargo test --workspace
cargo run -p deduced-cli
cargo run -p deduced-game
```

## Content Rules

- Category files live in `content/categories/`.
- Answer files live in `content/answers/`.
- Attribute keys in answers must be declared by the category.
- Category comparison rules decide how each attribute is compared.
- Image paths should be stored as relative asset paths.

## Design Direction

The game should stay fast, readable, and deduction-focused. Avoid adding accounts, network transport, monetization, progression systems, or complex shaders before the core game loop is fun.
