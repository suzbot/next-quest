# Lane Refactor — Design

**Status:** Draft
**Requirements:** [lane-refactor-requirements.md](lane-refactor-requirements.md)

---

## Scope

Two places in `nq-core/src/db.rs` encapsulate lane eligibility:

1. **Regular quest filter** — `Lane::includes_difficulty()` called from `get_quest_scores()` (line 2212)
2. **Saga lane inference** — `get_active_saga_steps()` infers a saga's lane from its hardest step (line 1512-1513)

Both need new logic that incorporates `cycle_days`. Everything else (scoring, overlay, saga selection, etc.) stays the same because it already receives the lane as an input — it doesn't need to know how the lane was decided.

---

## Design

### Quest lane eligibility

Replace `Lane::includes_difficulty(&Difficulty)` with `Lane::includes(&Difficulty, Option<i32>)`, where the second argument is `cycle_days`.

```rust
impl Lane {
    fn includes(&self, difficulty: &Difficulty, cycle_days: Option<i32>) -> bool {
        let is_daily = cycle_days == Some(1);
        match self {
            Lane::CastleDuties => is_daily,
            Lane::Adventures => !is_daily && matches!(difficulty, Difficulty::Trivial | Difficulty::Easy),
            Lane::RoyalQuests => !is_daily && matches!(difficulty, Difficulty::Moderate | Difficulty::Challenging | Difficulty::Epic),
        }
    }
}
```

Update the call site in `get_quest_scores`:

```rust
// before
.filter(|q| lane.includes_difficulty(&q.difficulty))
// after
.filter(|q| lane.includes(&q.difficulty, q.cycle_days))
```

### Saga lane inference

`get_active_saga_steps()` currently computes:

```rust
let max_rank = steps.iter().map(|s| difficulty_rank(&s.difficulty)).max().unwrap_or(1);
let saga_lane = lane_for_difficulty_rank(max_rank);
```

New logic:

```rust
let saga_lane = if saga.cycle_days == Some(1) {
    Lane::CastleDuties
} else {
    let max_rank = steps.iter().map(|s| difficulty_rank(&s.difficulty)).max().unwrap_or(1);
    lane_for_difficulty_rank(max_rank)
};
```

`lane_for_difficulty_rank()` stays — it still handles the non-daily case, mapping trivial/easy → Adventures and moderate+ → Royal Quests. No changes needed to that helper.

### The old `includes_difficulty` function

Remove it entirely. The one call site moves to `includes()`. No other references.

### Saga step cycle_days

Saga steps have `cycle_days = None` (they use the saga's cycle, not their own). The new quest filter uses `q.cycle_days`, which for a saga step would be `None` — meaning `is_daily = false`, so the step wouldn't match Lane 1 under the quest filter path. That's fine — saga steps don't go through the quest filter at all, they go through the saga lane path.

---

## Tests to Update

### Existing tests that assert lane behavior

Let me enumerate from the db.rs test module:

1. **`lane_filter_trivial_only`** — asserts that only trivial quests are in Castle Duties. Under new rules, needs to assert that only daily quests are in Castle Duties, regardless of difficulty.

2. **`lane_filter_adventures`** — asserts only easy quests in Adventures. New rule: trivial or easy, non-daily.

3. **`lane_filter_royal`** — asserts moderate+ in Royal Quests. New rule: moderate+, non-daily.

4. **`saga_lane_inference_from_hardest_step`** — still valid for non-daily sagas, but should stay as-is (verifies the difficulty-based inference still works when the saga isn't daily).

5. **`saga_with_only_easy_steps_in_adventures`** — still valid for non-daily sagas.

### New tests to add

1. **`lane_daily_moderate_goes_to_castle_duties`** — a moderate daily quest lands in Lane 1, not Lane 3.
2. **`lane_daily_epic_goes_to_castle_duties`** — an epic daily quest lands in Lane 1, not Lane 3.
3. **`lane_weekly_trivial_goes_to_adventures`** — a trivial weekly quest lands in Lane 2, not Lane 1.
4. **`lane_one_off_trivial_goes_to_adventures`** — a one-off trivial quest lands in Lane 2.
5. **`saga_daily_recurring_goes_to_castle_duties`** — a saga with `cycle_days == 1` and epic steps lands in Lane 1.
6. **`saga_weekly_recurring_uses_difficulty`** — a saga with `cycle_days == 7` and trivial steps still uses difficulty → Lane 2.

---

## Documentation Updates

Three docs reference the old rules:

1. **`docs/mechanics.md`** — "Lanes" section (under Quest Giver) has a table mapping difficulty → lane. Replace with the new decision table from the requirements doc.

2. **`DATA_MODEL.md`** — "Lanes" subsection (under Quest Selector) currently says:
   > | 1 | Castle Duties | Trivial |
   > | 2 | Adventures | Easy |
   > | 3 | Royal Quests | Moderate, Challenging, Epic |

   Replace with the new rules.

   Also the "Saga steps appear in the lane matching their saga's hardest step difficulty" sentence needs to be updated to reflect the daily-takes-precedence rule.

3. **`VISION.md`** — "Phase 3: The Three Quest Givers" section describes the lanes. Needs to be updated to reflect the new lane definitions.

---

## Implementation Steps

This is one vertical slice — small change, highly interconnected.

### Step 1: Refactor and update tests

1. Replace `Lane::includes_difficulty()` with `Lane::includes()` that takes difficulty + cycle_days
2. Update the call site in `get_quest_scores()`
3. Update the saga lane inference in `get_active_saga_steps()` to check for daily recurring first
4. Update the three lane filter tests to match new rules
5. Add six new tests for the new behavior
6. Update mechanics.md, DATA_MODEL.md, VISION.md

**Verify:** All tests pass. Launch the app and spot-check: daily quests (any difficulty) show up in Castle Duties, weekly quests go to Adventures or Royal Quests based on difficulty, daily sagas go to Castle Duties.

---

## Summary

Two-line logical change (quest filter + saga lane inference), but it propagates through tests and docs. No migration needed because lane is computed at query time. The existing scoring, overlay, collapse state, and lane themes all stay untouched — they just receive a different set of eligible quests.

One implementation step, fully testable in isolation.
