# Step Spec: Phase 4-2 — XP stats on Character tab ✅

## Goal

Character tab shows daily XP stats (Today's Score, Last Score, Avg XP/Day, High Score) and all-time earned XP. All computed from existing completion history data.

---

## Substep 1: Backend — get_xp_stats function

**New struct:**

```rust
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XpStats {
    pub today_xp: i64,
    pub last_day_xp: i64,
    pub avg_xp_per_day: f64,
    pub high_score_xp: i64,
    pub all_time_xp: i64,
}
```

**New function:** `get_xp_stats(conn) -> Result<XpStats, String>`

1. Query all completions: `SELECT completed_at, xp_earned FROM quest_completion`
2. For each, convert `completed_at` to local date using `utc_iso_to_local_days` (same logic as quest due dates)
3. Accumulate XP per local day into a HashMap<i64, i64> (day_count → total_xp)
4. Compute:
   - `today_xp`: sum for `local_today_days()`
   - `last_day_xp`: most recent non-zero day before today
   - `avg_xp_per_day`: mean across all days with XP > 0
   - `high_score_xp`: max daily total
   - `all_time_xp`: `character.xp` (from `get_character`)

**New command:** `get_xp_stats` wrapper. Register in main.rs.

**Tests:**

1. `xp_stats_today` — Complete two quests. Verify `today_xp` equals the sum of their `xp_earned`.
2. `xp_stats_high_score` — Insert completions across two different days (backdate some). Verify `high_score_xp` is the max daily total.
3. `xp_stats_avg_excludes_zero_days` — Insert completions on 2 of 3 days. Verify `avg_xp_per_day` is computed from only the 2 active days.
4. `xp_stats_last_day` — Complete quests today and backdate some to yesterday. Verify `last_day_xp` is yesterday's total (not today's).
5. `xp_stats_empty` — No completions. Verify all stats are 0.

**Testing checkpoint:** `cargo test` passes.

---

## Substep 2: Frontend — Character tab display

**Data loading:** Add `invoke("get_xp_stats")` to `loadCharacterView`'s parallel fetches. Pass to `renderCharacterView`.

**All-time XP:** Show next to character name:
```
Adventurer                        1,247 XP all-time
```

Uses `stats.allTimeXp` (or `character.xp` — same value).

**Stats row:** Below the character meter, above attributes:
```
Today: 168    Last: 602    Avg: 387/day    Best: 602
```

Compact single row. Each stat labeled. XP values formatted with commas for readability if large.

**CSS:** Stats row styled small and muted — informational, not prominent.

**Testing checkpoint:** Build app. See stats on Character tab. Complete a quest, reload Character tab — Today's Score increases. Verify all four stats make sense against your history.

---

## NOT in this step

- XP distribution on celebrations (step 3)

## Done When

Character tab shows all-time XP and daily stats row. Stats computed from completion history (including saga/campaign bonuses from step 1). `cargo test` passes.
