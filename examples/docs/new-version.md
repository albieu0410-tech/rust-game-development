# DEDUCED — Current State, Target Architecture & Full Development Roadmap

> Status: Pre-production / playable prototype
> Project: `rust-game-development`
> Game: DEDUCED
> Primary language: Rust
> Architecture goal: Offline-first, deterministic, reusable game engine with optional online services

---

# 1. Purpose of This Document

This document describes:

* the current state of the DEDUCED codebase;
* what parts are prototypes versus production-ready foundations;
* architectural problems that should be fixed before expanding the game;
* the proposed production architecture;
* how the client should be structured;
* how Solo, Daily, and Versus should work;
* how the backend should be structured;
* how offline play and synchronization should work;
* how content and assets should be managed;
* how local saves and player progression should work;
* how matchmaking and multiplayer should work;
* what order everything should be developed in;
* what should explicitly **not** be built yet.

The main objective is to move DEDUCED from:

```text
working prototypes
```

to:

```text
production game
    +
offline client
    +
online services
    +
multiplayer platform
```

without throwing away the reusable Rust work that already exists.

---

# 2. What DEDUCED Is

DEDUCED is an offline-first deduction game.

The player attempts to identify a hidden answer from a category.

Examples:

```text
Cars
Countries
Companies
Football clubs
Animals
Movies
Video games
Famous people
Technology
Brands
```

A player submits guesses.

Each incorrect guess reveals information about the hidden answer.

For example:

```text
Guess:

BMW X3

Result:

Country: Germany       ✓
Power: 184 HP          ↑
Vehicle Type: SUV      ✓
Year: 2022             ↓
Brand: BMW             ✓
```

The player uses these clues to progressively narrow down the correct answer.

The core gameplay loop is:

```text
Choose category
       ↓
Generate hidden answer
       ↓
Make guess
       ↓
Compare guess to target
       ↓
Reveal clues
       ↓
Update known information
       ↓
Guess again
       ↓
Win or lose
       ↓
Calculate score
```

The important architectural principle is:

> Game rules must remain independent from the UI, operating system, networking, database, and platform.

---

# 3. Current Repository State

The repository currently contains the following main architecture:

```text
rust-game-development/
│
├── crates/
│   ├── deduced-core/
│   ├── deduced-content/
│   ├── deduced-save/
│   └── deduced-bot/
│
├── apps/
│   ├── deduced-cli/
│   ├── deduced-game/
│   └── deduced-web/
│
├── content/
│   ├── categories/
│   └── answers/
│
├── docs/
│
├── examples/
│
└── tests/
```

The existing architecture already establishes an important separation between:

```text
game rules
content
save data
bots
clients
```

This separation should be preserved.

---

# 4. Current Shared Rust Crates

## 4.1 `deduced-core`

`deduced-core` contains the pure game logic.

It currently owns concepts such as:

```text
Answer
Attribute
AttributeValue
CategoryDefinition
Comparison
Round
GuessResult
Score
```

Modules currently include concepts such as:

```text
answer
attribute
category
comparison
game
guess
round
scoring
```

This crate is the strongest architectural part of the current project.

It should remain independent from:

```text
Bevy
Axum
SQLite
PostgreSQL
HTTP
WebSockets
Android
iOS
Steam
Accounts
Advertising
Cloud services
```

`deduced-core` should only understand things like:

```text
answers
categories
attributes
guesses
comparisons
rounds
attempts
wins
losses
scores
```

This rule should remain permanent.

---

# 5. `deduced-content`

`deduced-content` currently loads category and answer definitions from JSON files.

The current content structure includes:

```text
content/
│
├── categories/
│   ├── cars.json
│   ├── companies.json
│   └── countries.json
│
└── answers/
    ├── cars.json
    ├── companies.json
    └── countries.json
```

This is already the correct general direction.

Categories and answers should remain data-driven instead of hardcoded into game screens.

Long term, this crate should be responsible for:

```text
loading content
validating content
checking required attributes
content versioning
asset references
category metadata
answer metadata
```

---

# 6. `deduced-save`

`deduced-save` currently contains the beginnings of:

```text
Profile
Stats
SaveStorage
```

This will eventually become the foundation for offline-first player data.

It should remain independent from any particular persistence implementation.

For example:

```text
deduced-save
      │
      ├── filesystem implementation
      ├── SQLite implementation
      ├── IndexedDB implementation
      └── mobile implementation
```

The save model should describe the data.

Platform-specific code should determine where the data is stored.

---

# 7. `deduced-bot`

`deduced-bot` exists for future computer opponents.

Eventually this crate can support policies such as:

```text
Random bot
Easy bot
Normal bot
Hard bot
Expert deduction bot
```

This is useful for:

```text
offline Versus
practice modes
testing
simulated matchmaking
game balancing
```

Bot implementation should not be prioritized yet.

---

# 8. Current Applications

There are currently three application entry points.

```text
deduced-cli
deduced-game
deduced-web
```

Each currently serves a different prototype purpose.

---

# 9. `deduced-cli`

The CLI is useful because it validates the game rules without UI complexity.

It should remain in the repository permanently.

Its future purpose should become:

```text
developer testing
content testing
debugging
simulation
bot testing
round reproduction
seed testing
```

The CLI does not need to become a consumer product.

It is a developer tool.

---

# 10. `deduced-game`

`deduced-game` is currently the Bevy graphical prototype.

It already has concepts such as:

```text
Menu
Playing
Result
```

and application state transitions.

The current Bevy application uses a phone-like window size and proves that the Rust engine can be connected to a graphical client.

However, the existing graphical application should still be considered a prototype.

It should eventually become:

```text
deduced-client
```

rather than remaining named:

```text
deduced-game
```

because this application will become the actual production game client.

---

# 11. `deduced-web`

`deduced-web` currently performs two different roles:

```text
web UI prototype
+
Axum prototype backend
```

It contains:

```text
Rust Axum server
static HTML
CSS
JavaScript
game API
```

The web prototype is extremely useful as a design specification.

It already demonstrates screens including:

```text
Home
Categories
Game
Daily
Versus
Profile
Store
```

It also demonstrates the newer streamlined game layout:

```text
header
image reveal
known facts
guess history
guess input
result overlay
```

This work should **not be deleted**.

However, `deduced-web` should not become the production architecture in its current form.

Its main future purpose should be:

> UI and interaction prototype/reference for the real client.

---

# 12. Important Flag #1 — The Current Backend Has One Global Session

The current Axum prototype stores a round roughly like:

```rust
pub struct AppState {
    pub content: GameContent,
    pub session: Mutex<Option<RoundSession>>,
}
```

This means the server effectively has one global active round.

Conceptually:

```text
Server
  │
  └── Active Round
```

instead of:

```text
Server
  │
  ├── Player A Round
  ├── Player B Round
  ├── Player C Round
  └── ...
```

