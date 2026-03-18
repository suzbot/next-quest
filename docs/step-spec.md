# Step Spec: Phase 2F-1 — Time-of-Day Windows

## Goal

Quests gain a time-of-day preference. The quest giver doesn't use it yet (that's 2F-4), but the data is stored, editable, and displayed.

---

## Substep 1: Backend — migration, struct, CRUD

**Migration** (`db.rs` → `migrate()`):
- Add column `time_of_day TEXT NOT NULL DEFAULT 'anytime'` to `quest` table.
- Detection pattern: `SELECT time_of_day FROM quest LIMIT 0` (same as existing migrations).

**Quest struct** (`db.rs`):
- Add `time_of_day: String` to `Quest` struct.

**`get_quests`**:
- Add `q.time_of_day` to the SELECT. Map to the struct field.

**`add_quest`**:
- New parameter: `time_of_day: String`. Defaults handled by the frontend (passes `"anytime"` if unset).
- Include in INSERT.

**`update_quest`**:
- New optional parameter: `time_of_day: Option<String>`.
- When `Some`, UPDATE the column.

**`local_hour()`** — new helper function:
- Uses existing `libc::localtime_r` pattern (same as `unix_to_local_days`).
- Returns `u32` (0–23) for the current local hour.

**`matches_time_of_day(window: &str, hour: u32) -> bool`** — new public function:
- `"anytime"` → `true`
- `"morning"` → `hour >= 4 && hour < 12`
- `"afternoon"` → `hour >= 12 && hour < 17`
- `"evening"` → `hour >= 17 || hour < 4`
- Anything else → `true` (defensive default)

**commands.rs**:
- `add_quest`: add `time_of_day: String` parameter, pass through.
- `update_quest`: add `time_of_day: Option<String>` parameter, pass through.

**Tests:**
- `matches_time_of_day` — all four windows at boundary hours: 3, 4, 11, 12, 16, 17, 23, 0.
- `add_quest` with `time_of_day` set to `"morning"` — verify persisted and returned.
- `update_quest` changing `time_of_day` from default to `"evening"` — verify updated.
- `get_quests` returns `time_of_day` field.
- Existing tests updated to pass the new `time_of_day` parameter (use `"anytime"`).

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Frontend — add form, edit form, display

**Add form** (`index.html`):
- New dropdown after the difficulty select:
  ```html
  <select id="quest-time-of-day">
    <option value="anytime" selected>Anytime</option>
    <option value="morning">Morning</option>
    <option value="afternoon">Afternoon</option>
    <option value="evening">Evening</option>
  </select>
  ```
- `addForm` submit handler reads the value, passes to `invoke("add_quest", { ... timeOfDay })`.

**Edit form** (`renderEditMode`):
- New dropdown `id="edit-time-of-day"` with the same four options, pre-selecting the quest's current value.
- `saveEdit` reads the value, passes to `invoke("update_quest", { ... timeOfDay })`.

**Expanded detail row** (`renderQuestItem`):
- If `time_of_day` is not `"anytime"`, append it to the detail line (e.g., "Morning" alongside skill/attribute links).
- Capitalize for display: `morning` → `Morning`.
- If no links and time is anytime, no detail row (existing behavior — `hasDetail` stays false).

**Testing checkpoint:** Build app. Add a quest with "Morning" — see it in the expanded detail. Edit an existing quest to "Evening" — detail updates. Add a quest with "Anytime" — no time shown in detail.

---

## NOT in this step

- Day-of-week affinity (2F-2)
- Quest giver filtering by time-of-day (2F-4)
- Quest list filter bar (2F-3)
- Scoring system (2F-4)
- Skip tracking (2F-5)

## Done When

Both substeps complete. Quests can be created and edited with a time-of-day window. The value is stored, returned by the API, and visible in the quest list detail row. `cargo test` passes.
