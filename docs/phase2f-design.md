# Phase 2F: "Logic Enhancement" — Design

## Overview

Five features: two new quest attributes (time-of-day, day-of-week), a scoring-based quest selector with overdue escalation and skip-freshness tracking, manual quest list filtering, and a debug mode for tuning. The implementation order puts filtering before the scoring system so the user can inspect the candidate pool while testing the new selector.

## 1. Schema Changes

### New columns on `quest`

| Column | Type | Default | Description |
|---|---|---|---|
| `time_of_day` | TEXT | `'anytime'` | One of: `anytime`, `morning`, `afternoon`, `evening` |
| `days_of_week` | INTEGER | `127` | Bitmask: Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64. 127 = every day |

### New column on `settings`

| Column | Type | Default | Description |
|---|---|---|---|
| `debug_scoring` | INTEGER | `0` | 0 = off, 1 = show score breakdown in quest giver |

### Migration

Single migration adding the three columns with defaults. Existing quests get `anytime` and `127` (every day), preserving current behavior. No data migration needed — the defaults mean nothing changes until the user edits a quest.

## 2. Time-of-Day Windows

### Window definitions (local time)

| Window | Start | End |
|---|---|---|
| Morning | 4:00 AM | 11:59 AM |
| Afternoon | 12:00 PM | 4:59 PM |
| Evening | 5:00 PM | 3:59 AM |
| Anytime | — | — |

### Implementation

A Rust function `matches_time_of_day(window: &str, local_hour: u32) -> bool`:

```
anytime   → true
morning   → hour >= 4 && hour < 12
afternoon → hour >= 12 && hour < 17
evening   → hour >= 17 || hour < 4
```

Evening wraps past midnight. The existing `local_today()` infrastructure in `db.rs` already handles local time via `libc`; we add a `local_hour()` helper using the same approach.

### Quest add/edit form

Dropdown with four options: Anytime, Morning, Afternoon, Evening. Defaults to Anytime. Shown alongside existing Difficulty and Cycle fields.

### Quest list display

Collapsed row doesn't change (already tight). Expanded detail row shows time-of-day if not Anytime (e.g., "Morning").

## 3. Day-of-Week Affinity

### Bitmask encoding

```
Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64
Every day = 127 (all bits set)
Weekdays = 31 (Mon–Fri)
Weekends = 96 (Sat–Sun)
```

### Implementation

A Rust function `matches_day_of_week(mask: i32, local_weekday: u32) -> bool`:

```rust
let bit = 1 << local_weekday; // 0=Mon, 6=Sun
mask & bit != 0
```

`local_weekday()` derived from the same `libc` local time infrastructure. `tm_wday` returns 0=Sun, so we map: Sun(0)→6, Mon(1)→0, Tue(2)→1, etc.

### Quest add/edit form

Seven toggle buttons in a row: `M Tu W Th F Sa Su`. Each toggles its bit. All on by default. Visual state: filled = active, outline = inactive.

The UI should make it easy to set common patterns — clicking any day when all are selected should flip to "only that day" (clear others). But individual toggling is the primary interaction. No preset buttons needed.

### Quest list display

Expanded detail row shows active days if not every day, using the abbreviated format from requirements: `M Tu W Th F Sa Su` with inactive days dimmed or omitted.

## 4. Skip Tracking (Freshness)

### In-memory state

Stored in Tauri managed state alongside the existing timer state:

```rust
pub struct SkipState {
    pub skip_counts: HashMap<String, i32>,  // quest_id → skip count
    pub reset_date: String,                  // ISO date of last reset ("2026-03-17")
}
```

### Reset logic

On every `get_next_quest` call, compare `reset_date` to `local_today()`. If different, clear the map and update the date.

### Skip recording

New Tauri command: `skip_quest(quest_id: String)`. Increments the count for that quest in the map. Called by the frontend when the user clicks Something Else (Next Quest tab) or Run (overlay). Hide in the Shadows does NOT record a skip — it's "not right now," not a rejection of the quest.

### Why in-memory, not DB

Skip counts are ephemeral — they reset daily and losing them on app restart is fine (fresh start). No migration, no cleanup, no stale data. The app already holds timer state in memory the same way.

## 5. Quest Selector Scoring System

### Replacing `get_next_quest`

The current function filters to due quests, picks by list index with `skip_count` rotation. The new version scores all eligible quests and returns the highest-scored one.

### Candidate pool construction

```
1. All active quests
2. Remove: time_of_day doesn't match current local hour
3. Remove: days_of_week bitmask doesn't include today
4. Split into DUE and NOT_DUE pools
```

### Scoring formula (due quests)

```
score = overdue_ratio - (skip_count × 0.5) + list_order_bonus
```

