# Build Phases

This roadmap follows `new-version.md`: finish a production-quality offline Solo client first, then add online services only when a feature truly needs shared state, trust, synchronization, or remote players.

Permanent rules:

- `deduced-core` is the canonical game-rule crate.
- `deduced-core` must not depend on Bevy, Axum, filesystem APIs, databases, networking, platform APIs, UI, or time.
- Solo must remain playable without internet.
- Content remains data-driven.
- Rounds must remain reconstructable from `category + seed + content_version`.
- `deduced-web` is a UI prototype and design reference, not the production client/server architecture.
- The current prototype web backend's one-global-session model must not be expanded into production.

## Current Baseline

Goal: preserve what is already valuable and identify what is prototype-only.

- Keep `deduced-cli` as the fast developer tool for content testing, seed reproduction, debugging, and rule play-testing.
- Keep `deduced-game` as the current Bevy graphical client while it is gradually shaped into the production client.
- Keep `deduced-web` as a fast HTML/CSS/JavaScript design sandbox.
- Keep `deduced-core`, `deduced-content`, `deduced-save`, and `deduced-bot` separated by responsibility.
- Treat any Solo flow that calls web API round/guess endpoints as prototype-only.

Exit criteria:

- CLI, Bevy client, and web prototype remain runnable during the transition.
- Shared crates remain independent from client, server, and platform concerns.
- Documentation makes the prototype/production boundary clear.

## Phase A: Production Solo Foundation

Goal: represent a complete Solo game without Bevy, Axum, JavaScript, or platform-specific code.

- Create or complete `crates/deduced-gameplay`.
- Add the gameplay crate to the workspace.
- Add the target modules:
  - `controller.rs`
  - `state.rs`
  - `known_facts.rs`
  - `reveal.rs`
  - `result.rs`
- Implement `GameController` around the core round flow.
- Implement `GameViewState` for client-facing state.
- Implement `KnownFact` derivation in Rust.
- Implement `RevealState`.
- Implement `GameResult`.
- Move application-level deduction helpers out of prototype UI code.
- Prefer passing already-loaded content into gameplay rather than making gameplay load files.

Exit criteria:

- A complete playable Solo session can be expressed through `deduced-gameplay`.
- Known facts, reveal progression, game state, attempt progression, result generation, repeated guesses, and completion are covered by focused tests.
- `deduced-core` remains the only source of comparison, round, and scoring rules.

## Phase B: Real Client Shell

Goal: turn the current Bevy prototype into the start of the production game client.

- Gradually restructure `apps/deduced-game` toward the future `apps/deduced-client`.
- Keep Bevy dependencies inside the client app.
- Introduce a real application state machine.
- Start with the production states:
  - Home
  - Categories
  - Playing
  - Result
- Add reusable UI components only as they are needed by these screens.
- Update the client to consume `GameViewState` instead of manually coordinating every `Round` concern.

Exit criteria:

- Navigation flows cleanly from Home to Categories to Playing to Result.
- The graphical client renders gameplay through the gameplay layer.
- No Bevy type leaks into shared game logic.

## Phase C: Home Screen

Goal: build the production version of the web prototype's Home screen.

- Provide a clear Solo entry point.
- Include Daily, Versus, and Profile as placeholders if shown.
- Keep account creation out of first launch.
- Support touch, mouse, and keyboard input.
- Keep the screen responsive for phone-first layouts, with room to adapt to tablet and desktop later.

Exit criteria:

- A new player can launch the client and start Solo without explanation.
- Solo remains available with no server connection.
- Placeholder modes do not imply completed backend behavior.

## Phase D: Category Screen

Goal: select Solo categories from local bundled content.

- Load categories locally through `deduced-content`.
- Render category cards from content metadata.
- Show category name, answer count, icon, and attempts where available.
- Start a local Solo game from the selected category.
- Do not call a backend to list categories or start a Solo round.

Exit criteria:

- Category selection works offline.
- The client can start a deterministic local round from category content and seed.
- Category display is data-driven rather than hardcoded screen logic.