If multiple players used this architecture simultaneously, they could overwrite each other's game session.

This is acceptable for:

```text
prototype
localhost testing
single-player demo
```

It is **not acceptable for production**.

### Action

Do not expand the current global-session architecture.

The production backend should use proper identifiers such as:

```text
user_id
session_id
match_id
challenge_id
round_id
```

depending on the mode.

---

# 13. Important Flag #2 — Solo Should Not Depend on the Backend

The current browser prototype performs operations similar to:

```text
POST /api/round
POST /api/guess
```

for ordinary Solo gameplay.

That should not be the production architecture.

DEDUCED is intended to support offline usage.

Therefore Solo gameplay should be:

```text
CLIENT
│
├── bundled content
├── deduced-core
├── game controller
├── local save
└── local statistics
```

No backend is required.

The player should be able to:

```text
launch game
choose category
play multiple rounds
earn local progress
view stats
```

with:

```text
Wi-Fi disabled
mobile data disabled
server unavailable
airplane mode enabled
```

---

# 14. What Actually Requires the Server

The server should only be involved when a feature requires:

```text
shared state
trust
communication
synchronization
remote players
global data
```

Examples:

```text
Daily Challenges
Accounts
Cloud Saves
Leaderboards
Friends
Matchmaking
Online Versus
Purchases
Live Events
Remote Content Updates
```

Solo does not belong on this list.

---

# 15. Important Flag #3 — UI Logic Should Not Become Game Logic

The browser prototype currently computes some information directly in JavaScript.

For example:

```text
known facts
numeric upper/lower bounds
reveal progress
guess-history presentation
```

Some of these are visual concerns.

Some are gameplay/application concerns.

The distinction should become:

```text
deduced-core
    ↓
pure rules

deduced-gameplay
    ↓
game session interpretation

client
    ↓
rendering
```

The UI should not independently invent game meaning.

For example:

```text
Power > 300 HP
Year < 2020
Country = Germany
```

should ideally be represented by Rust data before reaching the renderer.

---

# 16. New Shared Crate — `deduced-gameplay`

A new crate should be introduced:

```text
crates/deduced-gameplay/
```

Its purpose is to sit between pure rules and the actual client.

Architecture:

```text
deduced-core
      ↓
deduced-gameplay
      ↓
deduced-client
```

`deduced-core` knows what a comparison means.

`deduced-gameplay` knows how a playable session is organized.

`deduced-client` knows how it is drawn.

---

# 17. Responsibilities of `deduced-gameplay`

Suggested structure:

```text
crates/deduced-gameplay/
└── src/
    ├── lib.rs
    ├── controller.rs
    ├── state.rs
    ├── known_facts.rs
    ├── reveal.rs
    ├── mode.rs
    └── result.rs
```

It should contain concepts such as:

```text
GameController
GameViewState
GameResult
KnownFact
RevealState
GameMode
```

---

# 18. `GameController`

Conceptually:

```rust
pub struct GameController {
    round: Round,
    category: CategoryDefinition,
    guesses: Vec<GuessResult>,
}
```

Possible interface:

```rust
impl GameController {
    pub fn new_solo(...) -> Result<Self, GameError>;

    pub fn submit_guess(
        &mut self,
        answer: &Answer,
    ) -> Result<GuessResult, GameError>;

    pub fn state(&self) -> GameViewState;

    pub fn result(&self) -> Option<GameResult>;
}
```

The exact API can evolve.

The important part is that a client does not need to manually coordinate every internal `Round` concern.

---

# 19. `GameViewState`

The graphical client should primarily render something similar to:

```rust
pub struct GameViewState {
    pub category: CategorySummary,

    pub attempts_used: usize,
    pub max_attempts: usize,

    pub reveal: RevealState,

    pub guesses: Vec<GuessView>,

    pub known_facts: Vec<KnownFact>,

    pub status: GameStatus,
}
```

This allows multiple clients to potentially reuse the same game session representation.

For example:

```text
Bevy client
WebAssembly client
CLI
future mobile shell
```

---

# 20. Known Facts

Known facts should be derived from previous clues.

Example guesses:

```text
Guess 1:
Horsepower: 200 ↑

Guess 2:
Horsepower: 350 ↓
```

The game may infer:

```text
Horsepower > 200
Horsepower < 350
```

Result:

```text
200 < Horsepower < 350
```

Exact matches can produce:

```text
Country = Germany
Body Type = SUV
Fuel = Petrol
```

The UI should render these facts.

It should not have to derive the logic itself.

---

# 21. Reveal State

Create a reusable concept such as:

```rust
pub struct RevealState {
    pub level: u8,
    pub max_level: u8,
}
```

The renderer determines how a reveal level looks.

For example:

```text
0 → heavily hidden
1 → 20% revealed
2 → 35%
3 → 50%
4 → 70%
5 → 85%
6 → completely revealed
```

The actual percentages can be balanced later.

---

# 22. Do Not Store Six Different Reveal Images Unless Necessary

Initially, use:

```text
one main answer image
+
deterministic visual mask
```

instead of:

```text
image_1.webp
image_2.webp
image_3.webp
image_4.webp
image_5.webp
image_6.webp
```

This makes content creation much easier.

A deterministic reveal mask can:

```text
hide tiles
blur regions
pixelate blocks
show silhouettes
remove masking progressively
```

This can become one of the game's main visual signatures.

---

# 23. New Shared Crate — `deduced-protocol`

Online features should not define JSON structures independently inside the server and client.

Introduce:

```text
crates/deduced-protocol/
```

This crate contains shared network message definitions.

For example:

```text
authentication DTOs
profile DTOs
daily challenge DTOs
match DTOs
WebSocket messages
error structures
content manifest structures
```

Architecture:

```text
              deduced-protocol
                ▲           ▲
                │           │
       deduced-client   deduced-server
```

Both sides share the exact message schema.

---

# 24. Target Workspace Architecture

The repository should evolve toward:

```text
rust-game-development/
│
├── crates/
│   │
│   ├── deduced-core/
│   │
│   ├── deduced-gameplay/
│   │
│   ├── deduced-content/
│   │
│   ├── deduced-save/
│   │
│   ├── deduced-bot/
│   │
│   └── deduced-protocol/
│
├── apps/
│   │
│   ├── deduced-cli/
│   │
│   ├── deduced-client/
│   │
│   ├── deduced-server/
│   │
│   └── deduced-web/
│       └── prototype/reference
│
├── content/
│
├── assets/
│
├── docs/
│
├── tools/
│
└── tests/
```

Later, tooling could include:

```text
tools/
├── content-editor/
├── content-validator/
├── image-preprocessor/
└── simulation-runner/
```

---

# 25. Production Client

The production graphical application should eventually be called:

```text
deduced-client
```

rather than:

```text
deduced-game
```

It will contain:

