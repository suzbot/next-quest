# Phase 2G.1: Sagas — Design

## Overview

Sagas are ordered multi-step goals. Steps are quests with a saga parent. The quest giver surfaces one step per saga alongside regular quests. Sagas can be one-off or recurring.

## 1. Schema Changes

### New table: `saga`

| Column | Type | Default | Description |
|---|---|---|---|
| id | TEXT PK | — | UUID |
| name | TEXT NOT NULL | — | Saga title |
| cycle_days | INTEGER | NULL | NULL = one-off, set = recurring |
| sort_order | INTEGER NOT NULL | — | Display order (creation order) |
| active | INTEGER NOT NULL | 1 | 1 = active |
| created_at | TEXT NOT NULL | — | ISO 8601 creation time |
| last_run_completed_at | TEXT | NULL | ISO 8601 timestamp when the most recent full run completed. Stamped on saga completion, not recomputed. |

### New columns on `quest`

| Column | Type | Default | Description |
|---|---|---|---|
| saga_id | TEXT | NULL | FK to saga. NULL = regular quest. |
| step_order | INTEGER | NULL | Position within saga. NULL = regular quest. |

### Migration

- Create `saga` table
- Add `saga_id` and `step_order` to `quest` with NULL defaults
- Existing quests are unaffected (both columns NULL)

## 2. Current Run Logic

The "current run" determines which completions count toward saga progress. No status columns — purely derived from completion timestamps.

### Current run start

```
one-off saga:                        saga.created_at
recurring, last_run_completed_at is NULL:  saga.created_at
recurring, last_run_completed_at is set:   last_run_completed_at + cycle_days
```

`last_run_completed_at` is stamped on the saga row when a full run completes. Not recomputed from completions — once stamped, it's permanent (consistent with XP-only-goes-up principle).

### Saga due check

A saga is **due** (has an active step to offer) when:
- One-off: at least one step has no completion ≥ current_run_start
- Recurring, never completed: immediately (first run starts on creation)
- Recurring, previously completed: `cycle_days` have elapsed since `last_run_completed_at`

A saga that is not yet due contributes no steps to the quest giver pool.

### Step due check (linear sequence)

Within a due saga, a step is **the active step** when:
1. All steps with lower step_order have a completion ≥ current_run_start
2. This step does NOT have a completion ≥ current_run_start

The quest giver only surfaces the active step — strictly linear. However, the user can complete any step from the saga tab regardless of sequence, same as completing a non-due quest from the Quest List. An out-of-order completion counts toward the current run — the quest giver then skips that step and surfaces the next one that still needs completion.

### Step auto-advance

When a step is completed, the next step not yet completed this run becomes due immediately. Since it was just activated, its overdue ratio starts low (1.0) — the quest giver won't push it aggressively over regular quests that are more overdue.

### Saga complete check

A saga's current run is **complete** when ALL steps have a completion ≥ current_run_start. This is true regardless of completion order — if the user completes steps 1, 2, 4, then goes back and completes 3, the saga is complete at that moment because all steps now have a current-run completion.

### Recurring saga reset

When all steps are complete, `last_run_completed_at` is stamped with the current timestamp. The saga becomes due again when `cycle_days` have elapsed since `last_run_completed_at`.

## 3. Quest Selector Integration

### `get_active_saga_steps(conn) -> Vec<Quest>`

New function that returns one quest per active saga: the first due step (lowest step_order meeting the due criteria).

### Changes to `get_next_quest`

Currently calls `get_quests` for the candidate pool. Updated to:
1. `get_quests` — regular quests only (saga_id IS NULL)
2. `get_active_saga_steps` — one step per active saga
3. Merge both into the candidate pool
4. Apply hard filters (time-of-day, day-of-week) and scoring as usual

### Saga step scoring

Active saga steps score like one-off quests:
```
overdue_ratio = (days_since_activated + 9) / 9
```

Where `days_since_activated` =
- First step: days since saga.created_at (new saga) or days since current_run_start (recurring reset)
- Subsequent steps: days since the previous step was completed

### Quest giver display

