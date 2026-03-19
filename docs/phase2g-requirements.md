# Phase 2G.1: Sagas — Requirements (WIP)

**Status:** Draft

**Goal:** Multi-step goals with ordered sub-quests, surfaced one step at a time through the quest giver. Sagas can be one-off or recurring.

---

## 1. Saga entity

A saga is a named container for an ordered sequence of steps.

| Field | Description |
|---|---|
| Name | User-given title ("Spring Cleaning", "File Taxes") |
| Cycle days | For recurring sagas: days after the last step's completion before the saga restarts. NULL for one-off sagas. |

Sagas do not have their own difficulty, time-of-day, day-of-week, or skill/attribute links. All of those live at the step level.

## 2. Saga steps are quests

Saga steps are quests with an additional saga parent and step order. They reuse all existing quest infrastructure: title, difficulty, time-of-day, day-of-week, skill/attribute links, XP calculation, completion tracking.

- Saga steps do **not** appear in the Quest List tab. They live in the Saga tab and surface through the quest giver.
- Each step has a `step_order` determining its sequence within the saga.

## 3. Step "due" logic

A saga step is due when:
1. All prior steps (by step_order) have been completed in the current run, AND
2. This step has not been completed in the current run

**Current run** = the set of completions that occurred after the saga's cycle reset point. For one-off sagas, the current run is all completions ever. For recurring sagas, the cycle reset point is the most recent completion of the last step (from the prior run) plus `cycle_days`.

A saga's first step becomes due when:
- **One-off saga**: immediately (never completed)
- **Recurring saga**: `cycle_days` have elapsed since the last step was most recently completed

## 4. Quest giver integration

- The quest giver pulls one active step per saga into the candidate pool alongside regular quests.
- The active step is the first due step (lowest step_order that meets the due criteria above).
- All active sagas contribute simultaneously — no limit on how many sagas are active.
- Saga steps score using the same formula as regular quests. The active step scores like a one-off: `(days_since_activated + 9) / 9`, where `days_since_activated` is the number of days since the previous step was completed (or since the saga became due, for the first step).
- **First step of a new recurring saga** (no prior completions): the step is due immediately and scores based on days since the saga was created, same as a new one-off quest. There's no prior run to derive a cycle reset from, so the saga is treated as starting its first run on creation.

## 5. Step auto-advance

When a step is completed, the next step enters the candidate pool immediately. Since it just became due, its overdue ratio starts at 1.0 — the quest giver won't push it aggressively at first.

## 6. Out-of-order completion

Users can navigate to the Saga tab and complete any step, even if prior steps are incomplete — same as completing a non-due quest from the Quest List. The quest giver still follows strict sequence: it only surfaces the first due step. Out-of-order completions count toward the current run.

## 7. Saga completion XP bonus

When the last step of a saga is completed, a bonus is awarded:
- **20% of the baseline XP** of all steps in the saga (calculated at time multiplier = 1.0, using each step's difficulty and a one-off cycle multiplier of 3x).
- This prevents the bonus from rewarding procrastination — it's based on what on-time completion would have earned, not actual inflated XP.
- The bonus is awarded to the character and to any skills/attributes linked to the final step.

For recurring sagas, the bonus is earned each time the full sequence is completed.

## 8. Saga tab

A new tab in the app for managing sagas.

**Collapsed view (saga list):**
- Saga name, cycle info, progress bar showing % of steps complete in the current run

**Expanded view (step list):**
- Same UI as the Quest List for managing steps: add, edit, resequence, delete, mark complete
- Same add-quest form (title, difficulty, TOD, DOW, skill/attribute tags)
- Steps show completion status for the current run
- **Full resequencing**: steps can be reordered freely within their saga via drag-and-drop and keyboard (arrow keys to move position), same as quest list reordering. Not limited to neighbor swaps — any step can be moved to any position.

**Saga management:**
- Add new saga (name, optional cycle days)
- Edit saga name and cycle
- Delete saga (deletes all steps; completions are orphaned, same as quest deletion)

---

## Out of scope

- Saga pause/shelve (can add later if needed)
- Non-linear/parallel steps (strictly sequential)
- Saga-level skill/attribute links
- Template sagas (Phase 3)
