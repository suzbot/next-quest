Guidance for Claude Code when working with code in this repo.

## Project Overview

Next Quest is an RPG-themed task motivator app designed for ADHD brains. It's a quest giver, not a quest list — it tells you one thing to do right now.

**Current phase:** Phase 5A-8 complete. Next: Phase 5B-1 (GitHub readme)

**Vision:** See [VISION.md](VISION.md) for full vision, modes, and phased roadmap.

---

## Quick Commands

Build (debug):
cargo tauri build --debug

Run the built app (user launches manually — don't run `open` or launch it automatically):
./src-tauri/target/debug/next-quest

Tests:
cd src-tauri && cargo test

Dev mode (hot reload, when we add a frontend dev server):
cargo tauri dev

## Architecture

Tauri app: Rust backend + web frontend.

- **Backend (Rust/Tauri):** Data persistence, quest selection logic, system tray, timers, notifications
- **Frontend (Web):** UI layer, 8-16bit RPG aesthetic, quest display, XP/level feedback

### File Structure

- `src-tauri/src/db.rs` — all data logic, migrations, and tests
- `src-tauri/src/commands.rs` — Tauri command wrappers, timer state, tray state, skip state, settings
- `src-tauri/src/tray.rs` — system tray icon, menu, and event handling
- `src-tauri/src/main.rs` — app setup, Encounters polling thread
- `src-tauri/build.rs` — build-time image manifest generation
- `ui/index.html` — main app frontend (HTML/CSS/JS)
- `ui/overlay.html` — Encounters overlay window
- `ui/images/manifest.json` — auto-generated image list (do not edit by hand)
- `ui/text/` — external flavor text pools (editable without code changes)
- `docs/` — requirements, design docs, step specs

### Data Model

See [DATA_MODEL.md](DATA_MODEL.md) for entities, relationships, and quest selector logic.

## Collaboration

### Process for Each Feature/Phase

1. **Requirements Discussion** — Talk through what we're building. Ask questions, surface ambiguities, explore edge cases. No docs yet — just conversation.
2. **Requirements Doc** — Claude drafts, user refines. Lives in `docs/`. This is what we're building and why.
3. **Tech Design Discussion** — Talk through how to build it. Entities, relationships, data flow, trade-offs. Think in vertical slices: each implementation step should deliver something the user can functionally test.
4. **Design Doc** — Claude drafts, user refines. Lives in `docs/`. This is how we're building it. Implementation steps must be vertical slices, not "backend then frontend."
5. **Step Spec** — Small implementation spec for the current slice of work. Scoped to what can be built, tested, and committed in one session.
6. **Implementation** — Code to the step spec. Tests where appropriate.
7. **Human Testing** — User runs the app and verifies.
8. **Documentation** — Update data model, CLAUDE.md, and any other docs to reflect what was built.

Do NOT skip steps or combine them without discussing it first. Do NOT draft docs that introduce decisions we haven't discussed.

### Doc & Commit Discipline

- **Don't commit docs without code.** Design docs, requirements, and step specs should only be committed alongside implementation code, or when the user explicitly asks for a doc-only commit.
- **Design docs must implement the requirements, not reinterpret them.** If a design decision would differ from what was agreed in requirements, flag it BEFORE writing the doc. Don't introduce deviations silently.
- **Actually think about gaps before saying there aren't any.** When asked "do you have questions before drafting," genuinely analyze the requirements for edge cases and uncertainties. Don't default to confidence.
- **Summarize after every design doc.** End with a conversational summary of the approach and implementation steps. Each step should be a vertical slice — something the user can functionally test, not just "wrote backend code."

### Communication Style

- **Functional terms, not code mechanics.** Prose for tradeoffs, not multiple-choice. Recommend with options.
- **The user is a PM** — she'll engage deeply on entities, relationships, and system design, not syntax.
- **Help realize the vision.** If you don't understand the intent, ask for context. Don't ask "are you sure?" — if there are substantive concerns, present trade-offs.
- **Trust user observations as evidence.** Verify where they point, don't reason about why they can't be true.
- **When things go wrong**: Gather evidence first before reasoning about causes. Don't speculate. Surface when stuck.
- **Always use numbered lists.** The user responds by number. Any time you present items, questions, options, or notes, number them.

## Dependencies & Security

- **Minimal dependencies**: Every npm package or Rust crate added to the app must be justified. Prefer fewer, well-known packages over many small ones.
- **Document what we install**: When adding a dependency, log it in this section with what it does and why we need it. The user can't evaluate security claims about unfamiliar packages — the record itself is the guardrail.
- **Prefer standard library**: Use Rust's std and browser-native APIs before reaching for a package.
- **No silent installs**: Never add dependencies as a side effect of another step. Each one gets called out explicitly.

### Installed Dependencies

| Package | Type | What it does | Why we need it |
|---|---|---|---|
| tauri 2 | Rust | App framework — webview, window management, system APIs | Core framework |
| serde 1 | Rust | Serialization/deserialization of data structures | Passing data between Rust and frontend |
| serde_json 1 | Rust | JSON parsing | Data format for Rust ↔ frontend communication |
| tauri-build 2 | Rust (build only) | Compiles Tauri config at build time | Required by Tauri |
| rusqlite 0.31 | Rust | SQLite database access (bundled) | Read/write quest data |
| uuid 1 | Rust | Generate unique IDs (v4) | Quest and completion IDs |
| dirs 6 | Rust | Platform data directory paths | Locate app data folder |
| libc 0.2 | Rust | C standard library bindings | Local timezone conversion for quest due dates |
| serde_json 1 | Rust (build only) | JSON serialization | Build-time image manifest generation |

## Pacing & Breaks

This is an ADHD productivity app being built by someone with ADHD. The development process must practice what the app preaches.

- **Build in stopping points**: After each meaningful milestone (something builds, something runs, something is visible), pause and check in. Don't chain three more steps onto a win.
- **Scope work in small chunks**: Propose work in pieces that can each be completed, tested, and committed independently. If a step has more than ~3 substeps, break it down further.
- **It's always okay to stop**: Any commit point is a valid stopping point. Frame it that way — "this is a good place to pause if you want" — not as a cliffhanger to the next thing.
- **Don't manufacture urgency**: Avoid "while we're at it" and "we should also." Stick to what was discussed.

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