When a saga step is shown, the saga name appears as smaller text above or below the quest title:
```
Spring Cleaning
  Mop the floors
```

## 4. Saga Completion Bonus

When the last step in a run is completed (all steps now have a current-run completion):

**Bonus formula:**
```
baseline_per_step = 5 × difficulty_mult × 3    (one-off cycle mult, time mult = 1.0)
bonus = round(0.20 × sum of all steps' baseline)
```

**Distribution:** bonus XP awarded to character + the final step's linked skills/attributes.

**Recurring sagas:** the bonus is earned each time the full sequence is completed.

## 5. Saga Tab UI

### Tab placement

Between Quest List and Character in the nav bar.

### Collapsed saga view

```
▸ Spring Cleaning          ↻ 30d   [████░░░░░░] 4/10
▸ File Taxes                        [██████████] Done
```

- Saga name, cycle text (same format as quests: ↻ #d or blank for one-off)
- Progress bar: completed steps in current run / total steps
- "Done" when all steps complete in current run (one-off stays done; recurring shows done until reset)

### Expanded saga view

Same UI patterns as Quest List:
- Add step form (same fields as quest add: title, difficulty, TOD, DOW, tags)
- Step list with same layout as quest list rows
- Edit mode with same controls
- Completion (✓) and Quest Now (⚔) buttons per step
- Full resequencing via drag-and-drop and keyboard arrow keys — any step can move to any position, not limited to neighbor swaps

### Saga management

- "Add Saga" form at top: name, optional cycle days
- Edit saga name/cycle inline (same pattern as attribute/skill rename)
- Delete saga with confirm (deletes all steps; completions are orphaned)

## 6. `get_quests` filter

Update `get_quests` to exclude saga steps (WHERE saga_id IS NULL) so the Quest List tab stays clean. Saga steps are only visible in the Saga tab and through the quest giver.

## Implementation Order

1. **2G.1-1: Schema + saga CRUD + basic tab** — User can create, rename, and delete sagas from a new Saga tab. Sagas appear as a list with name and cycle info. No steps yet — just the container. Testing: create a saga, rename it, delete it.

2. **2G.1-2: Step management** — User can add, edit, delete steps within a saga. Steps use the same add/edit form as quests (title, difficulty). Expanded saga view shows the step list. Completing a step from the saga tab records a normal completion with XP. Testing: add steps to a saga, edit and delete individual steps, complete a step and verify XP.

3. **2G.1-3: Quest selector integration** — Saga steps appear in the quest giver alongside regular quests. Current run logic determines which step is active per saga. Hard filters (TOD, DOW) apply to saga steps like any quest. Saga name shown as smaller text with the step title. Testing: create a saga with steps, see the first step offered by the quest giver. Complete it, see the next step surface. Complete all steps, saga no longer offers steps (or resets after cycle for recurring).

4. **2G.1-4: Completion detection + stamp + due styling + progress bar** — When all steps in a run are complete, stamp `last_run_completed_at`. Saga and step rows show due/not-due/cooldown styling. Progress bar on collapsed saga rows. Last-run date on saga row. Testing: complete all steps, verify saga shows as done/cooldown. Recurring saga resets after cycle elapses. Progress bar reflects current run.

5. **2G.1-5: Skill/attribute tags + Quest Now** — Tags button on step add/edit forms, linking steps to skills and attributes. XP flows to linked skills/attributes on completion. Quest Now (⚔) button on steps starts timer flow. Testing: add a step with tags, complete it, verify skill/attribute XP. Quest Now enters timer mode. Full step UI is now complete for balance testing.

6. **2G.1-6: Completion bonus + celebration** — 20% bonus of baseline step XP awarded when final step of a run completes. Celebration notification similar to level-up. Bonus distributes to character + final step's linked skills/attributes. Testing: complete all steps, verify bonus XP and celebration.

7. **2G.1-7: Resequencing** — Full drag-and-drop and keyboard resequencing for saga steps (any position, not neighbor-swap). Backend `reorder_saga_steps` is already implemented. Testing: reorder steps, verify order persists.