```text
UI
game screens
animations
input
audio
offline storage
networking
platform integrations
```

The client depends on:

```text
deduced-core
deduced-gameplay
deduced-content
deduced-save
deduced-protocol
```

---

# 26. Proposed Client Structure

```text
apps/deduced-client/
└── src/
    │
    ├── main.rs
    ├── app.rs
    │
    ├── screens/
    │   ├── mod.rs
    │   ├── splash.rs
    │   ├── onboarding.rs
    │   ├── home.rs
    │   ├── categories.rs
    │   ├── game.rs
    │   ├── result.rs
    │   ├── daily.rs
    │   ├── versus.rs
    │   ├── matchmaking.rs
    │   ├── lobby.rs
    │   ├── profile.rs
    │   ├── stats.rs
    │   ├── store.rs
    │   ├── settings.rs
    │   └── offline.rs
    │
    ├── components/
    │   ├── mod.rs
    │   ├── buttons.rs
    │   ├── topbar.rs
    │   ├── bottom_nav.rs
    │   ├── category_card.rs
    │   ├── clue_chip.rs
    │   ├── guess_card.rs
    │   ├── image_reveal.rs
    │   ├── progress.rs
    │   ├── avatar.rs
    │   ├── modal.rs
    │   └── toast.rs
    │
    ├── game/
    │   ├── mod.rs
    │   ├── solo.rs
    │   ├── daily.rs
    │   └── versus.rs
    │
    ├── networking/
    │   ├── mod.rs
    │   ├── client.rs
    │   ├── auth.rs
    │   ├── websocket.rs
    │   └── sync.rs
    │
    ├── storage/
    │   ├── mod.rs
    │   ├── profile.rs
    │   ├── settings.rs
    │   └── cache.rs
    │
    └── theme/
        ├── mod.rs
        ├── colors.rs
        ├── typography.rs
        ├── spacing.rs
        └── dimensions.rs
```

This structure does not need to be created all at once.

It represents the desired final organization.

---

# 27. Client Application States

Use a real state machine.

Example:

```rust
pub enum AppScreen {
    Splash,
    Onboarding,

    Home,
    Categories,

    SoloGame,
    DailyGame,

    VersusMenu,
    Matchmaking,
    Lobby,
    VersusGame,

    Result,

    Profile,
    Stats,
    Store,
    Settings,
}
```

The application controls navigation through state transitions.

Avoid arbitrary screen manipulation such as:

```text
hide every screen
find DOM element
add "active" class
```

which is suitable for the current HTML prototype but not the production client.

---

# 28. Screen — Splash

Purpose:

```text
initialize application
```

Visual:

```text
┌────────────────────────────┐
│                            │
│                            │
│             D              │
│          DEDUCED           │
│                            │
│       Loading...           │
│                            │
└────────────────────────────┘
```

Responsibilities:

```text
load local save
load settings
load bundled content
validate bundled content
restore player identity
detect connectivity
initialize audio
initialize cached remote metadata
```

Possible transitions:

```text
new user
    ↓
Onboarding

existing user
    ↓
Home
```

---

# 29. Screen — Onboarding

Onboarding should be short.

Do not create a six-screen tutorial before the player can play.

Possible onboarding:

```text
Screen 1
DEDUCED
Every wrong guess makes you smarter.

Screen 2
Make a guess.
We'll tell you how close you are.

Screen 3
Use the clues.
Deduce the hidden answer.

[START PLAYING]
```

Account creation should not be required.

---

# 30. Screen — Home

The existing web prototype provides a strong foundation.

Recommended structure:

```text
┌─────────────────────────────┐
│ Player               DEDUCED│
│ Level 12                    │
│                             │
│              D              │
│           DEDUCED           │
│                             │
│ Every wrong guess makes     │
│ you smarter.                │
│                             │
│ ┌─────────────────────────┐ │
│ │ SOLO                  › │ │
│ │ Choose a category       │ │
│ └─────────────────────────┘ │
│                             │
│ ┌─────────────────────────┐ │
│ │ DAILY                 › │ │
│ │ Today's shared puzzle   │ │
│ └─────────────────────────┘ │
│                             │
│ ┌─────────────────────────┐ │
│ │ VERSUS                › │ │
│ │ Challenge players       │ │
│ └─────────────────────────┘ │
│                             │
│ Home  Daily  VS  Profile    │
└─────────────────────────────┘
```

Solo must remain available offline.

---

# 31. Screen — Category Selection

The client should obtain Solo categories from locally bundled content.

Do not call the backend merely to list categories.

Conceptually:

```rust
content.categories()
```

instead of:

```text
GET /api/categories
```

Example screen:

```text
WHAT DO YOU KNOW?

┌──────────────┐ ┌──────────────┐
│      🚗      │ │      🌍      │
│     CARS     │ │  COUNTRIES   │
│ 150 answers  │ │ 195 answers  │
└──────────────┘ └──────────────┘

┌──────────────┐ ┌──────────────┐
│      🏢      │ │      ⚽      │
│  COMPANIES   │ │   FOOTBALL   │
│ 300 answers  │ │ 250 answers  │
└──────────────┘ └──────────────┘
```

Category metadata should eventually include:

```text
id
name
description
icon
theme
attempt count
answer count
unlock state
difficulty
asset references
```

---

# 32. Screen — Game

This is the most important screen.

The improved browser prototype already establishes the right general interaction model.

The user should **not** need to:

```text
guess
tap Next
read clue
tap Next
guess again
```

The complete gameplay loop should exist on one screen.

Structure:

```text
HEADER

IMAGE / REVEAL AREA

REVEAL PROGRESS

KNOWN FACTS

GUESS HISTORY

GUESS COMPOSER
```

---

# 33. Example Game Layout

```text
┌──────────────────────────────┐
│ ‹      🚗 CARS          ♥ 4 │
├──────────────────────────────┤
│                              │
│       HIDDEN IMAGE           │
│                              │
│        ██░█░██░█             │
│        ░█░██░░██             │
│                              │
│ ████████░░░░           3 / 6 │
│                              │
│ KNOWN FACTS                  │
│ [Germany ✓] [SUV ✓]          │
│ [Power > 184 ↑]              │
│                              │
│ YOUR DEDUCTIONS        2 / 6 │
│                              │
│ ┌──────────────────────────┐ │
│ │ BMW X3                   │ │
│ │ Country Germany      ✓   │ │
│ │ Power 184 HP         ↑   │ │
│ │ Type SUV             ✓   │ │
│ └──────────────────────────┘ │
│                              │
│ ┌──────────────────────────┐ │
│ │ Audi Q5                  │ │
│ │ Country Germany      ✓   │ │
│ │ Power 261 HP         ↑   │ │
│ │ Type SUV             ✓   │ │
│ └──────────────────────────┘ │
│                              │
│ 🔎 Guess...          [GUESS]│
└──────────────────────────────┘
```

