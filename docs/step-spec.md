# Step Spec: History UI — Show Full Completion Data

## Goal

Make the history tab display all the data we're capturing in completions, matching the quest list's visual patterns. Two changes:
1. Show difficulty, cycle, tags, skills, and attributes on each completion entry (inline + expandable, same as quest list)
2. Show parent saga name on saga step completions (badge, same as quest list)

## What changes

### Backend: Add saga_name to completions

The completion table snapshots difficulty, cycle, skills, attributes, and tags — but not saga name. Saga step completions are indistinguishable from regular quest completions in history.

1. **Migration:** `ALTER TABLE quest_completion ADD COLUMN saga_name TEXT`
2. **Backfill:** Update existing completions by joining through quest_id -> quest.saga_id -> saga.name (best-effort — only works for non-deleted quests/sagas)
3. **Completion struct:** Add `saga_name: Option<String>`
4. **get_completions:** Include saga_name in SELECT
5. **complete_quest:** When quest has saga_id, look up saga.name and snapshot it

### Frontend: Completion entry layout

Currently each completion shows: title (strikethrough) | +XP | timestamp | edit | delete

New layout, matching quest list patterns:

**Single row (all inline):**
- Saga badge — gray chip with saga name, only for saga step completions
- Title (strikethrough, gray) — existing
- Linked names — comma-separated skills, attributes, tags (small, gray, same style as quest list's `.quest-links`)
- +XP — existing
- Cycle text — `↻ Nd` or `One-off`, matching quest list's `.meta-cycle` style
- Difficulty label — color-coded text, matching quest list's `.meta-difficulty` style and color map
- Timestamp — existing
- Edit date / Delete — existing

If real estate can't handle it, switch links to an expandable toggle.

### CSS

Reuse existing classes where possible: `.saga-badge`, `.meta-difficulty`, `.meta-cycle`, `.expand-toggle`, `.quest-detail`, `.quest-links`. Add completion-specific overrides only if needed for spacing.

## Out of scope

- Level-up display in history (level_ups/xp_awards are computed ephemerally, not stored)
- XP derivation from history (separate discussion in progress)
- Grouping saga steps visually (just the badge for now)

## Test plan

1. Build and open the app
2. History tab should show difficulty labels (color-coded) and cycle text on existing completions
3. Completions with tags/skills/attributes should have ▸ toggle; expanding shows the linked names
4. Complete a saga step — its completion should show the saga name badge
5. Pre-migration completions (NULL difficulty/skills/etc.) should render cleanly with no metadata shown
6. Bonus entries (saga/campaign completion) should render cleanly — no difficulty, no cycle, no expand toggle
