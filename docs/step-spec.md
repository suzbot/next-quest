# Step Spec: Phase 2F-2 — Day-of-Week Affinity

## Goal

Quests gain a day-of-week preference stored as a bitmask. The quest giver doesn't filter by it yet (that's 2F-4), but the data is stored, editable, and displayed.

---

## Substep 1: Backend — migration, struct, CRUD

**Migration** (`db.rs` → `migrate()`):
- Add column `days_of_week INTEGER NOT NULL DEFAULT 127` to `quest` table.
- 127 = all bits set = every day. Detection: `SELECT days_of_week FROM quest LIMIT 0`.

**Bitmask encoding:**
- Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64

**Quest struct** (`db.rs`):
- Add `days_of_week: i32` to `Quest` struct.

**`get_quests`** / **`query_single_quest`**:
- Add `q.days_of_week` to SELECT, map to struct field.

**`add_quest`**:
- New parameter: `days_of_week: i32`. Frontend passes 127 by default.
- Include in INSERT.

**`update_quest`**:
- New optional parameter: `days_of_week: Option<i32>`.
- When `Some`, UPDATE the column.

**`local_weekday()`** — new helper:
- Uses `libc::localtime_r` (same pattern as `local_hour`).
- `tm_wday` returns 0=Sun. Map to: Mon=0, Tue=1, ..., Sun=6.
- Returns `u32`.

**`matches_day_of_week(mask: i32, weekday: u32) -> bool`** — new public function:
- `let bit = 1 << weekday;`
- `mask & bit != 0`
- mask=127 always true.

**commands.rs**:
- `add_quest`: add `days_of_week: Option<i32>` parameter, default to 127.
- `update_quest`: add `days_of_week: Option<i32>` parameter, pass through.

**Tests:**
- `matches_day_of_week` — all 7 days against single-day masks, 127 (every day), 31 (weekdays), 96 (weekends).
- `add_quest` with specific `days_of_week` — verify persisted and returned.
- `update_quest` changing `days_of_week` — verify updated.
- Existing tests updated to pass the new parameter (use `127`).

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Frontend — add form, edit form, display

**Add form** (`index.html`):
- Seven small toggle buttons in a row after the time-of-day dropdown: `M Tu W Th F Sa Su`.
- All on by default (filled). Clicking toggles individual bits.
- Hidden state tracked in a JS variable, combined into bitmask on submit.

**Edit form** (`renderEditMode`):
- Same seven toggles, pre-set from `q.days_of_week` bitmask.
- `saveEdit` reads the bitmask, passes to `invoke("update_quest", { ... daysOfWeek })`.

**Quest list display:**
- Detail row shows active days when not every day (127). Format: abbreviated day labels with inactive days omitted, e.g., "M W F" or "Sa Su".

**Testing checkpoint:** Build app. Add a quest with only weekdays — see "M Tu W Th F" in detail. Edit to weekends only — detail updates. Set all days — no days shown in detail.

---

## NOT in this step

- Quest giver filtering by day-of-week (2F-4)
- Quest list filter bar (2F-3)
- Scoring system (2F-4)
- Skip tracking (2F-5)

## Done When

Both substeps complete. Quests can be created and edited with day-of-week affinity. The bitmask is stored, returned by the API, and visible in the quest list. `cargo test` passes.