---

# 34. Game Screen Components

Internally:

```text
GameScreen
│
├── GameHeader
│
├── RevealCard
│
├── RevealProgress
│
├── KnownFactsPanel
│
├── GuessHistory
│   └── GuessCard[]
│
└── GuessComposer
    ├── SearchInput
    ├── Autocomplete
    └── SubmitButton
```

These should be separate reusable UI components.

---

# 35. Autocomplete

Answers within a category should support search.

Desired behavior:

```text
User types:

"bm"

Suggestions:

BMW M3
BMW X3
BMW X5
BMW i8
```

Already-guessed answers should be:

```text
removed
or clearly disabled
```

Matching can eventually include:

```text
case-insensitive search
prefix matching
substring matching
aliases
fuzzy matching
```

Do not over-engineer fuzzy search for the first production milestone.

---

# 36. Guess Submission Flow

Production Solo flow:

```text
User selects answer
      ↓
GameController.submit_guess()
      ↓
deduced-core compares answer
      ↓
GuessResult generated
      ↓
GameController updates known facts
      ↓
Reveal level changes
      ↓
UI animates new clues
      ↓
input becomes immediately available
```

No intermediate screen.

No "next guess" button.

No server call.

---

# 37. Guess Feedback

A wrong guess should feel satisfying rather than punitive.

Possible sequence:

```text
guess submitted
      ↓
small shake / impact
      ↓
"Not quite"
      ↓
new clue chips animate in
      ↓
image reveals slightly more
      ↓
known facts update
      ↓
input becomes active
```

The entire sequence should be quick.

The user should remain in flow.

---

# 38. Comparison Visual Language

Current comparisons include:

```text
Match
Higher
Lower
Different
Partial
```

A consistent visual language should exist.

Example:

```text
Match       ✓
Higher      ↑
Lower       ↓
Different   ×
Partial     ~
```

Potential semantic colors:

```text
Match       green
Higher      warm/orange
Lower       blue
Different   red
Partial     yellow
```

The exact palette can evolve.

Icons should still communicate meaning without relying entirely on color.

---

# 39. Screen — Result Overlay

The current overlay interaction should be retained.

Example:

```text
        ✓ SOLVED

        DEDUCED!

   ┌─────────────────┐
   │                 │
   │    BMW M3       │
   │                 │
   └─────────────────┘

 Attempts     Time      Score
    4/6       1:34       840

           +120 XP

      [ NEXT ROUND ]

     [ SHARE RESULT ]

       Change Category
```

For a loss:

```text
ANSWER REVEALED
```

instead of:

```text
DEDUCED!
```

The target answer should then be completely revealed.

---

# 40. Solo Game Mode

Solo is the first production feature.

Solo flow:

```text
Home
 ↓
Solo
 ↓
Categories
 ↓
Select category
 ↓
Generate local seed
 ↓
Create local Round
 ↓
Play
 ↓
Result
 ↓
Update local stats
 ↓
Next Round
```

Network status should be irrelevant.

---

# 41. Deterministic Rounds

One of the strongest parts of the existing architecture is deterministic round reconstruction.

A round should be reconstructable from:

```text
category
seed
content_version
```

Conceptually:

```text
(category, seed, content_version)
              ↓
        exact same answer
```

This property is essential for:

```text
Daily Challenges
Friend Challenges
Online Versus
Replays
Server validation
Debugging
Bug reproduction
```

Do not lose this property.

---

# 42. Daily Challenge

Daily Challenges should use deterministic rounds.

The backend generates or stores:

```json
{
  "challenge_id": "daily-2026-08-12-cars",
  "category": "cars",
  "seed": 891237891237,
  "content_version": "2026.08.1"
}
```

The client receives those parameters.

The client constructs the round locally.

---

# 43. Daily Flow

```text
Client
  │
  │ GET current daily
  ▼
Server
  │
  │ category + seed + content version
  ▼
Client
  │
  ├── reconstruct round
  ├── player guesses
  └── complete locally
  │
  │ submit result/replay
  ▼
Server
  │
  ├── reconstruct same round
  ├── replay guesses
  ├── validate score
  └── store result
```

This means the server can validate gameplay without manually running the whole game session in real time.

---

# 44. Daily Submission

Example:

```json
{
  "challenge_id": "daily-2026-08-12-cars",
  "guesses": [
    "audi-q5",
    "bmw-x3",
    "bmw-m3"
  ],
  "elapsed_ms": 44723
}
```

The server can replay the sequence.

The server should calculate authoritative:

```text
result
attempt count
score
```

using the shared Rust engine.

---

# 45. Daily Offline Behavior

Daily is an online-originated challenge.

However, once today's challenge data is downloaded, the player should ideally be able to complete it offline.

Example:

```text
morning:
client downloads daily challenge

afternoon:
player goes offline

player completes challenge

result stored locally

internet returns

result automatically submitted
```

This fits the offline-first philosophy.

---

# 46. Versus

Do not initially build many multiplayer variants.

Start with:

```text
Friend Match
Ranked / Quick Match
```

Friend Match should come first.

---

# 47. Friend Match Flow

```text
Player A
   │
   │ Create Match
   ▼
Server
   │
   └── Join Code: H7K9Q2

Player B
   │
   │ Join H7K9Q2
   ▼
Server
```

Lobby:

```text
PRIVATE MATCH

CODE

H7K9Q2

Sebastian        READY
Player 2         READY

Category:
Random

Rounds:
5

[ START MATCH ]
```

---

# 48. Recommended Versus Mechanic

Initial multiplayer mechanic:

> Both players receive the same target and category. They independently deduce the answer. The first player to solve it wins the round.

Conceptually:

```text
                    MATCH
                      │
             seed/category/version
                      │
           ┌──────────┴──────────┐
           ▼                     ▼
       Player A              Player B
           │                     │
     local engine            local engine
           │                     │
     own guesses             own guesses
           │                     │
           └────── server ───────┘
```

---

# 49. Do Not Reveal Opponent Guesses

By default, Player A should not see:

```text
Player B guessed BMW X5
```

because that could leak useful deduction information.

Instead show:

```text
Opponent

● ● ● ○ ○ ○

3 guesses used
```

or:

```text
Opponent is on guess 4
```

When the opponent solves:

```text
OPPONENT SOLVED
```

This creates pressure without revealing their strategy.

---

# 50. Possible Multiplayer Round

```text
Round 2 / 5

YOU
Guesses: 2

OPPONENT
Guesses: 3

Hidden target:
same for both players
```

Win condition:

```text
first correct answer
```

Potential tiebreakers later:

```text
fewest guesses
fastest time
highest score
```

Do not overcomplicate v1.

---

# 51. Multiplayer Transport

Use:

```text
HTTP
```

for ordinary API operations.

Use:

