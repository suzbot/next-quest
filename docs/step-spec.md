# Step Spec: Phase 5A-6b — Saga slots on quest list

## Goal

Sagas appear as slots on the quest list, interleaved with regular quests by sort_order. Each slot shows the current active step (or step 1 if not due/completed). Slots are filterable, searchable, and completable. Edit clicks navigate to the saga tab.

---

## Substep 1: Backend — get_quest_list

**New struct:**

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuestListItem {
    pub item_type: String,        // "quest" or "saga"
    pub quest: Option<Quest>,     // present for regular quests
    pub saga_slot: Option<SagaSlot>, // present for saga slots
    pub sort_order: i32,          // unified sort_order for ordering
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SagaSlot {
    pub saga_id: String,
    pub saga_name: String,
    pub saga_cycle_days: Option<i32>,
    pub step: Quest,              // current active step (or step 1)
    pub is_saga_due: bool,        // has an active run
    pub is_one_off: bool,         // one-off saga (cycle_days is None)
    pub is_completed: bool,       // one-off saga, all steps done ever
    pub sort_order: i32,          // saga's sort_order
}
```

**New function:** `get_quest_list(conn) -> Result<Vec<QuestListItem>, String>`

1. Call `get_quests(conn)` for regular quests
2. Call `get_sagas(conn)` + `get_saga_steps(conn, saga_id)` for each saga to compute slots:
   - For each saga, determine the current step to display:
     - If saga has an active run: first step not completed in current run (same logic as `get_active_saga_steps`)
     - If saga is not due (recurring between runs): step 1
     - If saga is completed (one-off, all steps done): step 1
   - Build `SagaSlot` with saga metadata + the display step
3. Merge quests and saga slots into `Vec<QuestListItem>`, sorted by sort_order
4. Batch-load skill/attribute/tag links for saga steps (same as regular quests)

**Command:** `get_quest_list` wrapper. Register in main.rs.

**Tests:**

1. `quest_list_includes_saga_slots` — Create quests and a saga with steps. Call `get_quest_list`. Verify both quest items and saga slot items are returned, sorted by sort_order.
2. `saga_slot_shows_active_step` — Create a saga with 2 steps. Complete step 1. Verify the saga slot shows step 2.
3. `saga_slot_not_due_shows_step_1` — Create a recurring saga, complete a run, verify the slot shows step 1 with `is_saga_due: false`.
4. `saga_slot_completed_oneoff` — Create a one-off saga, complete all steps. Verify `is_completed: true`, shows step 1.

**Testing checkpoint:** `cargo test` passes.

---

## Substep 2: Frontend — render saga slots on quest list

**Data loading:** Replace `invoke("get_quests")` in `loadAll` with `invoke("get_quest_list")`. Parse the merged list — separate quest items and saga slot items for rendering.

Actually simpler: `renderQuests` receives the full list and renders each item based on `item_type`.

**Saga slot row rendering:**

```
[Saga: Laundry] Move Laundry To Dryer  !!!!  MO  ↻4d  Trivial  03/25  ⚔ ✓
```

- `[Saga: Name]` badge styled distinctively (e.g., small, muted, before the title)
- Step metadata: importance, TOD, DOW, difficulty, last done — all from the step
- Cycle: from the saga (not the step)
- ⚔ calls `sagaStepQuestNow(stepId)`
- ✓ calls `completeSagaStep(stepId, sagaId)` — after completion, reloads quest list to show next step
- Title click: navigate to saga tab (call `showView('sagas')`, optionally expand the saga)
- Last-done date picker: NOT shown (saga steps manage last_completed through completion only)

**Due/cooldown styling:**

- Saga with active run, step is due: `quest-due` styling
- Recurring saga between runs: `quest-cooldown` styling (dimmed, like not-due recurring quests)
- One-off saga completed: `quest-cooldown` styling (dimmed, like completed one-offs)

**Filtering and search:**

`passesFilters` and `buildSearchText` work on saga slot items:
- `buildSearchText` includes: step title, saga name, step's skill/attribute/tag names, difficulty label, importance marks
- Difficulty filter checks step difficulty
- Importance filter checks step importance
- TOD/DOW filters check step settings
- Due filter: show saga slot only if `is_saga_due` is true

**Expand/detail row:**

Expanding a saga slot shows step links + debug scoring (same as quest rows).

**Debug scoring:**

`questScoreMap` keyed by step quest ID — saga steps' scores are already in the map from `get_quest_scores`.

**Testing checkpoint:** Build app. See saga slots interleaved on quest list with `[Saga: Name]` badge. Search by saga name — slot appears. Filter by difficulty — saga slot filtered by step difficulty. Complete a step from quest list — slot updates to next step. Completed/not-due sagas show dimmed.

---

## NOT in this step

- Mixed drag-and-drop reordering (next step)
- Saga tab reordering (not planned — quest list is source of truth)

## Done When

Saga slots appear on the quest list, interleaved by sort_order. Filterable, searchable, completable. Edit navigates to saga tab. Due/cooldown styling matches regular quest patterns. `cargo test` passes.
