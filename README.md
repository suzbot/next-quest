# Next Quest

An RPG-themed productivity app that gives you the right quest at the right time. You brain dump everything in, and it surfaces the best next thing to do.

Built for people who struggle with task initiation, routine maintenance, and the overwhelm of staring at a long to-do list.

![Next Quest screenshot](docs/Screenshot.png)

## What Makes It Different

- **Brain dumping.** Easy interface with full keyboard support lets you get everything out of your head and into the app fast.
- **Smart surfacing.** Multi-factor algorithm surfaces tasks as quests by weighing time past due, importance, time of day, list position, user responses, and more.
- **Only see 3 things.** Routine tasks, side quests, and bigger undertakings are each offered one at a time by their own quest giver.
- **No decision paralysis.** Each lane shows one quest for you based on what's overdue, important, and contextually relevant.
- **Encounters.** Timed overlay interrupts hyperfocus with low-effort tasks.
- **No punishment.** XP only goes up. Miss a day? Quests resurface naturally. No failure for breaking a streak.
- **RPG progression.** XP flows to your character, reflecting your actual attainments in personal values (attributes) and directional goals (skills).
- **Sagas.** Multi-step goals revealed one step at a time.
- **Campaigns.** Define achievements that are meaningful to you, track progress across quests and sagas, and commemorate accomplishments on your character.

For more detail, see the [Mechanics reference](docs/mechanics.md).

## How It Was Built

Next Quest is built through an iterative collaboration between a Product Manager and [Claude Code](https://claude.com/claude-code). Each feature follows a structured process:

1. **[Requirements](docs/requirements/)** — Talk through what to build, surface edge cases, explore trade-off, capture what and why in a document
2. **[Tech Design](docs/design/)** — Confer and capture the technical approach and implementation steps
3. **Cyclical implementation and testing** — Code to the spec, with tests, run the app, provide feedback
4. **[Vision and roadmap](VISION.md)** - Updated based on the feedback loop

The collaboration is shaped by [`CLAUDE.md`](CLAUDE.md), which defines the working relationship, communication style, process guardrails, and design principles.

## Build and Run

Requires [Rust](https://www.rust-lang.org/tools/install) and the [Tauri 2 CLI](https://tauri.app/start/).

```bash
# Build (debug)
cargo tauri build --debug

# Run
./src-tauri/target/debug/next-quest

# Run tests
cd src-tauri && cargo test
```

## Customization

Images and flavor text are loaded from external directories — no code changes needed to personalize the app.

**Images:** Drop `.gif` files into the folders under `ui/images/`. Each subfolder serves a different context: `quest-givers/`, `monsters/`, `victory/`, `defeat/`, and per-lane folders (`lane1/`, `lane2/`, `lane3/`). Rebuild to regenerate the image manifest.

**Flavor text:** Edit the `.txt` files under `ui/text/`. One line per entry — the app picks randomly. Per-lane folders hold quest giver dialog, and top-level files cover encounter lines and general quest giver lines.
