# Build Phases

## Phase 0: Workspace Foundation

Goal: create the workspace shape and rule boundaries.

- Create Rust workspace.
- Add `deduced-core`, `deduced-content`, `deduced-save`, `deduced-bot`.
- Add `deduced-cli` and placeholder `deduced-game`.
- Add starter JSON content.
- Add project docs and agent guidance.

Exit criteria:

- `cargo test --workspace` passes.
- `cargo run -p deduced-cli` starts a playable text loop.

## Phase 1: Core Deduction Loop

Goal: make the game mechanically solid without UI work.

- Finish generic comparison behavior.
- Add duplicate guess handling.
- Add better score rules.
- Add deterministic daily seed helpers.
- Add tests for numeric, exact, bool, tags, win, loss, and scoring.
- Expand starter content to 10 answers per category.

Exit criteria:

- CLI rounds are complete and understandable.
- Core tests cover the main game states.

## Phase 2: CLI Play-Test MVP

Goal: use the CLI to decide if DEDUCED is fun.

- Add category selection UX.
- Add fuzzy or prefix guess matching.
- Hide already guessed answers or mark them clearly.
- Add a simple end-of-round summary.
- Add optional seed input for replayable rounds.
- Add basic content quality checks.

Exit criteria:

- A full round can be played repeatedly without developer knowledge.
- The team can play-test Cars, Companies, and Countries.

## Phase 3: Progressive Reveal Assets

Goal: introduce image reveal data without complex rendering.

- Define reveal asset path convention.
- Add content fields for reveal image sets if needed.
- Start with manually prepared reveal images.
- Keep reveal level as pure round state.

Exit criteria:

- The client can ask the round for a reveal level.
- The asset lookup is deterministic and data-driven.

## Phase 4: Bevy Client

Goal: build the first graphical client on top of the proven core.

- Add Bevy dependencies only to `apps/deduced-game`.
- Build menu, category select, game, and results screens.
- Render clue table from `GuessResult`.
- Render progressive images.
- Keep all game-rule decisions in `deduced-core`.

Exit criteria:

- Bevy client can complete the same rounds as the CLI.
- No Bevy dependency leaks into core crates.

## Phase 5: Local Save and Stats

Goal: remember profile and play history locally.

- Implement concrete save storage in `deduced-save`.
- Track rounds played, wins, streaks, best score, and category stats.
- Wire saves into CLI first if useful, then Bevy.

Exit criteria:

- Local progress survives app restart.
- Save code remains independent from core rules.

## Phase 6: Bot and Local Multiplayer

Goal: add non-network opponents.

- Implement bot policies by difficulty.
- Support same-device multiplayer.
- Keep bot decisions based on public core/content APIs.

Exit criteria:

- Player can play against a basic bot.
- Same-device multiplayer can run deterministic rounds.

## Phase 7: Online-Ready Foundation

Goal: prepare for backend work without building it prematurely.

- Finalize content versioning.
- Add challenge code format.
- Add replay verification tests.
- Define network message types in a future crate.

Exit criteria:

- A round can be reconstructed from category, seed, and content version.
- Backend requirements are clear enough for implementation.

## Phase 8: Backend and Network Play

Goal: add online play after the game loop is proven.

- Add future `deduced-network` crate.
- Add Axum backend only after local play is stable.
- Add matchmaking, friend challenges, and account systems incrementally.

Exit criteria:

- Online play uses the same deterministic engine as offline play.
- Server does not become the source of game-rule truth.
