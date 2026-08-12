# DEDUCED

DEDUCED is an offline-first deduction game built as a Rust workspace. The core deduction engine is intentionally independent from UI, Bevy, networking, persistence, accounts, ads, and any future backend.

The first goal was a small playable CLI game using starter Cars, Companies, and Countries content. That loop now also runs in `deduced-game`, a Bevy client windowed at phone size (390x844) as an early visual demo, and in `deduced-web`, a browser client served by a small Axum backend, both sitting on top of the same reusable rules.

## Workspace Layout

```text
assets/
  images/
    cars/
    companies/
    countries/
  fonts/
content/
  categories/
  answers/
crates/
  deduced-core/      # pure game rules and scoring
  deduced-content/   # JSON loading and validation
  deduced-save/      # local profile/stats/storage abstractions
  deduced-bot/       # bot guessing policies
apps/
  deduced-cli/       # first playable client
  deduced-game/      # Bevy client, windowed as a phone-emulator demo
  deduced-web/       # Axum backend + static browser client
docs/
  architecture.md
  phases.md
  content-format.md
crates/deduced-core/tests/
```

## Architecture Rule

`deduced-core` must not depend on:

```text
Bevy
SQLite
HTTP
WebSockets
Android
iOS
Steam
Ads
Accounts
```

It should only understand answers, guesses, attributes, comparisons, rounds, attempts, score, win, and loss.

## First Run

CLI:

```bash
cargo run -p deduced-cli
```

Then choose a category and type guesses by answer name or id.

Bevy phone-emulator demo:

```bash
cargo run -p deduced-game
```

Opens a fixed 390x844 window: pick a category, tap answers to guess, and read the color-coded clue chips (green = match, orange = higher, blue = lower, red = different, yellow = partial).

Web client:

```bash
cargo run -p deduced-web
```

Starts a server at `http://127.0.0.1:4173` (must be run from the workspace root so it can find `content/`). Open that URL in a browser: pick a category, type or tap a quick guess, and submit. The bottom-nav Daily/Versus/Profile/Store screens and theme switcher are static UI mockups with no backend yet. Use the device-switcher bar at the top (Full / Phone / Tablet / Desktop) to preview the layout at different aspect ratios without resizing the browser window.

## Development Checks

```bash
cargo fmt --all
cargo test --workspace
cargo run -p deduced-cli
cargo run -p deduced-game
cargo run -p deduced-web
```

## Current Milestone

`v0.0.1` focuses on:

- generic attributes
- data-defined categories
- deterministic round generation from seeds
- JSON content loading
- basic CLI play loop
- initial scoring
- a Bevy phone-emulator demo of the same loop
- a browser client (Axum backend + static frontend) of the same loop

See [docs/phases.md](docs/phases.md) for the full staged plan (the Bevy client was built ahead of its Phase 4 slot by request).