## Phase E: Production Game Screen

Goal: implement the streamlined one-screen deduction loop.

- Build the screen around:
  - header
  - attempts
  - image card
  - reveal progress
  - known facts
  - guess history
  - autocomplete
  - guess button
  - result overlay
- Keep the player one action away from another guess.
- Submit guesses through `GameController`.
- Render clues, known facts, attempts, and status from gameplay state.
- Hide, remove, or clearly disable already-guessed answers.
- Avoid a "next clue" or "next guess" interaction.
- Make wrong-guess feedback quick and useful.

Exit criteria:

- A full round can be played on one screen from first guess to result.
- The UI no longer invents gameplay meaning that belongs in Rust.
- Result overlay handles both solved and answer-revealed outcomes.

## Phase F: Real Image Reveal

Goal: add real answer imagery with deterministic progressive reveal.

- Add answer asset references to content where needed.
- Use one main answer image at first.
- Implement deterministic masking from `RevealState`.
- Reveal more after incorrect guesses.
- Fully reveal the image at result.
- Avoid storing many separate reveal-stage images unless a category proves it needs them.
- Treat image licensing as a release requirement before shipping large commercial content sets.

Exit criteria:

- Reveal state is deterministic and reusable.
- The renderer controls the visual mask, while gameplay controls the reveal level.
- Content validation can catch missing or broken asset references.

## Phase G: Local Save

Goal: make offline Solo progress persistent.

- Wire `deduced-save` into the client.
- Store local player identity.
- Store settings.
- Store round stats.
- Store streaks.
- Store category stats.
- Store XP only if early progression is implemented.
- Keep storage implementation details out of `deduced-core`.

Exit criteria:

- Closing and reopening the game preserves progress.
- Solo progress updates locally immediately after a round.
- No account or server is required for local play.

## Phase H: Solo MVP Complete

Goal: reach the first major product milestone.

Required flow:

```text
launch
  ↓
home
  ↓
choose Solo
  ↓
choose category
  ↓
play round
  ↓
win / lose
  ↓
result
  ↓
next round
```

Required properties:

- Zero internet required.
- Known facts are useful.
- Reveal pacing feels good.
- Scoring is understandable.
- Result feedback makes the player want another round.
- Cars, Companies, and Countries are play-tested.

Exit criteria:

- The Solo loop is polished enough to evaluate retention honestly.
- `cargo test --workspace` passes.
- Backend complexity is still deferred.

## Phase I: Content Expansion

Goal: improve replayability after Solo is fun.

- Expand Cars.
- Expand Countries.
- Expand Companies.
- Add a fourth category only after the initial categories are balanced.
- Improve category metadata.
- Add or improve aliases where useful.
- Add content manifest/version support.
- Improve validation for duplicate IDs, duplicate names, missing required attributes, unknown categories, invalid comparisons, empty values, bad numeric values, and broken asset paths.

Exit criteria:

- The base content set supports repeated offline play.
- Content quality is high enough that clues feel fair and useful.
- Deterministic reconstruction includes content versioning.

## Phase J: Daily Challenge

Goal: create the production backend only when Daily needs it.

- Add or complete `crates/deduced-protocol` for shared DTOs.
- Create or complete `apps/deduced-server` as the production backend.
- Do not expand the prototype global-session backend into production.
- Implement health checks.
- Implement current Daily challenge lookup.
- Represent Daily as:
  - `challenge_id`
  - `category`
  - `seed`
  - `content_version`
- Let the client reconstruct and play the Daily locally.
- Submit replay data after completion.
- Validate submissions server-side by replaying guesses through the shared Rust engine.
- Add Daily leaderboard only after validation is reliable.
- Support downloading Daily online, completing it offline, and submitting later.

Exit criteria:

- Daily is the first production feature that requires server communication.
- The server validates score, attempts, and result.
- The client is not trusted to report competitive truth.

## Phase K: Cloud Profile

Goal: add online identity and sync after local profile behavior is stable.

