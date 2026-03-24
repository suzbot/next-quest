# Step Spec: Phase 2H.1-2 — Evening/Night time-of-day split ✅

## Goal

Split the Evening time window (5pm–4am) into Evening (5pm–9pm) and Night (9pm–4am). The bitmask model goes from 3-bit to 4-bit. Existing quests with Evening set get both Evening and Night.

---

## Substep 1: Backend — migration, matches_time_of_day, defaults

**Migration** (`db.rs` → `migrate()`):

Detection: check if any quest has `time_of_day > 7` (new bitmask values use bit 8). If none do, run migration.

```sql
-- Quests with old evening bit (4) get night bit (8) added
UPDATE quest SET time_of_day = time_of_day | 8 WHERE (time_of_day & 4) != 0;
-- Old "all times" mask 7 → new "all times" mask 15
UPDATE quest SET time_of_day = 15 WHERE time_of_day = 7;
```

Note: mask 0 continues to mean "all times" — no migration needed for those.

**`matches_time_of_day` update:**

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

Keep: mask 0 or 15 = all times.

**Default changes:**

- `default_time_of_day()` → return 15 (was 7)
- `NewQuest::default()` → `time_of_day: 15`
- `NewSagaStep` default → 15

**Tests:**

1. `matches_time_of_day` at boundary hours: 3→night(8), 4→morning(1), 11→morning(1), 12→afternoon(2), 16→afternoon(2), 17→evening(4), 20→evening(4), 21→night(8)
2. Mask 15 matches all hours
3. Mask 0 matches all hours
4. Mask 4 (evening only) matches hour 17 but NOT hour 21
5. Mask 8 (night only) matches hour 21 but NOT hour 17
6. Mask 12 (evening+night) matches both 17 and 21

Update existing `matches_time_of_day` tests to use the new boundaries.

**Testing checkpoint:** `cargo test` — all existing + new/updated tests pass.

---

## Substep 2: Frontend — multiselect and display updates

**Time-of-day constants** — add Night option to all TOD-related arrays:

- `TOD_FULL` gains `{ value: 8, label: "NT" }` (or equivalent)
- `todSummary` / `todText` updated for 4 values
- "All" threshold changes from checking mask 7 to checking mask 15 (or mask 0)

**Affected UI locations:**

1. Quest add form — TOD multiselect gains NT
2. Quest edit mode — TOD multiselect gains NT
3. Saga step add form — TOD multiselect gains NT
4. Saga step edit mode — TOD multiselect gains NT
5. Quest list filter bar — TOD filter gains NT
6. Quest list display — `todText` shows EV/NT correctly

**Testing checkpoint:** Build app. Verify new quests default to all 4 windows selected. Create a quest with only EV selected — verify it doesn't appear in quest giver during night hours. Edit an existing quest — verify it shows EV+NT (migrated from old Evening). Filter by NT — works correctly.

---

## NOT in this step

- Quest selector tuning (2H.1-3 through 2H.1-7)

## Done When

Both substeps complete. 4-window time model (MO/AF/EV/NT). Existing quests migrated. Quest selector hard-filters correctly. `cargo test` passes.
