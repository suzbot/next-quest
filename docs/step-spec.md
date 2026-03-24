# Step Spec: Phase 2H.1-1 — Stored last-done date ✅

## Goal

Move `last_completed` from a derived value (subquery on quest_completion) to a stored column on the quest table. Completion history becomes a read-only log. Users can manually set the last-done date via a date picker on the quest list. Deleting completions no longer affects due dates.

---

## Substep 1: Backend — migration, query changes, complete_quest update, set_quest_last_done

**Migration** (`db.rs` → `migrate()`):

Add `last_completed TEXT` column to quest. Populate from existing completions. Detection: check if quest table has `last_completed` column.

```sql
ALTER TABLE quest ADD COLUMN last_completed TEXT;
UPDATE quest SET last_completed = (
    SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = quest.id
);
```

**Query changes — remove completion subqueries:**

Four locations currently derive `last_completed` via `(SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id)`:

1. `get_quests` (~line 1722) — replace subquery with `q.last_completed`
2. `get_saga_steps` (~line 934) — replace subquery with `q.last_completed`
3. `query_single_quest` (~line 2598) — replace subquery with `q.last_completed`
4. `complete_quest` time-elapsed multiplier (~line 2155) — replace subquery with reading `quest.last_completed` from the initial quest data query (already fetched at top of function)

For #4: the initial query in `complete_quest` (`SELECT title, difficulty, quest_type, cycle_days, saga_id FROM quest`) needs to also select `last_completed`. Use this value for the time-elapsed multiplier instead of querying the completions table.

**`complete_quest` — update stored column:**

After inserting the completion record and deactivating one-offs, also update the quest's `last_completed`:

```sql
UPDATE quest SET last_completed = ?1 WHERE id = ?2
```

Using the same `completed_at` timestamp as the completion record. This goes inside the existing transaction (before `tx.commit()`).

**New function: `set_quest_last_done`**

```rust
pub fn set_quest_last_done(conn: &Connection, quest_id: String, last_done: Option<String>) -> Result<(), String>
```

- Updates `quest.last_completed` to the given value (or NULL if None)
- Validates the quest exists and is NOT a saga step (`saga_id IS NULL`). Returns error if it's a saga step.
- Does not create a completion record. Does not award XP.

**New command** (`commands.rs`): wrapper for `set_quest_last_done`. Register in `main.rs`.

**`init_db_memory` (test helper):**

The in-memory test DB creates tables directly. The quest CREATE TABLE needs to include `last_completed TEXT` so tests work without running migrations.

**Tests:**

1. `complete_quest_sets_last_completed` — Complete a quest. Verify `quest.last_completed` matches the completion timestamp (via `get_quests`).
2. `delete_completion_preserves_last_completed` — Complete a quest, note last_completed. Delete the completion. Verify last_completed is unchanged.
3. `set_quest_last_done_updates_due` — Create a daily quest. Set last_done to today. Verify `is_due` is false. Set last_done to 3 days ago. Verify `is_due` is true.
4. `set_quest_last_done_clear` — Set last_done to a date, then clear it (None). Verify quest shows as never completed (last_completed is None, is_due is true for recurring).
5. `set_quest_last_done_saga_step_rejected` — Create a saga with a step. Try to set last_done on the step. Verify it returns an error.
6. `time_elapsed_multiplier_uses_stored_column` — Existing XP tests should continue passing (verifies the time-elapsed multiplier reads from the stored column correctly).

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Frontend — date picker on quest list

**Date input** next to the last-done display in the normal (non-edit) quest list row.

The quest list row currently shows last-done as formatted text (e.g., "03/15 09:30"). Add an `<input type="date">` next to it:

```
  Take vitamins    MO  Easy  ↻1d   03/15 09:30  [2026-03-15]  ⚔  ✓
```

- The date input's value is populated from `quest.last_completed` (date portion only)
- Changing the date calls `set_quest_last_done` with the selected date as an ISO timestamp (midnight UTC)
- If the quest has no last_completed, the date input is empty
- Clearing the date input calls `set_quest_last_done` with null
- After setting, reload the quest list to reflect the updated due state

**Not shown on:**
- Saga steps (they appear in the saga tab, not the quest list)
- Inactive one-off quests (already completed, row is dimmed)

**Testing checkpoint:** Build app. Add a new quest. Set its last-done to yesterday — it should show as not due (for a daily quest). Clear the last-done — it becomes due again. Complete a quest, then delete the completion from history — last-done stays, quest stays not-due.

---

## NOT in this step

- Evening/Night time split (2H.1-2)
- Quest selector tuning (2H.1-3 through 2H.1-7)

## Done When

Both substeps complete. `last_completed` is a stored column on quest. Completions are a read-only log that don't drive due logic. Manual date picker works on quest list. Deleting completions doesn't change due dates. `cargo test` passes.
