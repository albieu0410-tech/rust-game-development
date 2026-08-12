# DEDUCED

DEDUCED is an offline-first deduction game built as a Rust workspace. The core deduction engine is intentionally independent from UI, Bevy, networking, persistence, accounts, ads, and any future backend.

The first goal was a small playable CLI game using starter Cars, Companies, and Countries content. That loop now also runs in `deduced-game`, a Bevy client windowed at phone size (390x844), and in `deduced-web`, a browser client served by a small Axum backend — both sitting on top of the same reusable rules via the shared `deduced-gameplay` session layer. A separate production backend, `deduced-server`, adds the features that genuinely need a server: an authoritative Daily challenge, best-effort cloud profile sync, and real-time Friend Versus / Quick Match multiplayer — while Solo stays fully playable offline.

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
  deduced-gameplay/  # playable-session layer (GameController, known facts, reveal state)
  deduced-content/   # JSON loading and validation
  deduced-save/      # local profile/stats/storage (FileSaveStorage)
  deduced-bot/       # bot guessing policies
  deduced-protocol/  # shared client/server DTOs (Daily challenge, health, ...)
apps/
  deduced-cli/       # first playable client, now also a dev/playtest tool (--seed, --reveal-answer)
  deduced-game/      # Bevy client: Home/Categories/Playing/Result, local save
  deduced-web/       # Axum backend + static browser client (design prototype)
  deduced-server/    # production backend: Daily challenge, Postgres via sqlx
docs/
  architecture.md
  phases.md          # staged roadmap, Phase 0 through Phase 13 (this workspace's current state)
  new-version.md     # the longer-form architecture/roadmap doc phases.md was distilled from
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

Then choose a category and type guesses by answer name or id. Two dev-only flags help with testing and content debugging:

```bash
cargo run -p deduced-cli -- --seed 12345               # force/reproduce a specific round
cargo run -p deduced-cli -- --seed 12345 --reveal-answer cars   # print the target and exit, no interactive loop
```

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

Production Daily challenge server:

```bash
docker compose up -d postgres   # first time only, starts local Postgres on :5432
cargo run -p deduced-server     # runs migrations automatically, listens on :4000
```

Then, from the workspace root:

```bash
curl http://127.0.0.1:4000/health
curl http://127.0.0.1:4000/daily/current
curl -X POST http://127.0.0.1:4000/daily/submit \
  -H "Content-Type: application/json" \
  -d '{"challenge_id":"<from /daily/current>","player_id":"<any local uuid>","guesses":["<answer id>", "..."],"elapsed_ms":12000}'
curl "http://127.0.0.1:4000/daily/leaderboard?challenge_id=<from /daily/current>"
```

Unlike `deduced-web`'s prototype API, this server never trusts a client's claimed win/score — `/daily/submit` replays the guess sequence through `deduced-core` itself and returns the authoritative result. Set `DATABASE_URL` to point elsewhere; it defaults to the `docker-compose.yml` Postgres instance.

Cloud profile sync (guest identity, best-effort — Solo never blocks on this):

```bash
curl -X POST http://127.0.0.1:4000/profile/sync \
  -H "Content-Type: application/json" \
  -d '{"player_id":"<profile.player_id from save/profile.json>","updated_at":<unix seconds>,"profile":{...}}'
```

Last-write-wins on `updated_at`: an older submission gets the server's copy back (`accepted:false`) instead of overwriting it.

Friend Versus (private lobby) and Quick Match (matchmaking queue), both same-target-hidden-independently, first-correct-guess-wins, server-validated:

```bash
# Friend Versus: one player creates, shares the join_code, the other joins.
curl -X POST http://127.0.0.1:4000/matches -d '{"player_id":"alice"}'
curl -X POST http://127.0.0.1:4000/matches/join -d '{"join_code":"<code>","player_id":"bob"}'

# Quick Match: both call queue; the second call pairs immediately, the first
# picks up the match_id on its next status poll.
curl -X POST http://127.0.0.1:4000/matchmaking/queue -d '{"player_id":"alice"}'
curl -X POST http://127.0.0.1:4000/matchmaking/queue -d '{"player_id":"bob"}'
curl "http://127.0.0.1:4000/matchmaking/status?player_id=alice"
```

Both players then connect to `ws://127.0.0.1:4000/matches/<match_id>/ws?player_id=<id>`, send `{"type":"Ready"}`, and once both are ready the server broadcasts `MatchStarted` (category/seed/content_version — each client reconstructs the round locally) and accepts `{"type":"Guess","answer_id":"..."}` per attempt. Opponent guesses are never revealed, only `OpponentProgress`/`OpponentSolved`. A dropped connection has 15s to reconnect (same match id + player id) before the match is forfeited to the opponent. Finished matches are recorded — `GET /matches/history?player_id=<id>`.

## Development Checks

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p deduced-cli
cargo run -p deduced-game
cargo run -p deduced-web
cargo run -p deduced-server   # needs `docker compose up -d postgres` first
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
- a `deduced-gameplay` session layer shared by every client
- `deduced-protocol`: shared, `deduced-core`-independent wire DTOs for client/server communication
- local save/profile/stats via `deduced-save`
- a production Daily challenge backend (`deduced-server` + Postgres) that authoritatively validates results
- best-effort cloud profile sync with a guest-first local identity
- Friend Versus and Quick Match: real-time WebSocket 1v1 (same hidden target, first correct guess wins), server-validated, with reconnect handling and match history

See [docs/phases.md](docs/phases.md) for the full staged plan (the Bevy client was built ahead of its Phase 4 slot by request). Not yet built: real answer images (Phase 5, blocked on an art/licensing decision), ranked matchmaking, and any store/monetization backend (intentionally deferred per the docs until engagement is proven).
