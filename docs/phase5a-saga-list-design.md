# Phase 5A-6/7: Sagas on the Quest List — Design

## Overview

Sagas appear as first-class rows on the quest list, interleaved with regular quests. Each saga is a "slot" that shows its current active step. Users drag sagas up and down alongside quests to set priority. Sort order feeds into scoring identically for both. The saga tab remains for creating/editing sagas and steps.

## 1. Quest list display

### Active saga (has an active step)

```
[Saga: Laundry] Move Laundry To Dryer  !!!!  MO  ↻4d  Trivial  03/25 18:50  ⚔ ✓
```

- Saga badge: `[Saga: Name]` before the step title
- Step metadata displayed same as a regular quest: importance, TOD, DOW, cycle (from saga), difficulty, last done
- ⚔ and ✓ buttons work (Quest Now starts timer, Done completes the step)
- When step completes, the slot updates to show the next step in the saga
- When saga run completes (all steps done), the slot dims and shows step 1

### Recurring saga — not due (between runs)

Displays like a not-due recurring quest: dimmed/cooldown styling. Shows step 1 (first step). Not actionable until due.

### One-off saga — completed

Displays like a completed one-off quest: dimmed styling. Shows step 1. Can be deleted.

### Expanding a saga row

Clicking the expand toggle shows:
- Links (skills, attributes, tags) of the current step
- Debug scoring (if enabled)
- Same pattern as regular quest detail rows

### Editing

Clicking the saga slot's title navigates to the saga tab (does NOT open inline quest edit mode). Saga steps are managed on the saga tab.

### Filtering and search

Saga rows are subject to all quest list filters:
- **Fuzzy search** matches against step title, step's linked skills/attributes/tags, saga name, difficulty label, importance marks
- **Difficulty filter** matches the current step's difficulty
- **Importance filter** matches the current step's importance
- **TOD/DOW/Due filters** match the current step's settings and saga due state

## 2. Unified sort order

### Problem

Quest sort_order and saga sort_order are currently separate namespaces. To interleave them on the quest list, they need to share a namespace.

### Approach

Add a `list_position` field to both quests and sagas. This is a unified ordering across both entity types, independent of the existing sort_order fields.

Actually, simpler: reuse the existing `sort_order` field but ensure they're in the same numeric range. On migration, interleave existing sagas after existing quests (append saga sort_orders after the max quest sort_order).

### Migration

```sql
-- Shift saga sort_orders to be after all quest sort_orders
UPDATE saga SET sort_order = sort_order + (SELECT COALESCE(MAX(sort_order), 0) FROM quest WHERE saga_id IS NULL);
```

After migration, quest and saga sort_orders share one range. New quests and sagas get the next available sort_order from the combined max.

### Reordering

The `reorder_quests` function is replaced with a `reorder_list` function that accepts a mixed list of items:

```rust
pub struct ListItem {
    pub id: String,
    pub item_type: String, // "quest" or "saga"
}

pub fn reorder_list(conn, items: Vec<ListItem>) -> Result<(), String>
```

Updates `quest.sort_order` or `saga.sort_order` for each item in the provided order.

Frontend passes the ordered list of `{id, type}` pairs after a drag-and-drop.

### Scoring

`global_max_sort` in `get_quest_scores` considers both quest and saga sort_orders:

```rust
let quest_max = quests.iter().map(|q| q.sort_order).max().unwrap_or(0);
let saga_max = // max sort_order across active sagas
let global_max_sort = quest_max.max(saga_max).max(1) as f64;
```

Saga step `list_order_bonus` uses the saga's sort_order:

```rust
let list_order_bonus = 0.5 + 0.5 * (saga_sort_order as f64 / global_max_sort);
```

Range: 0.5–1.0. Bottom saga still gets a meaningful boost (committed work). Top saga matches top regular quest.

Wait — if quests range 0–1.0 and sagas range 0.5–1.0, a bottom-of-list saga always beats a bottom-half regular quest on sequence alone. But the user agreed on option B (0.5–1.0) earlier.

Actually, with unified sort_order, sagas and quests are in the same sequence. A saga dragged to position 3 out of 20 gets `sort_order / global_max = 3/20 = 0.15`. That's the same formula as regular quests. No special range needed — the position on the list IS the priority.