```text
WebSocket
```

for live match state.

Examples of HTTP:

```text
create lobby
join lobby
view match history
profile
daily challenge
leaderboards
```

Examples of WebSocket:

```text
player ready
match started
guess submitted
opponent progress
player solved
round finished
match finished
disconnect
reconnect
```

---

# 52. Shared WebSocket Messages

Inside `deduced-protocol`:

```rust
pub enum ClientMessage {
    Ready,

    Guess {
        answer_id: String,
    },

    Leave,
}
```

Server events might look conceptually like:

```rust
pub enum ServerMessage {
    MatchStarted {
        round_id: String,
        category_id: String,
        seed: u64,
        content_version: String,
    },

    OpponentProgress {
        attempts_used: u8,
    },

    GuessAccepted {
        attempts_used: u8,
    },

    OpponentSolved {
        attempts_used: u8,
        elapsed_ms: u64,
    },

    RoundFinished {
        winner_id: String,
    },

    MatchFinished {
        winner_id: String,
    },
}
```

Exact structures can evolve.

---

# 53. Important Multiplayer Security Rule

The client may run the same game engine locally for responsiveness.

However, the server must validate competitive actions.

The client should not be trusted to report:

```text
I won
My score is 5000
I guessed correctly
I only used two attempts
```

The server should reconstruct and verify those results.

---

# 54. Production Backend

Create:

```text
apps/deduced-server/
```

The current Axum application can provide useful code and patterns, but should not simply be renamed and expanded without restructuring.

---

# 55. Proposed Server Structure

```text
apps/deduced-server/
└── src/
    │
    ├── main.rs
    ├── config.rs
    ├── state.rs
    ├── error.rs
    │
    ├── api/
    │   ├── mod.rs
    │   ├── health.rs
    │   ├── auth.rs
    │   ├── profile.rs
    │   ├── content.rs
    │   ├── daily.rs
    │   ├── leaderboard.rs
    │   ├── friends.rs
    │   └── matches.rs
    │
    ├── multiplayer/
    │   ├── mod.rs
    │   ├── lobby.rs
    │   ├── matchmaking.rs
    │   ├── match_actor.rs
    │   └── websocket.rs
    │
    ├── services/
    │   ├── mod.rs
    │   ├── auth.rs
    │   ├── player.rs
    │   ├── daily.rs
    │   ├── matchmaking.rs
    │   ├── scoring.rs
    │   └── content.rs
    │
    └── db/
        ├── mod.rs
        ├── users.rs
        ├── profiles.rs
        ├── stats.rs
        ├── matches.rs
        └── challenges.rs
```

---

# 56. Backend Technology

Recommended initial stack:

```text
Rust
Axum
Tokio
Serde
sqlx
PostgreSQL
```

Redis may be added later.

Do not introduce Redis just because multiplayer exists.

Redis becomes useful when:

```text
multiple server instances
distributed matchmaking
distributed presence
temporary match coordination
pub/sub
high traffic
```

For an MVP, one Axum server can keep live match coordination in memory while permanent results live in PostgreSQL.

---

# 57. Database

Use PostgreSQL.

Initial database concepts:

```text
users
profiles
player_stats
category_stats

daily_challenges
daily_attempts

matches
match_players
match_rounds

friendships

content_versions
```

Store tables should be added only when a real store exists.

---

# 58. User Identity

Do not require account registration before the player can play.

Recommended:

```text
guest-first
```

First launch:

```text
create local player UUID
```

Example:

```text
player_id:
5b20ba9d-....
```

The user can immediately play Solo.

An online guest identity can later be created when needed.

---

# 59. Account Upgrade

Later:

```text
Guest
   ↓
Sign in / Create Account
   ↓
Guest data linked to account
```

Benefits:

```text
cloud save
friends
cross-device progression
leaderboards
match history
purchases
```

This is much better for a casual game than forcing registration at launch.

---

# 60. Authentication

Do not build a giant authentication platform initially.

Potential progression:

```text
Phase 1:
local identity only

Phase 2:
guest server identity

Phase 3:
email / social account linking
```

Authentication implementation should only start when Daily, online syncing, or Versus requires it.

---

# 61. Local Player Profile

Expand the save model toward something conceptually like:

```rust
pub struct LocalProfile {
    pub player_id: PlayerId,
    pub display_name: String,

    pub xp: u64,
    pub level: u32,

    pub stats: Stats,

    pub settings: Settings,

    pub unlocks: Unlocks,

    pub last_sync_at: Option<Timestamp>,
}
```

Do not necessarily implement every field immediately.

---

# 62. Local Stats

Useful initial stats:

```text
rounds played
rounds won
rounds lost
win percentage

current streak
best streak

average attempts
best score
total score

per-category rounds
per-category wins
per-category best score
```

Stats should update locally immediately.

Cloud synchronization can happen later.

---

# 63. Offline-First Save Model

Architecture:

```text
GAME
 ↓
LOCAL SAVE
 ↓
player keeps playing
 ↓
internet available?
 │
 ├── no → continue
 │
 └── yes
       ↓
     SYNC
       ↓
     SERVER
```

A temporary server outage must not stop Solo gameplay.

---

# 64. Event-Based Sync Direction

Long-term, prefer syncing meaningful events instead of blindly replacing the entire profile.

Examples:

```text
RoundCompleted
DailyCompleted
AchievementUnlocked
CosmeticUnlocked
SettingChanged
```

This avoids difficult merge situations.

For an early MVP, simpler profile synchronization is acceptable.

The architecture should simply avoid making server connectivity mandatory.

---

# 65. Content Architecture

The current JSON content approach should be expanded.

Suggested future structure:

```text
content/
│
├── manifest.json
│
├── categories/
│   ├── cars.json
│   ├── countries.json
│   ├── companies.json
│   ├── football.json
│   ├── animals.json
│   ├── movies.json
│   └── games.json
│
└── answers/
    ├── cars.json
    ├── countries.json
    ├── companies.json
    ├── football.json
    ├── animals.json
    ├── movies.json
    └── games.json
```

---

# 66. Content Manifest

Example:

```json
{
  "version": "2026.08.1",
  "categories": [
    "cars",
    "countries",
    "companies"
  ]
}
```

Eventually:

```json
{
  "version": "2026.08.1",
  "minimum_client_version": "0.2.0",
  "categories": [
    {
      "id": "cars",
      "version": 4
    },
    {
      "id": "countries",
      "version": 2
    }
  ]
}
```

---

# 67. Bundled Content

The game executable/application package should always contain a base content set.

For example:

```text
Cars
Countries
Companies
```

This ensures that installing DEDUCED immediately gives the player something to play offline.

---

# 68. Downloadable Content Packs

Later the server can publish newer content packs.

Example:

```text
Client version
     │
     ├── bundled content v1
     │
     └── downloaded content v3
```

