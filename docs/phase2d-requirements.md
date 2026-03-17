# Phase 2D: Levelling — Requirements (WIP)

**Status:** Complete

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

Quest XP is currently flat based on cycle and difficulty. This adds a time-based multiplier using a piecewise formula (Option B from design discussion):

- **Below cycle** (r < 1): `0.1 + 0.9 × √r` — square root ramp from 0.1x to 1.0x
- **At cycle** (r = 1): 1.0x — full base XP
- **Above cycle** (r >= 1): `1.0 + 0.5 × ln(r)` — log growth, ~1.35x at 2x overdue, ~2.7x at 30x overdue
- **Floor**: 0.1x (always earn something)
- **One-off quests**: no modifier (1.0x)
- **Never completed**: no modifier (1.0x)

See `docs/mechanics.md` for full formula and reference table.

## 4. Retuning leveling curves — DONE

Attribute and skill level curves adjusted to slow progression:

- **Attributes**: changed from 1/5 to 1/2 of character curve (seeds: 60,100 → 150,250)
- **Skills**: changed from 1/10 to 1/8 of character curve (seeds: 30,50 → 37,62)
- **Character curve**: unchanged (seeds: 300, 500)
- **Base XP per difficulty**: unchanged
- **Skill level-up attribute bump**: unchanged (70 XP)

See `docs/mechanics.md` for updated level tables.
