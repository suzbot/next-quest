# Lane Refactor: Cadence + Difficulty

**Goal:** Shift lane assignment so daily-recurring quests (any difficulty) live in Lane 1, and non-daily quests split by difficulty into Lanes 2 and 3.

**Requirements:** [lane-refactor-requirements.md](lane-refactor-requirements.md)
**Design:** [lane-refactor-design.md](lane-refactor-design.md)

---

## Gaps Caught During Review

1. **`lane_for_difficulty_rank` needs updating.** Under the old rules, rank 1 (trivial) mapped to `CastleDuties`. Under the new rules, non-daily trivial sagas go to Adventures. This function is only called from the non-daily branch of `get_active_saga_steps`, so we can fix it directly — Castle Duties is determined by cadence now, not by rank.

2. **`get_quests()` already filters `saga_id IS NULL`.** Saga steps never flow through the quest filter path, so we don't need to worry about saga steps matching wrong lanes via the new `Lane::includes()` check. Saga lane assignment stays exclusively in `get_active_saga_steps`.

3. **Saga one-off with trivial-only steps.** Under the old rules, a one-off saga with trivial steps would have gone to Castle Duties. Under the new rules, one-off means non-daily, and trivial means Adventures — a noticeable shift. Worth a dedicated test.

4. **Existing quest data redistributes automatically.** Lane is computed at query time, no stored lane field. The user's current trivial weekly quests will move from Lane 1 to Lane 2 on first launch — that's the intended effect, not a bug.

---

## Changes

### 1. `Lane::includes()` — new method with cadence + difficulty

Replace `Lane::includes_difficulty(&Difficulty)` with `Lane::includes(&Difficulty, Option<i32>)`:

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

Delete `includes_difficulty`.

### 2. Update the call site in `get_quest_scores`

```rust
// before
.filter(|q| lane.includes_difficulty(&q.difficulty))
// after
.filter(|q| lane.includes(&q.difficulty, q.cycle_days))
```

### 3. Update `lane_for_difficulty_rank`

Under the new rules, this function is only called for non-daily sagas. It should never return `CastleDuties` — that's decided by cadence in the caller.

```rust
fn lane_for_difficulty_rank(rank: u8) -> Lane {
    match rank {
        3 | 4 | 5 => Lane::RoyalQuests,
        _ => Lane::Adventures,  // 1 (trivial) or 2 (easy)
    }
}
```

### 4. Update `get_active_saga_steps` to check cadence first

```rust
// before
let max_rank = steps.iter().map(|s| difficulty_rank(&s.difficulty)).max().unwrap_or(1);
let saga_lane = lane_for_difficulty_rank(max_rank);

// after
let saga_lane = if saga.cycle_days == Some(1) {
    Lane::CastleDuties
} else {
    let max_rank = steps.iter().map(|s| difficulty_rank(&s.difficulty)).max().unwrap_or(1);
    lane_for_difficulty_rank(max_rank)
};
```

### 5. Update existing tests to match new rules

Three tests assert the old difficulty-based lane rules and need updating:

- **`lane_filter_trivial_only`** — now should assert that Castle Duties contains daily quests (any difficulty) and excludes non-daily quests
- **`lane_filter_adventures`** — now should assert Adventures contains non-daily trivial/easy and excludes daily quests
- **`lane_filter_royal`** — now should assert Royal Quests contains non-daily moderate+ and excludes daily quests

Existing saga tests to check:

- **`saga_lane_inference_from_hardest_step`** — ensure the test uses a non-daily saga (cycle_days != Some(1)). If it uses a daily saga, update to a non-daily cycle. Still a valid test for the difficulty-based fallback path.
- **`saga_with_only_easy_steps_in_adventures`** — same check. Must be non-daily. Under new rules, trivial-only non-daily sagas also go to Adventures, which is a behavior change this test doesn't cover alone.

### 6. Add new tests

```rust
#[test]
fn lane_daily_moderate_in_castle_duties() {
    // Moderate quest with cycle_days=1 appears in Castle Duties, not Royal Quests
}

#[test]
fn lane_daily_epic_in_castle_duties() {
    // Epic quest with cycle_days=1 appears in Castle Duties, not Royal Quests
}

#[test]
fn lane_daily_trivial_still_in_castle_duties() {
    // Trivial daily still in Castle Duties — regression guard
}

#[test]
fn lane_weekly_trivial_in_adventures() {
    // Trivial quest with cycle_days=7 appears in Adventures, not Castle Duties
}

#[test]
fn lane_one_off_trivial_in_adventures() {
    // One-off trivial appears in Adventures (not Castle Duties)
}

#[test]
fn lane_weekly_moderate_still_in_royal_quests() {
    // Regression guard for non-daily moderate
}

#[test]
fn saga_daily_recurring_in_castle_duties() {
    // Saga with cycle_days=Some(1) and epic steps appears in Castle Duties
}

#[test]
fn saga_one_off_trivial_steps_in_adventures() {
    // Non-daily saga (cycle_days=None) with only trivial steps goes to Adventures
    // under new rules (would have been Castle Duties before)
}

#[test]
fn saga_weekly_moderate_steps_in_royal_quests() {
    // Non-daily saga with moderate steps still goes to Royal Quests
}
```

### 7. Documentation updates

**`docs/mechanics.md`** — "Lanes" section. Replace the existing table with the new decision table from requirements. Also update the "Lane Assignment" subsection under Sagas to reflect cadence-first rule.

**`DATA_MODEL.md`** — "Quest Selector" > "Lanes" subsection. Replace the lane table. Update the "Saga steps appear in the lane matching their saga's hardest step difficulty" sentence to reflect the new rule (daily sagas in Castle Duties, otherwise by hardest step).

**`VISION.md`** — "Phase 3: The Three Quest Givers" section. Update the lane descriptions. Castle Duties is now "daily rhythm" rather than "trivial routine."

---

## Verification

1. **Tests pass:** `cargo test` — all existing tests updated, new tests pass.
2. **Manual check — regular quests:**
   - Create (or find) a daily moderate quest. Confirm it shows in Castle Duties.
   - Create a weekly trivial quest. Confirm it shows in Adventures.
   - Create a one-off trivial quest. Confirm it shows in Adventures.
   - Create a weekly epic quest. Confirm it stays in Royal Quests.
3. **Manual check — sagas:**
   - Find a one-off saga with all trivial steps. Confirm it shows in Adventures (previously Castle Duties).
   - Find a one-off saga with a moderate+ step. Confirm it stays in Royal Quests.
4. **Encounters overlay:** Enable Call to Adventure. Confirm it surfaces Lane 1 quests (which now may include any-difficulty daily quests).
5. **Existing quests:** Launch the app. Current daily quests redistribute into Lane 1; current weekly/one-off quests distribute into Lanes 2 and 3 by difficulty. Nothing should be missing from the lists — everything should just be in a different lane.