- Add guest backend identity if Daily or sync requires it.
- Keep account creation optional.
- Support later account upgrade/linking.
- Sync profile data or meaningful profile events.
- Preserve offline-first behavior during server outages.
- Avoid building a large auth system before it unlocks concrete features.

Exit criteria:

- A player can continue Solo while offline.
- Local progress can reconcile with cloud profile state.
- First launch still does not require registration.

## Phase L: Friend Versus

Goal: build the first real-time multiplayer mode.

- Create private lobbies.
- Generate join codes.
- Join via code.
- Add ready state.
- Use WebSocket for live match state.
- Use deterministic same-target rounds.
- Share messages through `deduced-protocol`.
- Show opponent progress without revealing opponent guesses.
- Track round winner and match winner.
- Validate competitive actions and match results server-side.

Exit criteria:

- Two players can play the same hidden target independently in a private match.
- Opponent progress creates pressure without leaking deduction information.
- The server does not trust client-reported wins, guesses, attempts, or scores.

## Phase M: Matchmaking

Goal: add public matching after private games are stable.

- Add Quick Match.
- Add queue and leave queue behavior.
- Add match-found flow.
- Reuse the Friend Versus match engine.
- Add disconnect handling.
- Add reconnection.
- Add match history.
- Add ranking only after unranked matchmaking is reliable.
- Avoid Redis or distributed infrastructure until scale requires it.

Exit criteria:

- Players can queue, match, finish a match, and receive recorded results.
- Reconnect and disconnect behavior is defined.
- Ranking builds on stable match results instead of replacing the match model.

## Phase N: Store and Monetization

Goal: add commercial systems only after engagement justifies them.

- Keep Store UI prototype-only until the game has proven retention.
- Consider themes, avatars, cosmetics, optional category packs, ad-free upgrade, or premium challenge packs.
- Avoid pay-to-win mechanics.
- Avoid building a large economy, battle pass, season system, or store backend prematurely.

Exit criteria:

- Monetization does not compromise deduction gameplay.
- Paid systems do not block offline Solo.

## Testing Strategy

`deduced-core` should test:

- round generation
- deterministic seeds
- comparisons
- duplicate guesses
- win and loss
- scoring
- numeric attributes
- exact attributes
- tags
- partial matches

`deduced-gameplay` should test:

- reveal progression
- known-fact aggregation
- game state
- attempt progression
- result generation
- repeated guesses
- game completion

`deduced-content` should test:

- JSON loading
- invalid answers
- missing attributes
- duplicate IDs
- unknown categories
- broken asset references once assets exist

Server tests should cover:

- Daily validation
- match creation
- join codes
- submission verification
- authorization where applicable

Integration tests should prove:

- The same `category + seed + content_version` produces the same answer for client and server.
- Replaying the same guess sequence reaches the same final state on client and server.

## CI Direction

Eventually CI should run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Content validation should be added once the validator exists.

## Development Commands

Current commands:

```bash
cargo test --workspace
cargo run -p deduced-cli
cargo run -p deduced-game
cargo run -p deduced-web
```

Expected later commands:

```bash
cargo run -p deduced-client
cargo run -p deduced-server
```

## Immediate Checklist

- [ ] Complete `crates/deduced-gameplay`.
- [ ] Complete `GameController`.
- [ ] Complete `GameViewState`.
- [ ] Complete `KnownFact`.
- [ ] Complete `RevealState`.
- [ ] Complete `GameResult`.
- [ ] Move reusable deduction-state derivation into Rust.
- [ ] Update the graphical client to consume `GameViewState`.
- [ ] Formalize application screen state.
- [ ] Build production Home.
- [ ] Build production Category screen.
- [ ] Build production Game screen.
- [ ] Implement autocomplete.
- [ ] Implement clue-history rendering.
- [ ] Implement result overlay.
- [ ] Add real answer image support.
- [ ] Add deterministic progressive image reveal.
- [ ] Wire in `deduced-save`.
- [ ] Store local stats.
- [ ] Persist local profile.
- [ ] Verify Solo works fully offline.
- [ ] Play-test extensively.
- [ ] Start production backend work only after Solo MVP is complete.
