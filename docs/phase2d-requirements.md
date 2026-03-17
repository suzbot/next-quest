# Phase 2D: Levelling — Requirements (WIP)

**Status:** WIP — items to be discussed individually before implementation

**Goal:** Make XP and leveling feel right. Reward consistency without punishing absence, add time-awareness to quest rewards, and clean up the quest list UI.

---

## 1. Tab renames — DONE

- "Quests" tab → "Quest List"
- "Quest Giver" tab → "Next Quest"

## 2. Re-organize quest list — DONE

- **Collapsed row**: ▸ | Title | Difficulty | Cycle | Last Done | ⚔ | ✓
- **Expanded row**: ▸ toggle reveals skills/attributes detail line (and future meta in 2F+)
- **Fixed-width columns**: Difficulty, Cycle, and Last Done are fixed-width, right-aligned for vertical scanning
- **Icon buttons**: "Done" → ✓, "Quest Now" → ⚔
- **Delete button**: moved to edit mode only (alongside Save and Del)
- **Edit close**: unstyled ✕ replaces Esc button, right-aligned
- **"Links" button → "Tags"** on add form
- **Completed section → "History"**, moved to bottom of Character tab
- **Removed "Next Quest" header** from quest list view
- **Next Quest tab text top-aligned** with quest giver image

## 3. XP time-elapsed modifier

Quest XP is currently flat based on cycle and difficulty. This adds a time-based multiplier:

- **Too soon** (quest completed well before its cycle is up): diminishing XP. Doing laundry twice in one day shouldn't give full XP both times.
- **On time** (around when the cycle says it's due): full XP. This is the baseline.
- **Overdue** (past due): increasing XP to encourage doing the thing. But log-curve — procrastinating for a month shouldn't give 10x XP.
- Exact curve shape and multiplier ranges TBD.
- One-off quests: no modifier (no cycle to measure against).

## 4. Retuning XP amounts and leveling formula

Current XP values were set during Phase 0.5 and haven't been adjusted since. After living with the app, revisit:

- Base XP per difficulty tier
- Level thresholds (how much XP per level)
- Attribute/skill XP rates relative to general XP
- How the time-elapsed modifier interacts with all of the above

This is a tuning pass, not a formula rewrite. Exact values TBD after the time-elapsed modifier is in and playable.