**Overdue ratio** (`days_since_last_completed / cycle_days`):
- Recurring, has completions: `days_elapsed / cycle_days`. Minimum 1.0 (they're due).
- Recurring, never completed: `(days_since_created + cycle_days) / cycle_days`. Gives new quests a head start proportional to cycle.
- One-off, never completed: `(days_since_created + 9) / 9`. Consistent with the one-off XP cycle equivalent of 9 days.

**Skip penalty** (`skip_count × 0.5`):
- Each skip subtracts 0.5 from the score.
- A just-due quest (ratio 1.0) sinks to ≤0 after 2 skips.
- A 3x-overdue quest takes 6 skips to reach 0.
- Quests can go negative — they're still eligible via exhaustion fallback.

**List order bonus** (`0.01 × normalized_position`):
- `(max_sort_order - sort_order + 1) / max_sort_order × 0.01`
- Range: 0.0–0.01. Only breaks ties.

### Scoring formula (not-due fallback)

Only reached when the due pool is empty or all due quests have been skipped past (all scores ≤ 0).

```
score = days_since_completed / max_days_in_pool - (skip_count × 0.5) + list_order_bonus
```

Longest-ago-completed gets ~1.0. Same skip penalty. Same tiebreaker.

### "Something Else" flow

1. Frontend calls `skip_quest(current_quest_id)`
2. Frontend calls `get_next_quest()` (no more `skip_count` parameter)
3. Backend scores all candidates, returns the highest

If the returned quest is the same one (it had the highest score even after the skip increment), the frontend shows it again. This can happen if a quest is extremely overdue — that's correct behavior, the quest is insisting.

### Exhaustion fallback

If all candidates score ≤ 0 in the due pool, include the not-due pool. If all candidates across both pools score ≤ 0, return the highest-scored anyway (the least-negative). The app always has something to give.

### Reference scoring table

| Scenario | Overdue ratio | 0 skips | 1 skip | 2 skips | 3 skips |
|---|---|---|---|---|---|
| Just due (1x cycle) | 1.0 | 1.0 | 0.5 | 0.0 | -0.5 |
| 2x overdue | 2.0 | 2.0 | 1.5 | 1.0 | 0.5 |
| 3x overdue | 3.0 | 3.0 | 2.5 | 2.0 | 1.5 |
| 7x overdue | 7.0 | 7.0 | 6.5 | 6.0 | 5.5 |
| New recurring (1-day cycle, created today) | 2.0 | 2.0 | 1.5 | 1.0 | 0.5 |
| New one-off (created today) | 1.0 | 1.0 | 0.5 | 0.0 | -0.5 |
| New one-off (created 9 days ago) | 2.0 | 2.0 | 1.5 | 1.0 | 0.5 |

## 6. Debug Scoring Display

### Settings toggle

New toggle on the Settings tab: "Debug Scoring" with the same on/off toggle style as Encounters. Persisted in the `debug_scoring` settings column.

### Quest giver display

When debug is on, the Next Quest view shows a score breakdown below the quest name:

```
Score: 2.50  (overdue: 3.0 | skips: -0.5 | order: +0.01)
Pool: due | Candidates: 8 due, 4 not-due
```

Small, monospace, muted text — informational, not part of the game aesthetic. Hidden when debug is off.

### Overlay

Debug info not shown on the overlay — it's meant to be glanceable. Debug inspection happens in the main app.

## 7. Manual Quest List Filtering

### Filter bar

A row of controls above the quest list:

```
[Attribute ▾] [Skill ▾] [Time ▾] [Day ▾] [☐ Due only] [Clear]
```

- **Attribute dropdown**: "All" + list of user's attributes
- **Skill dropdown**: "All" + list of user's skills
- **Time of Day dropdown**: "All", Morning, Afternoon, Evening, Anytime
- **Day of Week dropdown**: "All", today's day, plus individual days
- **Due only checkbox**: when checked, shows only quests where `is_due` is true
- **Clear button**: resets all filters to default

### Implementation

Frontend-only JavaScript filtering. The quest list data is already fully loaded. On any filter change, re-render the list showing only quests that pass all active filters. Filters are AND-combined.

No backend changes needed for filtering.

### Session-only

Filter state is held in JS variables. Not persisted — resets on app restart or tab switch. Simple and low-stakes.

### Interaction with time-of-day and day-of-week

Filtering by Time = "Morning" shows quests whose `time_of_day` is `morning` or `anytime`. Filtering by Day = "Monday" shows quests whose `days_of_week` bitmask includes Monday, or is 127 (every day). This mirrors what the quest giver's candidate pool would look like.

## Implementation Order

Ordered so that quest attributes and filtering are available before the scoring system, giving the user visibility into the candidate pool during testing:

1. **2F-1: Time-of-day** — Migration for `time_of_day` column, `local_hour()` helper, `matches_time_of_day()`, add/edit form dropdown, expanded detail row display. Tests for window matching logic.

2. **2F-2: Day-of-week** — Migration for `days_of_week` column, bitmask helpers, `matches_day_of_week()`, toggle buttons on add/edit form, expanded detail row display. Tests for day matching logic.

3. **2F-3: Quest list filtering** — Filter bar UI, frontend-only filtering logic. No backend changes. Lets the user see which quests would be eligible for a given time/day before the scorer exists.

4. **2F-4: Scored selector + debug mode** — Replace `get_next_quest` with scoring based on overdue ratio, candidate pool hard filters (time-of-day + day-of-week), list order tiebreaker. Debug scoring settings toggle + score breakdown display in quest giver. "Something Else" still advances through the scored list by index (same UX, new ordering).

5. **2F-5: Skip tracking + freshness** — In-memory skip state, `skip_quest` command, freshness penalty in scoring formula, exhaustion fallback. "Something Else" and Run now record a skip and re-score instead of index-advancing. Debug display updated to show skip counts. Hide in the Shadows does NOT count as a skip (it's "not right now", not "not this").
