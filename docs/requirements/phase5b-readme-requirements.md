# Phase 5B-1: README and Mechanics Doc — Requirements

## Step 1: README

### Goals

1. Standard README functions: summary, install/build/launch, basic usage
2. Showcase what is unique and innovative compared to other productivity apps
3. Showcase the AI-human collaboration process and link to collateral documents

### Audiences

1. Prospective employers assessing PM skills and ability to leverage AI
2. Coworkers/peers interested in the build process
3. Friends who may want to use or modify the app

### Tone and Style

- Relaxed professional
- Succinct — all audiences are tech-savvy
- Let the work speak for itself, don't oversell
- Process-oriented on the AI collaboration angle (not just "built with AI" — show how)

### Content Outline

1. **What it is** — One-sentence summary. RPG-themed quest giver for ADHD brains, not another task list.
2. **What makes it different** — Key design decisions:
   - Quest giver, not quest list — tells you one thing to do right now
   - No streaks, no punishment — XP only goes up
   - Smart scoring that surfaces the right quest at the right time based on overdue-ness, importance, time of day, and more
   - Three difficulty lanes so you're never overwhelmed
   - Sagas for multi-step goals, revealed one step at a time
   - Encounters overlay that interrupts hyperfocus with low-effort tasks
   - "Not Today" to dismiss without penalty
3. **Install / Build / Launch** — Brief. Assumes Rust/Cargo/Tauri are understood.
4. **Basic usage** — How the app works at a high level. Tabs, quest giver flow, quest list, sagas.
5. **How it was built** — AI-human collaboration process:
   - Requirements → Design → Step Spec → Implementation → Testing → Documentation
   - Link to `CLAUDE.md` as the collaboration contract
   - Link to `docs/requirements/` and `docs/design/` as process artifacts
   - Link to `docs/mechanics.md` for system depth
   - Link to `VISION.md` for roadmap
6. **Tech stack** — Tauri, Rust, HTML/CSS/JS, SQLite. Minimal dependencies by design.

### Linked Documents

- `CLAUDE.md` — AI collaboration contract
- `VISION.md` — Full vision and roadmap
- `docs/mechanics.md` — Game system details (user-facing reference)
- `docs/requirements/` — Requirements documents (process examples)
- `docs/design/` — Design documents (process examples)

---

## Step 2: Mechanics doc rework

### Purpose

Reframe `docs/mechanics.md` as a user-facing reference for someone who wants to understand how the app works at a deeper level than the README. Not internal developer documentation.

### Changes

1. **Rewrite intro** — Frame as "here's how the app works under the hood" rather than "formulas and tuning values."

2. **Reorganize by user concepts** rather than implementation order:
   - Core concepts: quests, difficulty, cycles, sagas
   - Quest giver: lanes, how quests are selected, scoring
   - XP and progression: earning XP, time modifier, levels, skills/attributes
   - Features: filters, not today, campaigns, category tags

3. **Remove implementation details** that a user doesn't need:
   - Bitmask values for TOD/DOW (replace with plain descriptions of the four time windows and day selection)
   - CSS class names
   - Database column/table names
   - Internal type names (`target_type = 'quest_completions'`)

4. **Add context for orphaned sections** — Skills/attributes mapping needs a brief intro explaining what skills and attributes are before listing the defaults.

5. **Reframe saga slot section** — Currently reads like a changelog of what was built. Rewrite as "how sagas appear and behave on the quest list."

6. **Keep the good stuff** — XP formula with examples, design rationale notes, reference value tables. These are the kind of detail a curious user wants.
