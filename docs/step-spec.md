# Step Spec: Phase 2H.1-3c — List-order weight increase + importance-scaled skip penalty ✅

## Goal

List-order bonus increases from negligible (0.01 max) to meaningful (1.0 max), using the full quest list's max sort_order. Skip penalty scales with importance so skipping high-importance quests has proportional teeth.

---

## Substep 1: Backend — list-order weight + skip penalty

**List-order weight:**

`score_quests_due` and `score_quests_not_due` currently compute `max_sort` from their input slice (candidate pool). Change to accept a `global_max_sort: f64` parameter.

In `get_next_quest`, before scoring:
```rust
let global_max_sort = quests.iter().map(|q| q.sort_order).max().unwrap_or(1) as f64;
```

Pass to both scoring functions. Update the formula:
```rust
let list_order_bonus = q.sort_order as f64 / global_max_sort;
```

Max bonus is now 1.0 (was 0.01). Saga steps continue using a fixed 0.1.

**Skip penalty:**

Currently `skips × 0.5`. Change to scale with importance:
```rust
const IMPORTANCE_WEIGHT: f64 = 30.0;
let skip_penalty = skips * (0.5 + q.importance as f64 * IMPORTANCE_WEIGHT / 2.0);
```

- 0! quest: 0.5 per skip (unchanged)
- 1! quest: 15.5 per skip
- 2! quest: 30.5 per skip
- 5! quest: 75.5 per skip

Apply in all three scoring paths: `score_quests_due`, `score_quests_not_due`, and saga step scoring in `get_next_quest`.

Extract `IMPORTANCE_WEIGHT` as a constant at the top of the scoring section so both importance_boost and skip_penalty reference it.

**Debug display:** Already shows `list_order_bonus` and `skip_penalty` — values will just change. No struct changes needed.

**Tests:**

1. `list_order_bonus_uses_global_max` — Create 3 quests (sort_order 1, 2, 3). Make only the bottom two due. Verify the list_order_bonus for sort_order 2 is `2/3 ≈ 0.67`, not `2/2 = 1.0`.
2. `top_of_list_gets_max_bonus` — Quest at max sort_order gets list_order_bonus ≈ 1.0.
3. `skip_penalty_scales_with_importance` — Create a quest with importance 2. Skip it once. Verify skip_penalty is `0.5 + 2 × 15.0 = 30.5`.
4. `skip_penalty_zero_importance` — Create a 0! quest. Skip once. Verify skip_penalty is 0.5 (unchanged behavior).

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## NOT in this step

- Saga/campaign membership bonus (3d)
- Attribute/skill balancing (3e)

## Done When

List-order bonus uses global max sort_order with max 1.0. Skip penalty scales with importance (0.5 base + importance × IMPORTANCE_WEIGHT/2 per skip). `cargo test` passes.
