# CLAUDE.md

Guidance for Claude or other coding agents working in this repository.

## Project Intent

DEDUCED is a Rust workspace for a reusable deduction game engine plus multiple clients and a production backend. The staged roadmap lives in `docs/phases.md` (distilled from the longer `docs/new-version.md`); the workspace currently implements Phases 1–13 of it — everything through Friend Versus and Quick Match multiplayer, short of the store/monetization backend, which is intentionally not built yet. Phase 5 (real per-answer images) is also not built: it's blocked on an art/licensing decision, not a technical one.

`deduced-web` (Axum + static HTML/CSS/JS) is a UI **prototype and design reference** only — quick to iterate on for layout/interaction ideas, but not production architecture. `deduced-game` (Bevy) is the production client. `deduced-server` is the production backend; it does not share code or state with `deduced-web`'s prototype API.

## Key Constraint

`deduced-core` is a pure rules crate. Do not introduce dependencies or concepts from UI, storage, network, accounts, ads, or platform-specific code — not Bevy, not Axum, not sqlx, not WebSockets. It should only understand answers, guesses, attributes, comparisons, rounds, attempts, score, win, and loss. `deduced-gameplay` and `deduced-protocol` also must not depend on Bevy/Axum/sqlx; they sit between `deduced-core` and the actual clients/server.

## Crate & App Responsibilities

- `deduced-core`: answers, attributes, categories, comparison, rounds, guesses, scoring. The single source of game-rule truth — client and server both depend on it; never reimplement comparison/scoring logic elsewhere.
- `deduced-gameplay`: session layer above `deduced-core` — `GameController`, `GameViewState`, known-fact derivation, `RevealState`, `GameResult`. Known-fact/reveal logic belongs here, not duplicated in client UI code.
- `deduced-content`: load and validate JSON content (`content/categories/`, `content/answers/`).
- `deduced-save`: local profile/stats/storage traits + `FileSaveStorage`. Guest-first local `player_id`, no account required.
- `deduced-bot`: bot policies using the public core API.
- `deduced-protocol`: shared wire DTOs for client↔server communication (Daily, profile sync, multiplayer WS messages). No game logic, no `deduced-core` dependency — just message shapes.
- `deduced-cli`: terminal client and the primary developer play-test/debug tool. Supports `--seed <n>` (force/reproduce a round) and `--reveal-answer <category>` (print the target and exit, non-interactive) — use these instead of guessing blind when testing.
- `deduced-game`: Bevy production client (`Home`/`Categories`/`Playing`/`Result` screens in `src/screens/`), drives rounds through `deduced-gameplay::GameController`, persists via `deduced-save`, best-effort syncs profile to `deduced-server` on a background thread (never blocks Solo).
- `deduced-web`: Axum backend + static browser client. Design/UI prototype only — do not extend its API as if it were production backend architecture.
- `deduced-server`: production backend (Axum + sqlx + PostgreSQL, local dev via `docker compose up -d postgres`). Owns Daily challenge generation/validation, profile sync, and the Friend Versus / Quick Match multiplayer engine (`src/multiplayer/`). State is keyed by proper identifiers (`challenge_id`, `player_id`, `match_id`) — never a single global session.

## Server Authority Rule

The server never trusts a client's claimed win, score, or attempt count. Daily submissions and multiplayer guesses are replayed/validated server-side through `deduced-core` itself (`services::daily::replay_submission`, `multiplayer::match_actor`). If you add a new competitive feature, follow the same pattern — don't add an endpoint that just accepts and stores whatever result the client reports.

## Before Editing

- Inspect existing module patterns before adding abstractions.
- Keep changes scoped to the requested milestone.
- Do not rewrite unrelated files.
- Do not hard-code category-specific logic unless a phase document explicitly calls for a prototype exception.
- Solo must stay fully playable offline — don't make it depend on `deduced-server` being reachable.

## Verification

Run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
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

For `deduced-server` changes, start Postgres first and actually exercise the endpoint(s) you touched (curl for HTTP, a small WS test client for multiplayer) — don't rely on `cargo check` alone for anything touching the database or the match actor:

```bash
docker compose up -d postgres
cargo run -p deduced-server
```