The client should keep content version metadata so deterministic challenges can specify the correct content version.

---

# 69. Answer Assets

Answer files should eventually reference images.

Example:

```json
{
  "id": "bmw-m3",
  "name": "BMW M3",
  "category": "cars",

  "assets": {
    "main": "cars/bmw-m3/main.webp"
  }
}
```

Possible future assets:

```json
{
  "assets": {
    "main": "cars/bmw-m3/main.webp",
    "thumbnail": "cars/bmw-m3/thumb.webp",
    "silhouette": "cars/bmw-m3/silhouette.webp"
  }
}
```

Do not introduce more asset variants until they are actually needed.

---

# 70. Image Copyright Must Be Considered

Before shipping a large commercial answer database, image licensing needs to be treated seriously.

Possible strategies include:

```text
licensed images
public-domain images
Creative Commons-compatible images
original renders
custom illustrations
user-created graphics
brand-permitted assets
```

Do not assume random Google Images can be shipped with the game.

This is a product/legal concern, not only a coding concern.

---

# 71. Content Quality

The game will only be as good as its answer database.

Every category needs:

```text
consistent attributes
reasonable difficulty
few ambiguous answers
clean images
correct metadata
balanced clue usefulness
```

A category with 500 low-quality answers is worse than one with 80 carefully curated answers.

---

# 72. Content Validation Tooling

Eventually build:

```text
tools/content-validator
```

Checks could include:

```text
duplicate IDs
duplicate names
missing required attributes
broken asset paths
invalid category references
unsupported comparison types
empty values
bad numeric formats
missing aliases
```

This should run in CI.

---

# 73. Development Mode

The current web device preview is useful for design.

Production client development should also support quick testing.

Useful developer features:

```text
force seed
force target answer
skip splash
unlock all categories
show target answer
reset profile
simulate offline
simulate slow network
show content version
show FPS
```

These should be development-only features.

---

# 74. Store

The web prototype contains a Store screen.

Do not build the backend for the Store yet.

Store UI can remain a prototype.

Before implementing monetization, establish:

```text
fun gameplay
retention
Daily usage
category progression
multiplayer interest
```

Possible monetization later:

```text
cosmetics
themes
avatars
category packs
ad-free purchase
premium challenge packs
```

Avoid pay-to-win mechanics.

---

# 75. Profile

Initial Profile screen can be local-only.

Example:

```text
PROFILE

DeduceR
Level 12

Total Games     142
Wins            105
Win Rate        74%

Current Streak  8
Best Streak     21

BEST CATEGORIES

Cars            82%
Countries       77%
Companies       66%
```

Online identity can be added later without redesigning the entire screen.

---

# 76. Progression

Progression is useful but should not block gameplay.

Potential systems:

```text
XP
levels
achievements
category mastery
streaks
cosmetics
titles
```

Example XP:

```text
Complete round       +20
Win                  +30
Low-attempt bonus    +0–40
Daily completion     +25
Versus win           +40
```

Balance later.

Do not deeply tune progression during the first Solo client milestone.

---

# 77. Audio

Audio should eventually include:

```text
button tap
guess submit
wrong guess
clue reveal
correct answer
loss
level up
match found
countdown
```

The game should still work comfortably muted.

Audio implementation is not part of the first architectural refactor.

---

# 78. Animations

Likely useful:

```text
screen transitions
category-card interaction
guess shake
clue chip reveal
image reveal
score counting
XP progress
confetti
match-found transition
opponent progress pulse
```

Gameplay speed should remain fast.

Animations should reinforce information, not delay interaction.

---

# 79. Backend API Direction

The production API should eventually resemble:

```text
/api/v1/
│
├── health
│
├── auth/
│   ├── guest
│   ├── login
│   ├── refresh
│   └── logout
│
├── me
├── me/stats
│
├── content/
│   └── manifest
│
├── daily/
│   ├── current
│   ├── submit
│   └── leaderboard
│
├── matches/
│   ├── create
│   ├── join
│   ├── history
│   └── :id
│
├── matchmaking/
│   ├── queue
│   └── leave
│
└── ws
```

Do not build all of these immediately.

They represent a target structure.

---

# 80. APIs That Should Disappear for Solo

The current prototype architecture includes operations conceptually similar to:

```text
POST /api/round
POST /api/guess
```

for local Solo games.

Those should not be required in the production Solo flow.

Equivalent gameplay occurs locally.

Backend round endpoints should only exist where the server must participate in the mode.

---

# 81. Server Is Not the Source of Game-Rule Truth

The following is an important permanent rule:

```text
deduced-core
    =
canonical game rules
```

Both client and server depend on it.

Do not create:

```text
client scoring algorithm
+
different backend scoring algorithm
```

Do not create:

```text
client comparison logic
+
backend comparison logic
```

Instead:

```text
              deduced-core
                 ▲    ▲
                 │    │
            client    server
```

One implementation.

---

# 82. Architecture Rules

## Rule 1

`deduced-core` must never import Bevy.

## Rule 2

`deduced-core` must never import Axum.

## Rule 3

`deduced-core` must never access a database.

## Rule 4

`deduced-core` must never care whether a player is online.

## Rule 5

Solo must remain playable without internet.

## Rule 6

The client must not duplicate game-rule logic.

## Rule 7

The server must validate competitive results.

## Rule 8

Content must remain data-driven.

## Rule 9

Daily and multiplayer rounds should remain deterministic.

## Rule 10

The current web prototype should be treated as a design/reference implementation, not production architecture.

---

# 83. Things We Should Explicitly Not Do Yet

Do **not** begin by building:

```text
full authentication
ranked matchmaking
Redis cluster
microservices
Kubernetes
complex cloud infrastructure
real-money store
battle pass
guilds
chat
season system
push notification platform
huge admin panel
```

These add complexity without proving the game.

---

# 84. Development Order

The game should be built vertically.

A vertical slice means:

```text
player can actually complete a polished experience
```

rather than:

```text
database is 90% done
backend is 80% done
UI is 30% done
nothing is actually fun yet
```

---

# 85. Phase A — Production Solo Foundation

This is the immediate priority.

Tasks:

```text
create deduced-gameplay
move known-fact derivation into Rust
formalize RevealState
formalize GameViewState
formalize GameResult
```

Exit criteria:

```text
A complete playable Solo game can be represented
without Bevy or Axum-specific logic.
```

---

# 86. Phase B — Real Client Shell

Turn:

```text
deduced-game
```

into:

```text
deduced-client
```

Initial application states:

```text
Home
Categories
Playing
Result
```

Do not implement every prototype screen immediately.

---

# 87. Phase C — Home Screen

Build the production version of the current web Home screen.

Requirements:

```text
responsive
keyboard/mouse support
touch-friendly
clear Solo button
Daily placeholder
Versus placeholder
Profile placeholder
```

At this stage only Solo must actually work.

