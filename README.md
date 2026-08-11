# DEDUCED

DEDUCED is an offline-first deduction game built as a Rust workspace. The core deduction engine is intentionally independent from UI, Bevy, networking, persistence, accounts, ads, and any future backend.

The first goal is a small playable CLI game using starter Cars, Companies, and Countries content. Once the core loop feels good, Bevy and persistence can sit on top of the same reusable rules.

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
  deduced-game/      # future Bevy client
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

```bash
cargo run -p deduced-cli
```

Then choose a category and type guesses by answer name or id.

## Development Checks

```bash
cargo fmt --all
cargo test --workspace
cargo run -p deduced-cli
```

## Current Milestone

`v0.0.1` focuses on:

- generic attributes
- data-defined categories
- deterministic round generation from seeds
- JSON content loading
- basic CLI play loop
- initial scoring

See [docs/phases.md](docs/phases.md) for the full staged plan.
