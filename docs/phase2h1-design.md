# Phase 2H.1: Second Logic Pass — Design

## Overview

Three independent improvements implemented in order: stored last-done date, evening/night time split, and quest selector tuning (5 layered scoring changes).

## 1. Stored last-done date

### Architecture change

Move `last_completed` from a derived value to a **stored column on the quest table**. Completion history becomes a read-only log that does not drive any game logic.

### Schema

Add column to `quest`:

```sql
ALTER TABLE quest ADD COLUMN last_completed TEXT;
```

Migration: populate from existing completions:

```sql
UPDATE quest SET last_completed = (
    SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = quest.id
);
```

### Backend changes

**`complete_quest`:**
- After inserting the completion record, update `quest.last_completed` to the completion timestamp
- The time-elapsed multiplier reads `quest.last_completed` (before updating it) instead of querying the completions table

**`get_quests`:**
- Read `q.last_completed` directly from the quest row
- Remove the `(SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id)` subquery

**`get_saga_steps`:**
- Same change — read from stored column, drop subquery

**`delete_completion`:**
- No longer has any side effect on quest due dates

**New function: `set_quest_last_done(conn, quest_id, last_done: Option<String>)`**
- Updates `quest.last_completed` directly
- Setting to None clears the last-done (quest becomes "never completed" for due calculation)
- Only allowed on non-saga-step quests (saga steps manage last_completed through `complete_quest` only)

**New command:** `set_quest_last_done` wrapper. Register in `main.rs`.

### Frontend

Date input next to the last-done display in the quest list row (normal view, not edit mode). Changing the date calls `set_quest_last_done`. Input type is `date` (date only, no time — sets to midnight UTC of that day).

Saga steps do not show this date picker (they appear in the saga tab, not the quest list).

### Tests

1. Complete a quest, verify `last_completed` column is set
2. Delete a completion, verify `last_completed` is unchanged
3. Set last-done manually, verify `is_due` shifts accordingly
4. Clear last-done, verify quest becomes "never completed" for due calculation
5. Time-elapsed multiplier reads from stored column (existing XP tests should still pass)

## 2. Evening/Night time-of-day split

### Time windows

| Window | Bit | Hours | Label |
|---|---|---|---|
| Morning | 1 | 4am – noon | MO |
| Afternoon | 2 | noon – 5pm | AF |
| Evening | 4 | 5pm – 9pm | EV |
| Night | 8 | 9pm – 4am | NT |

"All times" mask: 15 (was 7). Mask 0 continues to mean "all times."

### Migration

```sql
-- Quests with old evening bit (4) get night bit (8) added
UPDATE quest SET time_of_day = time_of_day | 8 WHERE (time_of_day & 4) != 0;
-- Same for default "all times" mask
UPDATE quest SET time_of_day = 15 WHERE time_of_day = 7;
```

