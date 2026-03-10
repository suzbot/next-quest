Guidance for Claude Code when working with code in this repo.

## Project Overview

Next Quest is an RPG-themed task motivator app designed for ADHD brains. It's a quest giver, not a quest list — it tells you one thing to do right now.

**Current phase:** MVP — "The Quest Giver"

**Vision:** See [VISION.md](VISION.md) for full vision, modes, and phased roadmap.

---

## Quick Commands

Build and run (TBD — will fill in once project is scaffolded)

## Architecture

Tauri app: Rust backend + web frontend.

- **Backend (Rust/Tauri):** Data persistence, quest selection logic, system tray, timers, notifications
- **Frontend (Web):** UI layer, 8-16bit RPG aesthetic, quest display, XP/level feedback

### File Structure

(TBD — will fill in once project is scaffolded)

### Data Model

See [DATA_MODEL.md](DATA_MODEL.md) for entities, relationships, and quest selector logic.

## Collaboration

**Discuss → Design → Tests → Implementation → Human Testing → Documentation**

- **Before writing code**: Discuss approach. Frame the problem first — current state, desired state, confirm alignment before implementing.
- **Communication**: Functional terms, not code mechanics. Prose for tradeoffs, not multiple-choice. Recommend with options. The user is a PM — she'll engage deeply on entities, relationships, and system design, not syntax.
- **Interaction**: The user has a vision — help realize it. If you don't understand the intent, ask for context. Don't ask "are you sure?" — if there are substantive concerns, present trade-offs. Trust user observations as evidence.
- **When things go wrong**: Gather evidence first before reasoning about causes. Don't speculate. Surface when stuck.
- **Quality gates**: User must test before marking complete. Keep docs current.

## Design Principles

- **One quest at a time**: The app picks, the user acts. No list paralysis.
- **No streaks, ever**: XP and levels only go up. No decay, no punishment for absence.
- **RPG framing**: Quests not tasks, grinding not chores, boss fights not "big goals."
- **Don't become the problem**: The app must never become another system to manage. If a feature adds management overhead, cut it.
- **8-16bit aesthetic**: Think 1985 Apple IIGS Bard's Tale. Pixel art, pixel fonts, simple charm.

## Reference Documents

| Document | Purpose |
|---|---|
| [VISION.md](VISION.md) | Full vision, modes, RPG theme, phased roadmap |
| [DATA_MODEL.md](DATA_MODEL.md) | MVP entities, relationships, quest selector logic |
