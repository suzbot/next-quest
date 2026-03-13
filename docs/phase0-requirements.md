# Phase 0: "The List" — Requirements

## Goal

A basic quest list that lets you seed, view, manage, and complete quests.
This is the foundation the quest giver will later draw from. For now,
you are your own quest giver — the app just holds the list and tracks completion.

## Core Concepts

- **Quest** = a template for something you do ("Take a shower", "File taxes"). Has a title and cycle. Can be edited, deleted, reordered.
- **Completion** = a visible record that you did a quest at a specific time. Each completion is its own row in the list. Can be individually deleted.
- Quests and completions are independent: deleting a quest does not delete its completion history. Deleting a completion does not affect the quest.

## Quest Properties

Every quest has:
- **Title**: Free text name ("Take a shower", "File taxes")
- **Cycle**: Number of days between reminders (null = one-off quest). Default: recurring, 1 day.
- **Sort Order**: Player-defined position in the active section. Higher = more prominent / suggested first.

## Views & Interactions

### List Layout

The list has two sections:

1. **Active quests** (top): ordered by sort_order (highest first). These are reorderable.
2. **Completion history** (bottom): reverse chronological (most recent first). Strikethrough style. Not reorderable.

### Active Quest Visual States

- **Due / Refreshed**: Emphasized style. The quest's cycle has elapsed since last completion (or it's never been completed). Ready to do.
- **Recently completed (recurring)**: De-emphasized style. Cycle hasn't elapsed yet. Still visible and completable, just visually quieter.

### Completion Records

- Each completion is a separate row showing: quest title, completion timestamp, [Del] button.
- Strikethrough style.
- Individually deletable (for cleanup of mistakes, tests, etc.).

### Actions

- **Add quest**: Create a new quest. Title is required. Cycle defaults to recurring/1 day (user can change to one-off or a different cycle).
- **Edit quest**: Change title, type (recurring/one-off), or cycle of an active quest.
- **Delete quest**: Remove a quest from the active list. Does not delete its completion records.
- **Mark done**: Record a completion. Can be done multiple times per day. On completion:
  - A completion record is created with the current timestamp and appears in the completion section.
  - For one-off quests: the quest is deactivated (greyed out in the active section). Can still be explicitly deleted.
  - For recurring quests: the quest stays active, shifts to de-emphasized style until cycle elapses.
- **Delete completion**: Remove an individual completion record from the history.
- **Re-sequence**: Drag or move active quests to change their sort order. Completions are not reorderable.

## Behaviors

- **Cycle refresh**: A recurring quest returns to "due/refreshed" style when `last_completed + cycle_days` has passed. No notification or automation yet — just the visual state change.
- **No lockout**: A quest can always be completed regardless of cycle state. Cycle only affects visual emphasis.
- **Multiple completions**: Completing a quest multiple times in a day is allowed. Each completion is a separate record.

## Interaction Requirements

- **Keyboard-first**: Every action (add, edit, delete, mark done, resequence) must be achievable with keyboard only. Tab navigation through the list and actions. Keyboard shortcut for reordering (e.g. Alt+Up/Down).
- **Mouse/drag also supported**: Drag-and-drop reordering works too, but keyboard is not an afterthought — it's the primary input.

## Visual Design

- **Bare text for now.** No pixel art, no themed UI. Functional and readable.
- **Monospace font** throughout.
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
| Recreate / duplicate a completed one-off quest | TBD |
| More sophisticated timing (time-of-day, day-of-week) | TBD |
| Quest linking/ordering dependencies | TBD |