Same migration for saga steps (they're quests with `saga_id` set, so the above covers them).

Settings: no migration needed (settings stores the filter value, which resets on change).

### Backend changes

**`matches_time_of_day`:** Update to 4-window logic:

```rust
let current_bit = if hour >= 4 && hour < 12 {
    1  // morning
} else if hour >= 12 && hour < 17 {
    2  // afternoon
} else if hour >= 17 && hour < 21 {
    4  // evening
} else {
    8  // night (9pm-4am)
};
```

**`default_time_of_day`:** Change from 7 to 15.

**`NewQuest` and `NewSagaStep` defaults:** Change from 7 to 15.

### Frontend changes

- Time-of-day multiselect gains NT option in quest add/edit, saga step add/edit, and filter bar
- Display: `todText` and `todSummary` updated for 4 values (MO, AF, EV, NT)
- "All" threshold changes from 7 to 15

### Tests

1. `matches_time_of_day` at boundary hours (4, 12, 17, 21, 3)
2. Migration: quest with old mask 4 → new mask 12 (4+8)
3. Migration: quest with old mask 7 → new mask 15
4. New quest defaults to 15

## 3. Quest selector tuning

### Current scoring formula

```
score = overdue_ratio - skip_penalty + list_order_bonus
```

Where:
- `overdue_ratio`: `(days_overdue + cycle) / cycle` for recurring, `(days + 9) / 9` for one-off
- `skip_penalty`: `skip_count × 0.5`
- `list_order_bonus`: `0.01 × sort_order / max_sort_order` (negligible)

### New scoring formula (after all 5 changes)

```
score = overdue_ratio + importance_boost + list_order_bonus + membership_bonus + balance_bonus - skip_penalty
```

### 3a. Saga step scoring — use saga cycle

**Change:** Saga steps currently use `(days + 9) / 9` regardless of saga cycle. Change to `(days_since_activated + saga_cycle) / saga_cycle`. One-off sagas (cycle_days=null) continue using 9.

**Code location:** `get_next_quest` in db.rs, saga step scoring block (~line 1629).

Currently:
```rust
let overdue_ratio = (days_since + 9.0) / 9.0;
```

Becomes:
```rust
let saga_cycle = saga_cycle_days.unwrap_or(9) as f64;
let overdue_ratio = (days_since + saga_cycle) / saga_cycle;
```

`get_active_saga_steps` currently returns `(Quest, saga_name, activated_at)`. Add `saga_cycle_days: Option<i32>` to the tuple so the scoring code can use it.

**Tests:**
1. Daily saga step scores 2.0 after 1 day (was 1.11)
2. Weekly saga step scores ~1.14 after 1 day (was 1.11)
3. One-off saga step still uses 9-day base

### 3b. Importance field

**Schema:** Add column to `quest`:

```sql
ALTER TABLE quest ADD COLUMN importance INTEGER NOT NULL DEFAULT 0;
```

Values 0–5. Default 0.

**Structs:** Add `importance: i32` to `Quest`, `NewQuest`, `QuestUpdate`, and `NewSagaStep`. Update `add_quest`, `update_quest`, `add_saga_step`, and all query functions that build Quest structs to include the field.

**Scoring:** `importance_boost = importance × 30.0`

At importance 5: +150.0. Each importance level is ~30 days of overdue on a daily quest. A 0! quest needs ~150 days overdue to tie with a just-due 5!.

Applies to both due and not-due pools — a high-importance quest should surface even in the not-due fallback.

**Display:** Quest list and saga step list show importance as exclamation marks after the title:
- 0: nothing
- 1–5: "!" through "!!!!!"

Styled in a muted color so it's visible but not distracting. Not shown in the quest giver — only in the list/step views.

**Quest add/edit:** Importance selector (0–5) in the add form and edit mode.

**Saga steps:** Also get the importance field via `NewSagaStep` and the step add/edit form.

**Tests:**
1. Quest with importance 3 scores 1.2 higher than importance 0 at same overdue
2. Default importance is 0

### 3c. List-order weight increase

**Change:** List-order bonus uses the **full quest list's max sort_order** (not the candidate pool) and increases from 0.01 to 1.0 max.

```rust
// Computed once from all quests, not just the due pool
let global_max_sort = all_quests.iter().map(|q| q.sort_order).max().unwrap_or(1) as f64;

// Per quest:
let list_order_bonus = 1.0 * q.sort_order as f64 / global_max_sort;
```

Top-of-list quest gets +1.0, bottom gets ~0. This reflects the user's intended priority.

Applies to both due and not-due pools (same global max, same weight).

**Saga steps:** Continue using a fixed small bonus (0.1) since they don't participate in the quest list ordering.

**Combined max check:** A just-due quest at top of list, importance 5:
```
1.0 (overdue) + 2.0 (importance) + 1.0 (list order) = 4.0
```
This ties with a 3-day-overdue daily at importance 0, bottom of list. A 4+ day overdue quest still wins. Skip penalty (0.5/skip) provides session-level escape valve.

**Tests:**
1. Top-of-list quest scores higher than bottom-of-list at same overdue/importance
2. Bonus uses global max, not candidate pool max

### 3d. Saga/campaign membership bonus

**Change:** +0.2 for quests that are:
- Active saga steps (quest has `saga_id` set and saga is active), OR
- Referenced as a criterion in any active campaign (`completed_at IS NULL`)

These don't stack — it's +0.2 if either condition is true.

Applies to both due and not-due pools.

For regular quests (not saga steps), the campaign check requires a query. To avoid N+1 queries, precompute the set of quest IDs referenced by active campaigns (`target_type = 'quest_completions'`, `completed_at IS NULL`) before scoring.

**Tests:**
1. Quest in active campaign scores 0.2 higher than equivalent quest not in any campaign
2. Saga step already gets the bonus (it's always an active saga step)
3. Quest in completed campaign does not get the bonus

### 3e. Attribute/skill balancing

**Change:** Small boost for quests linked to underleveled attributes or skills.

**Formula:**
1. Compute `avg_attr_level` across all attributes
2. Compute `avg_skill_level` across all skills
3. For each quest's linked attributes: `max(0, avg_attr_level - attr_level) × 0.1`
4. For each quest's linked skills: `max(0, avg_skill_level - skill_level) × 0.1`
5. Quest's balance bonus = max across all linked attributes and skills

**Example:** Average attribute level is 3, average skill level is 3. A quest linked to Cooking (level 1, skill) and Health (level 2, attribute):
- Skill bonus: (3 - 1) × 0.1 = 0.2
- Attribute bonus: (3 - 2) × 0.1 = 0.1
- Quest gets max: 0.2

Applies to both due and not-due pools.

**Performance:** Attribute/skill levels need to be loaded once per `get_next_quest` call. Quest links are already loaded in the Quest struct (`skill_ids`, `attribute_ids`). The level lookup is a small additional query at the start of scoring.

**Tests:**
1. Quest linked to underleveled skill scores higher than quest linked to average-level skill
2. Quest with no links gets 0 balance bonus
3. Quest linked to overleveled attribute gets 0 balance bonus

### Debug scoring display

Add new scoring components to `ScoredQuest` struct: `importance_boost`, `membership_bonus`, `balance_bonus`. Update the debug scoring display in the quest giver to show all components:

```
Score: 3.60 (overdue: 2.00 | importance: +1.20 | order: +0.80 | member: +0.20 | balance: +0.10 | skips: -0.70)
```

## Implementation Order

Each step is a vertical slice — backend + frontend + tests, independently testable.

1. **Stored last-done date** — migration, `complete_quest` update, query simplification, `set_quest_last_done` command, date picker UI
2. **Evening/Night split** — migration, `matches_time_of_day` update, frontend multiselect + display
3. **3a: Saga step scoring** — one formula change + tests
4. **3b: Importance field** — migration, struct updates, scoring change, UI (add/edit/display)
5. **3c: List-order weight** — scoring change (use global max, increase weight)
6. **3d: Saga/campaign membership** — precompute campaign quest set, scoring change
7. **3e: Attribute/skill balancing** — level lookup, scoring change

### Summary

Seven vertical slices, each independently deployable. The first two (last-done, evening/night) are independent changes. Steps 3a–3e are layered scoring changes that build on each other but can each be tested in isolation. The scoring formula grows from:

```
score = overdue_ratio - skip_penalty + list_order_bonus
```

To:

```
score = overdue_ratio + (importance × 30.0) + (sort_order/global_max) + membership_bonus + balance_bonus - (skips × 0.5)
```

With typical ranges:
- overdue_ratio: 1.0 – 5.0+ (primary signal)
- importance: 0 – 2.0 (strong secondary)
- list_order: 0 – 1.0 (meaningful nudge)
- membership: 0 or 0.2 (tiebreaker)
- balance: 0 – ~0.3 (gentle nudge)
- skip_penalty: 0 – 2.0+ (session escape valve)