---

# 88. Phase D — Category Screen

Requirements:

```text
load categories locally
render cards from content
show category name
show answer count
show icon
start local game
```

No server.

---

# 89. Phase E — Production Game Screen

Implement:

```text
header
attempts
image card
reveal progress
known facts
guess history
autocomplete
guess button
result overlay
```

Everything should remain on one screen.

No "next clue" interaction.

---

# 90. Phase F — Real Image Reveal

Add real images.

Implement deterministic masking.

Requirements:

```text
answer loads asset
mask determined from reveal state
more image revealed after incorrect guesses
whole image shown at result
```

---

# 91. Phase G — Local Save

Wire `deduced-save`.

Store:

```text
player identity
settings
round stats
streak
category stats
XP if implemented
```

Exit criteria:

```text
close game
reopen game
progress remains
```

---

# 92. Phase H — Solo MVP Complete

At this stage DEDUCED should support:

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

with:

```text
zero internet required
```

This is the first major product milestone.

---

# 93. Phase I — Content Expansion

Only once the Solo loop feels good:

```text
expand Cars
expand Countries
expand Companies
```

Then introduce a fourth category.

Do not add ten categories immediately.

Use the first categories to refine:

```text
difficulty
attribute usefulness
attempt counts
image reveal
scoring
```

---

# 94. Phase J — Daily Challenge

Now create the production backend.

Initial backend responsibilities:

```text
health check
daily definition
daily submission
daily leaderboard
guest identity if needed
```

Daily provides the first reason for the client to talk to the server.

---

# 95. Phase K — Cloud Profile

After Daily:

```text
guest backend identity
profile sync
cross-device account linking
```

Do not build this before local profile behavior is stable.

---

# 96. Phase L — Friend Versus

Build:

```text
create lobby
join via code
ready state
WebSocket connection
same deterministic rounds
round winner
match winner
```

This is the first real-time networking milestone.

---

# 97. Phase M — Matchmaking

Once private games are stable:

```text
Quick Match
queue
match found
rating
ranked results
disconnect handling
reconnection
```

Matchmaking should reuse the same multiplayer engine as private matches.

---

# 98. Phase N — Store / Monetization

Only after retention and player behavior justify it.

Possible systems:

```text
themes
avatars
cosmetics
optional category packs
ad-free upgrade
```

Avoid implementing a giant economy before the game has players.

---

# 99. Immediate Repository Changes

The next code changes should be approximately:

```text
1. create crates/deduced-gameplay

2. add it to Cargo workspace

3. add:
   controller.rs
   state.rs
   known_facts.rs
   reveal.rs
   result.rs

4. move application-level deduction helpers there

5. update deduced-game to consume GameViewState

6. gradually rename/restructure deduced-game
   into deduced-client

7. keep deduced-web intact
   as design prototype
```

---

# 100. First `deduced-gameplay` Target Structure

```text
crates/deduced-gameplay/
│
├── Cargo.toml
│
└── src/
    ├── lib.rs
    ├── controller.rs
    ├── state.rs
    ├── known_facts.rs
    ├── reveal.rs
    └── result.rs
```

Dependencies should likely include:

```text
deduced-core
```

and possibly:

```text
deduced-content
```

only if necessary.

Prefer to pass content into gameplay rather than making gameplay itself responsible for filesystem loading.

---

# 101. Dependency Direction

Good:

```text
deduced-content
      │
      ▼
deduced-client
      │
      ▼
deduced-gameplay
      │
      ▼
deduced-core
```

Also possible:

```text
deduced-client
   ├── deduced-content
   ├── deduced-gameplay
   ├── deduced-save
   └── deduced-protocol
```

Avoid:

```text
deduced-core
      ↓
deduced-client
```

or:

```text
deduced-core
      ↓
deduced-server
```

Core should not depend upward.

---

# 102. Testing Strategy

## `deduced-core`

Test:

```text
round generation
deterministic seeds
comparisons
duplicate guesses
win
loss
scoring
numeric attributes
exact attributes
tags
partial matches
```

## `deduced-gameplay`

Test:

```text
reveal progression
known-fact aggregation
game state
attempt progression
result generation
repeated guesses
game completion
```

## `deduced-content`

Test:

```text
JSON loading
invalid answers
missing attributes
duplicate IDs
unknown categories
```

## Server

Test:

```text
daily validation
match creation
join codes
submission verification
authorization
```

---

# 103. Integration Tests

Important deterministic test:

```text
Given:

category = cars
seed = 12345
content_version = X

Client produces answer A.

Server produces answer A.
```

This should always pass.

Another:

```text
Replay guess sequence:

A
B
C

Client final state == server final state.
```

These tests become extremely important once online modes exist.

---

# 104. CI

Eventually CI should run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Content validation should eventually be included.

For example:

```bash
cargo run -p deduced-content-validator
```

---

# 105. Development Commands

Current style remains useful:

```bash
cargo test --workspace
```

CLI:

```bash
cargo run -p deduced-cli
```

Current Bevy prototype:

```bash
cargo run -p deduced-game
```

Current web prototype:

```bash
cargo run -p deduced-web
```

After restructuring:

```bash
cargo run -p deduced-client
```

and later:

```bash
cargo run -p deduced-server
```

---

# 106. What Happens to `deduced-web`?

Do not delete it.

Change its conceptual status to:

```text
UI PROTOTYPE / DESIGN REFERENCE
```

It is useful for quickly experimenting with:

```text
layouts
colors
spacing
responsive behavior
game screen changes
new screens
```

HTML/CSS is faster for UI experimentation than rebuilding every visual idea directly inside Bevy.

Recommended workflow:

```text
design idea
    ↓
prototype quickly in deduced-web
    ↓
approve interaction
    ↓
implement production version in deduced-client
```

That gives the project a useful design sandbox.

---

# 107. Bevy Decision

For the current direction, continuing with Bevy is reasonable.

DEDUCED is not simply a forms application.

It can benefit from:

```text
animated clues
reveal effects
particles
transitions
timers
responsive game feedback
multiplayer presentation
audio
custom rendering
```

Bevy provides a strong Rust-native foundation for those experiences.

However, Bevy should remain entirely inside the client application.

Never allow Bevy types to leak into shared game logic.

---

# 108. Mobile Considerations

The game design is strongly mobile-oriented.

Primary design target:

```text
portrait phone
```

Example reference size:

```text
390 × 844
```

But layouts should adapt to:

```text
small phones
large phones
tablets
desktop
```

Avoid hardcoding the entire interface to one exact resolution.

---

# 109. Desktop Layout

Desktop does not necessarily need an entirely different game.

A wider layout could become:

