# Architecture

DEDUCED is split around one rule: the deduction engine must be reusable by any interface or transport.

```text
apps/deduced-cli
apps/deduced-game
        |
        v
crates/deduced-core
        ^
        |
crates/deduced-content
crates/deduced-save
crates/deduced-bot
```

## Core

`deduced-core` owns the pure model:

- `Answer`
- `Attribute`
- `AttributeValue`
- `CategoryDefinition`
- `Comparison`
- `Round`
- `GuessResult`
- `Score`

It does not load files, render UI, store saves, talk to servers, or know platform details.

## Content

`deduced-content` loads category definitions and answers from JSON:

```text
content/categories/*.json
content/answers/*.json
```

Validation ensures answers reference known categories and provide all attributes required by their category.

## Clients

`deduced-cli` is the first playable client because it gives the fastest feedback on whether the mechanic is fun.

`deduced-game` is reserved for Bevy after the core loop is proven.

## Future Backends

Future networking should reconstruct rounds from:

```text
category
seed
content_version
```

That keeps daily challenges, friend challenges, replays, and online matches deterministic.
