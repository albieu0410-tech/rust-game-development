# Agent Guide

This repo is organized around a strict separation between the reusable deduction engine, the playable-session layer above it, and every client or service that uses them. See `docs/phases.md` for the full staged roadmap (currently implemented through Phase 13) and `docs/new-version.md` for the longer-form architecture rationale.

## Non-Negotiable Boundaries

- Keep `crates/deduced-core` free of Bevy, Axum, sqlx, filesystem, networking, database, time, platform, and UI dependencies.
- Keep `crates/deduced-gameplay` and `crates/deduced-protocol` free of Bevy/Axum/sqlx too — `deduced-gameplay` sits between core and clients; `deduced-protocol` is wire DTOs only, with no game logic and no dependency on `deduced-core`.
- Add category behavior through data where possible, not category-specific Rust structs.
- Preserve deterministic round creation from `category + seed + content_version` — this is what makes Daily and multiplayer possible without the server running the game loop live.
- Prefer small, testable pure functions in core and in `deduced-gameplay`.
- Keep app-specific rendering, input, assets, and persistence outside core.
- The server is the authority on competitive results. `deduced-server` never trusts a client's claimed win/score — it replays guesses through `deduced-core` itself (see `services::daily::replay_submission` and `multiplayer::match_actor`). Follow that pattern for any new competitive feature.
- Solo must stay fully playable with the server unreachable. Anything that talks to `deduced-server` (profile sync, Daily, Versus) must degrade silently, not block or crash Solo.
- `deduced-web` is a UI/design prototype, not production backend architecture — don't grow its Axum API as if it were `deduced-server`.

## Preferred Workflow

1. Change `deduced-core` first when modifying game rules; add focused tests for deduction/scoring behavior there.
2. If the change affects what a client renders (known facts, reveal progression, result), update `deduced-gameplay` next rather than deriving that logic again in a client.
3. Update content validation (`crates/deduced-content/src/validation.rs`) when the JSON shape changes.
4. Keep `deduced-cli` as the fastest way to play-test the loop — it also carries the dev tools (`--seed`, `--reveal-answer`) for reproducing/inspecting a specific round without guessing blind.
5. Prefer proving new rules in the CLI or core/gameplay tests before wiring them into `deduced-game` screens (`apps/deduced-game/src/screens/`).
6. For anything touching `deduced-server`, start Postgres and actually exercise the endpoint/WS flow — `cargo check` alone won't catch a broken SQL query or a match-actor logic error.

## Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p deduced-cli
cargo run -p deduced-game
cargo run -p deduced-web
docker compose up -d postgres && cargo run -p deduced-server
```

## Content Rules

- Category files live in `content/categories/`.
- Answer files live in `content/answers/`.
- Attribute keys in answers must be declared by the category.
- Category comparison rules decide how each attribute is compared.
- Image paths should be stored as relative asset paths.
- An attribute that never varies across a category's answers isn't a useful clue — check for this when adding content (e.g. countries all being `coastline: true` was a real bug caught by playtesting, fixed by adding landlocked countries).

## Design Direction

The game should stay fast, readable, and deduction-focused. Monetization/store backend is intentionally still not built — the docs are explicit that it should wait until engagement is proven; don't add one preemptively. Ranked matchmaking is similarly deferred until basic Quick Match has proven reliable. Real per-answer images are blocked on an art/licensing decision, not a technical one — don't source images without that decision being made first.
