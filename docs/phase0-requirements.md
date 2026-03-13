# Phase 0: "The List" — Requirements

## Goal

A basic quest list that lets you seed, view, manage, and complete quests.
This is the foundation the quest giver will later draw from. For now,
you are your own quest giver — the app just holds the list and tracks completion.

## Quest Properties

Every quest has:
- **Title**: Free text name ("Take a shower", "File taxes")
- **Cycle**: Number of days between reminders (null = one-off quest). Default: recurring, 1 day.
- **Sort Order**: Player-defined position in the list. Higher = more prominent / suggested first.

## Views & Interactions

### Quest List View

A single scrollable list showing all active quests, ordered by sort order (highest first).

**For each quest, show:**
- Title
- Cycle (e.g. "Every 3 days" or "One-off")
- Last completed date/time (if ever completed)

**Visual states:**
- **Refreshed / Due**: Emphasized style. The quest's cycle has elapsed since last completion (or it's never been completed). These are the quests ready to be done.
- **Recently completed (recurring)**: De-emphasized style. Cycle hasn't elapsed yet. Still visible, still completable, just visually quieter.
- **Completed (one-off)**: Strikethrough style. Stays in the list as a visible record of accomplishment.

### Actions

- **Add quest**: Create a new quest. Title is required. Cycle defaults to recurring/1 day (user can change to one-off or a different cycle).
- **Edit quest**: Change title, cycle, or sort order of an existing quest.
- **Delete quest**: Remove a quest from the list entirely.
- **Mark done**: Record a completion. Can be done multiple times per day. On completion:
  - A completion record is created with the current timestamp.
  - Recurring quests move to the bottom of the list and shift to de-emphasized style.
  - One-off quests shift to strikethrough style.
- **Re-sequence**: Drag or move quests to change their sort order.

## Behaviors

- **Cycle refresh**: A recurring quest returns to "refreshed/due" style when `last_completed + cycle_days` has passed. No notification or automation yet — just the visual state change.
- **No lockout**: A quest can always be completed regardless of cycle state. Cycle only affects visual emphasis.
- **Multiple completions**: Completing a quest multiple times in a day is allowed. Each completion is recorded. Last Done reflects the most recent.

## Interaction Requirements

- **Keyboard-first**: Every action (add, edit, delete, mark done, resequence) must be achievable with keyboard only. Tab navigation through the list and actions. Keyboard shortcut for reordering (e.g. Alt+Up/Down).
- **Mouse/drag also supported**: Drag-and-drop reordering works too, but keyboard is not an afterthought — it's the primary input.

## Visual Design

- **Bare text for now.** No pixel art, no themed UI. Functional and readable.
- **Full visual overhaul deferred** to a later phase (Bard's Tale 8-16bit aesthetic).

## What This Phase Does NOT Include

| Deferred Item | Planned For |
|---|---|
| XP, levels, character progression | Phase 0.5 |
| Difficulty levels on quests | Phase 0.5 |
| Skills, Attributes, quest-to-skill links | Phase 0.5 |
| Quest giver / "next quest" suggestion | Phase 1 |
| Menu bar app, notifications, interrupts | Phase 1 |
| AFK timer | Phase 1 |
| Cleanup/archival of completed one-offs | TBD |
| More sophisticated timing (time-of-day, day-of-week) | TBD |
| Quest linking/ordering dependencies | TBD |
