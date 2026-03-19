# Step Spec: Phase 2F-4 — Scored Selector + Debug Mode

## Goal

Replace the current list-order quest selector with a scoring system. Quests are scored by overdue ratio, hard-filtered by time-of-day and day-of-week. Debug mode shows score breakdowns. "Something Else" still advances by index through the scored list.

---

## Substep 1: Backend — scored selector

**Replace `get_next_quest`** in `db.rs`:

New signature: `get_next_quest(conn, skip_count) -> Result<Option<ScoredQuest>, String>`

New return struct `ScoredQuest`:
```rust
pub struct ScoredQuest {
    pub quest: Quest,
    pub score: f64,
    pub overdue_ratio: f64,
    pub list_order_bonus: f64,
    pub pool: String,          // "due" or "not_due"
    pub due_count: usize,
    pub not_due_count: usize,
}
```

**Candidate pool:**
1. All active quests
2. Hard-filter: `matches_time_of_day(q.time_of_day, local_hour())`
3. Hard-filter: `matches_day_of_week(q.days_of_week, local_weekday())`
4. Split into due and not-due pools

**Scoring (due quests):**
- Overdue ratio:
  - Recurring with completions: `days_elapsed / cycle_days` (min 1.0)
  - Recurring never completed: `(days_since_created + cycle_days) / cycle_days`
  - One-off never completed: `(days_since_created + 9) / 9`
- List order bonus: `0.01 * (max_sort_order - sort_order + 1) / max_sort_order`
- Score = overdue_ratio + list_order_bonus

**Scoring (not-due fallback):**
- Only if due pool is empty
- Score = `days_since_completed / max_days_in_pool` + list_order_bonus
- Never completed = max

**`skip_count`** still used as index into scored list (same as before, but list is now score-ordered instead of sort-order)

**commands.rs:**
- Update `get_next_quest` to return `ScoredQuest`

**Tests:**
- More-overdue quest scores higher than less-overdue
- Time-of-day filter excludes out-of-window quests
- Day-of-week filter excludes off-day quests
- Empty DB returns None
- Fallback to not-due pool when no due quests
- Update existing quest selection tests to match new behavior

---

## Substep 2: Debug mode

**Migration:** Add `debug_scoring INTEGER NOT NULL DEFAULT 0` to settings.

**Settings struct update:** Add `debug_scoring: bool` to `SettingsInfo`.

**`get_settings_db`** / **`update_settings`:** Include the new field.

**Frontend — Settings tab:** New toggle for "Debug Scoring" (same style as Encounters toggle).

**Frontend — Quest giver:** When debug is on, show score breakdown below quest name:
```
Score: 2.01  (overdue: 2.0 | order: +0.01)
Pool: due | Candidates: 5 due, 3 not-due
```

---

## NOT in this step

- Skip tracking / freshness penalty (2F-5)
- Changing "Something Else" mechanism (2F-5)

## Done When

Quest giver suggests quests by overdue score. Time/day filters exclude ineligible quests. Debug mode shows score breakdown. `cargo test` passes.
