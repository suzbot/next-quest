# Step Spec: Phase 2H.1-3a — Saga step scoring uses saga cycle ✅

## Goal

Saga steps use their saga's `cycle_days` for overdue scoring instead of a hardcoded 9-day base. Daily saga steps score with the same urgency as daily quests. One-off sagas continue using 9.

---

## Substep 1: Backend — tuple change + formula update

**`get_active_saga_steps`** — change return type from `Vec<(Quest, String, String)>` to `Vec<(Quest, String, String, Option<i32>)>`, adding the saga's `cycle_days` as the fourth element. The saga is already loaded in this function — just include `saga.cycle_days` in the tuple.

Update the push:
```rust
results.push((step.clone(), saga.name.clone(), prev_completed_at.clone(), saga.cycle_days));
```

**`get_next_quest`** — update all references to the saga step tuple throughout the function:

1. Type annotation for `eligible_saga` filter: `Vec<&(Quest, String, String, Option<i32>)>`
2. Destructuring in the scoring loop: `for (quest, saga_name, activated_at, saga_cycle_days) in &eligible_saga`
3. Overdue ratio calculation:
   ```rust
   let saga_cycle = saga_cycle_days.unwrap_or(9) as f64;
   let overdue_ratio = (days_since + saga_cycle) / saga_cycle;
   ```
4. The final `scored.push` and `top` destructuring — add the extra tuple element

**Tests:**

1. `saga_step_scoring_daily_cycle` — Create a daily saga (cycle_days=1) with one step. Backdate the saga's `created_at` to 1 day ago. Call `get_next_quest` with no other quests due. Verify the returned quest is the saga step and its overdue_ratio is ~2.0 (not ~1.11).

2. `saga_step_scoring_weekly_cycle` — Create a weekly saga (cycle_days=7) with one step. Backdate to 1 day ago. Verify overdue_ratio is ~(1+7)/7 = ~1.14.

3. `saga_step_scoring_oneoff_uses_nine` — Create a one-off saga (cycle_days=None) with one step. Backdate to 1 day ago. Verify overdue_ratio is ~(1+9)/9 = ~1.11 (unchanged behavior).

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## NOT in this step

- Importance field (3b)
- List-order weight (3c)
- Saga/campaign membership (3d)
- Attribute/skill balancing (3e)

## Done When

Saga step overdue ratio uses `(days + saga_cycle) / saga_cycle`. Daily saga steps surface with the same urgency as daily quests. One-off sagas unchanged. `cargo test` passes.