```text
┌──────────────────────────────────────────────┐
│                    HEADER                    │
├──────────────────────┬───────────────────────┤
│                      │                       │
│     IMAGE REVEAL     │   KNOWN FACTS         │
│                      │                       │
│                      │   GUESS HISTORY       │
│                      │                       │
│                      │   GUESS INPUT         │
│                      │                       │
└──────────────────────┴───────────────────────┘
```

Mobile can stack those areas.

---

# 110. Important UX Principle

The player should almost always be:

```text
one action away from another guess
```

Avoid unnecessary screens between guesses.

Avoid excessive confirmations.

Avoid slow animations.

Avoid forcing scrolling for critical gameplay information when possible.

---

# 111. Important Performance Principle

Do not prematurely optimize backend infrastructure.

Do optimize:

```text
client startup
image loading
UI responsiveness
content loading
memory footprint
```

because these directly affect players.

---

# 112. Important Product Principle

The core question before building complex infrastructure is:

> Is the deduction loop fun enough that somebody wants to play another round immediately?

Until that is true, prioritize:

```text
game feel
clue quality
image reveal
categories
speed
scoring
results
progression feedback
```

over backend sophistication.

---

# 113. Production Milestones Summary

## Milestone 1 — Production Solo Client

```text
deduced-gameplay
Home
Categories
Game
Result
Known Facts
Autocomplete
Image Reveal
Local Stats
Persistent Save
```

---

## Milestone 2 — Better Content

```text
larger answer sets
real assets
content validation
balance testing
```

---

## Milestone 3 — Daily Challenge

```text
deduced-server
deduced-protocol
daily seed
submission verification
leaderboard
```

---

## Milestone 4 — Account Sync

```text
guest identity
optional account
cloud profile
sync
```

---

## Milestone 5 — Friend Versus

```text
lobbies
join codes
WebSocket
same-target rounds
match result
```

---

## Milestone 6 — Matchmaking

```text
queue
match found
ranking
reconnection
match history
```

---

## Milestone 7 — Monetization / Expansion

```text
cosmetics
new categories
premium packs
events
store
```

only if the game has proven engagement.

---

# 114. Immediate Development Checklist

The next implementation work should be:

* [ ] Add `crates/deduced-gameplay`.
* [ ] Add `deduced-gameplay` to the Cargo workspace.
* [ ] Implement `GameController`.
* [ ] Implement `GameViewState`.
* [ ] Implement `KnownFact`.
* [ ] Move known-fact derivation out of JavaScript.
* [ ] Implement `RevealState`.
* [ ] Implement `GameResult`.
* [ ] Update the graphical client to consume the gameplay layer.
* [ ] Formalize application screen state.
* [ ] Build production Home screen.
* [ ] Build production Category screen.
* [ ] Rebuild the streamlined Game screen.
* [ ] Implement autocomplete.
* [ ] Implement clue-history rendering.
* [ ] Implement result overlay.
* [ ] Add real answer image support.
* [ ] Add deterministic progressive image reveal.
* [ ] Wire in `deduced-save`.
* [ ] Store local stats.
* [ ] Persist local profile.
* [ ] Verify entire Solo flow works offline.
* [ ] Play-test extensively.
* [ ] Only then begin the production backend.

---

# 115. Current Architectural Flags Summary

## FLAG — Global Server Session

Current prototype backend stores one active session globally.

**Fix:** do not use that model in production.

---

## FLAG — Solo Depends on Server

Current browser prototype calls the server for Solo rounds and guesses.

**Fix:** production Solo should run entirely locally.

---

## FLAG — Application Logic in JavaScript UI

Known-fact derivation and similar behavior currently exists inside prototype JavaScript.

**Fix:** move reusable gameplay state derivation into Rust.

---

## FLAG — `deduced-web` Has Two Responsibilities

It currently acts as client and server.

**Fix:** treat it as a prototype and create separate production client/server applications.

---

## FLAG — Backend Too Early

The existing roadmap originally placed networking after local gameplay matured.

That principle remains correct.

**Fix:** finish polished Solo before expanding backend complexity.

---

## FLAG — Authentication Can Easily Become Overbuilt

Accounts are not needed for Solo.

**Fix:** guest-first, optional sign-in later.

---

## FLAG — Image Licensing

A commercial guessing game may rely heavily on recognizable images.

**Fix:** establish an asset licensing strategy before large-scale content production.

---

## FLAG — Content Quality Will Determine Game Quality

Generic or inaccurate answer data will make deduction frustrating.

**Fix:** create validation and curation workflows, not just bigger datasets.

---

# 116. Final Target Architecture

```text
                         DEDUCED
                            │
         ┌──────────────────┴──────────────────┐
         │                                     │
         ▼                                     ▼

   DEDUCED CLIENT                       DEDUCED SERVER

   Bevy / Rust                          Axum / Rust
   UI                                   API
   Screens                              Accounts
   Local Solo                           Daily
   Local Saves                          Leaderboards
   Local Content                        Multiplayer
   Animations                           Matchmaking
   Networking                           Cloud Sync

         │                                     │
         │                                     │
         └───────────────┬─────────────────────┘
                         │
                         ▼

                  deduced-protocol

                         │

              ┌──────────┴───────────┐
              │                      │
              ▼                      ▼

       deduced-gameplay        deduced-content
              │
              ▼
         deduced-core

              +
         deduced-save
              +
         deduced-bot
```

Backend infrastructure eventually becomes:

```text
DEDUCED SERVER
      │
      ├── PostgreSQL
      │
      ├── object storage / CDN
      │
      └── Redis
           only when scaling needs it
```

---

# 117. Final Development Philosophy

DEDUCED should be built in this order:

```text
RULES
  ↓
PLAYABLE SESSION
  ↓
POLISHED CLIENT
  ↓
OFFLINE SAVE
  ↓
CONTENT
  ↓
DAILY
  ↓
ACCOUNTS
  ↓
FRIEND MULTIPLAYER
  ↓
MATCHMAKING
  ↓
MONETIZATION
```

Not:

```text
database
↓
authentication
↓
microservices
↓
matchmaking
↓
store
↓
maybe eventually make the game fun
```

The project already has a solid Rust foundation.

The goal now is not to restart.

The goal is to turn the existing pieces into clear layers:

```text
deduced-core
    =
the rules

deduced-gameplay
    =
the playable session

deduced-content
    =
the game's knowledge

deduced-save
    =
local progression

deduced-client
    =
the product players interact with

deduced-protocol
    =
shared online language

deduced-server
    =
online services
```

The current HTML/CSS/JavaScript application should remain available as the fast visual prototype.

The existing Bevy application becomes the foundation for the production client.

The current Axum server provides useful experimentation but should be replaced by a proper multi-user backend architecture when online features actually become necessary.

The **next major goal is therefore not multiplayer or accounts**.

It is:

> Build one polished, completely offline Solo round from Home → Category → Game → Result that feels like a finished game rather than a prototype.

Once that works well, every online feature can be built around a game that is already worth playing.