**Revised: sagas use the same `sort_order / global_max` formula as regular quests.** No 0.5 floor. If you drag a saga to the top, it gets max bonus. If you drag it to the bottom, it gets minimal bonus. The unified list makes this natural — your position is your priority.

### Open question resolved

The 0.5–1.0 range was designed for when sagas were separate from the quest list and needed a floor to ensure committed work surfaced. With sagas on the quest list, the user explicitly positions them — no floor needed. A saga at position 3 should score like a quest at position 3.

## 3. Backend — get_quests includes saga slots

### New function or modified get_quests

Option A: Modify `get_quests` to also return saga slots interleaved by sort_order.
Option B: New function `get_quest_list` that merges quests and saga slots.

**Recommend B** — `get_quests` is called in many places (scoring, quest giver, etc.) and changing its behavior could have side effects. A new `get_quest_list` function returns the merged view for the quest list tab only.

### Saga slot data

For each active saga, compute the "slot" — the current step to display:

```rust
pub struct SagaSlot {
    pub saga_id: String,
    pub saga_name: String,
    pub saga_sort_order: i32,
    pub saga_cycle_days: Option<i32>,
    pub step: Quest,          // the current active step (or step 1 if not due)
    pub is_saga_due: bool,    // whether the saga has an active run
    pub is_completed: bool,   // one-off saga, all steps done
}
```

`get_quest_list` returns a unified list: regular quests + saga slots, sorted by sort_order. The frontend renders both using the same row pattern, with the saga badge for saga slots.

### Commands

- `get_quest_list` — returns the merged list for the quest list tab
- `reorder_list` — reorders mixed quest/saga items

## 4. Frontend — rendering saga slots

### Quest list rendering

`renderQuests` receives the merged list. Each item is either a regular quest or a saga slot. Saga slots render with:

- `[Saga: Name]` badge before the step title
- Step metadata (importance, difficulty, TOD, DOW, last done from the step)
- Cycle from the saga (not the step)
- ⚔ button calls `sagaStepQuestNow(stepId)`
- ✓ button calls `completeSagaStep(stepId, sagaId)`
- Title click navigates to saga tab
- Due/cooldown styling based on saga due state
- Expand shows step links + debug scoring

### Drag and drop

Existing drag-and-drop on the quest list extends to handle saga slots. The `reorder_list` call passes `{id, type}` for each row.

### Filtering

`passesFilters` and `buildSearchText` work on the current step's data. Saga name is included in search text. Difficulty/importance filters check the current step. Due filter checks the saga's due state.

## 5. Saga tab

Unchanged — still used for:
- Creating sagas
- Adding/editing/reordering steps within a saga
- Viewing all sagas and their steps

The saga tab displays sagas in sort_order but does NOT allow reordering of sagas themselves. The quest list is the single source of truth for priority ordering — sagas are reordered there, among quests. The saga tab reflects the quest list's ordering.

## Implementation order

1. ~~**Unified sort_order + scoring**~~ ✅ — Sort_orders share one namespace. Saga step scoring uses `saga.sort_order / global_max_sort`. `global_max_sort` considers both tables.

2. **Saga slots on quest list** — `SagaSlot` struct. `get_quest_list` returns merged quests + saga slots sorted by unified sort_order. Frontend renders saga slots with `[Saga: Name]` badge, step metadata, ⚔/✓, due/cooldown styling. Title click navigates to saga tab. Filtering/searching works on step data + saga name. Testing: see saga slots interleaved on quest list, filter by difficulty or search by saga name, complete a step from the quest list.

3. **Mixed drag-and-drop on quest list** — `reorder_list` backend function accepts mixed quest/saga items. Frontend drag-and-drop handles both row types. Testing: drag a saga slot above a quest, verify sort_orders update, verify scoring reflects new position.

### Summary

Three vertical slices. Step 1 is done (scoring). Step 2 shows saga slots on the quest list (visible, filterable, completable). Step 3 enables mixed reordering (drag sagas among quests). Saga tab does NOT allow reordering — quest list is the single source of truth.
