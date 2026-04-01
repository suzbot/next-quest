# Next Quest

An RPG-themed productivity app that gives you the right quest at the right time. You brain dump everything in, and it surfaces the best next thing to do.

Built for people who struggle with task initiation, routine maintenance, and the overwhelm of staring at a long to-do list.

![Next Quest screenshot](docs/Screenshot.png)

## What Makes It Different

**Three things at a time** You have three quests available to you: the next routine task, easy side quests, and bigger undertakings. You always know what's coming without being overwhelmed by it. You choose how to respond and the algorithm adapts accordingly.

**No decision paralysis.** The app picks your next quest in each tier based on what's overdue, what you've marked as important, what time of day it is, and what you've been avoiding. You don't manage a list — you respond to a quest giver.

**Encounters that interrupt hyperfocus.** A timed overlay pops up with a low-effort task to pull you away from the screen.

**No streaks. No punishment.** XP and levels only go up. Miss a day? The app doesn't care. Quests resurface naturally based on their cycle. The best strategy is doing things on time, but there's no penalty for being human.

**Smart scoring.** A multi-factor algorithm considers time past due, importance, time of day, day of week, list position, campaign membership, and skill balance. You brain-dump everything — daily routines, one-off errands, long-deferred projects — and the system raises them at the right time.

**RPG progression.** Quests earn XP that flows to a character that represents whats important to you in terms of personal values (attributes), and directional goals (skills), allowing you to visualize how you are growing as you accomplish tasks. Everything levels up on a Fibonacci curve. Harder and less frequent tasks earn more. Doing things late earns a bit extra — but not enough to make procrastination a strategy.

**Sagas for multi-step goals.** Break big goals into ordered steps. The app reveals one step at a time, so "deep clean the kitchen" becomes "clear the counters" today and "scrub the stove" tomorrow. Sagas can be one-off or recurring.

**Campaigns for self-defined achievements.** Group quests and sagas into campaigns that track completion counts toward a goal you set. "Read 12 books this year," "Complete spring cleaning," or any milestone meaningful to you. When all criteria are met, the campaign completes and records accomplishment on your character.

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

## Usage

**Next Quest tab** — The quest giver. Three lanes show one quest each by difficulty. Accept it, start a timer, skip it, or dismiss it for the day.

**Encounters Overlay** — A small always-on-top window that fires on a configurable interval. Shows a single trivial quest with a quick-complete button. Designed to break hyperfocus cycles without demanding much effort.

**Quest List tab** — All your quests and saga slots in priority order. Add, edit, reorder (drag or shift+arrow), complete, and filter. Search by name, skill, tag, difficulty, or importance.

**Sagas tab** — Create and manage multi-step goals. Add steps, set difficulty and cycle. Sagas appear as slots on the quest list alongside regular quests.

**Character tab** — Your RPG character, attributes, skills, and level progression. Link quests to skills and attributes to direct where XP flows.

**Campaigns tab** — Track progress toward larger goals by combining quest and saga completion criteria.

**Settings** — Configure the Encounters overlay interval and debug scoring display. Reset data.

For more detail, see the [Mechanics reference](docs/mechanics.md).

## Customization

Images and flavor text are loaded from external directories — no code changes needed to personalize the app.

**Images:** Drop `.gif` files into the folders under `ui/images/`. Each subfolder serves a different context: `quest-givers/`, `monsters/`, `victory/`, `defeat/`, and per-lane folders (`lane1/`, `lane2/`, `lane3/`). Rebuild to regenerate the image manifest.

**Flavor text:** Edit the `.txt` files under `ui/text/`. One line per entry — the app picks randomly. Per-lane folders hold quest giver dialog, and top-level files cover encounter lines and general quest giver lines.
