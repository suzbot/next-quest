# Step Spec: Phase 2H.1-3f — Skip penalty rework

## Goal

Replace the subtractive skip penalty with a divisive model for importance. Instead of subtracting `skips × (0.5 + importance × weight/2)`, divide the importance boost by `(1 + skips)` and keep a small base penalty for 0! quests.

---

## Substep 1: Backend — formula change in all three scoring paths

**Old formula:**
```
importance_boost = importance × IMPORTANCE_WEIGHT
skip_penalty = skips × (0.5 + importance × IMPORTANCE_WEIGHT / 2.0)
score = overdue + importance_boost - skip_penalty + order + membership + balance
```

**New formula:**
```
importance_boost = importance × IMPORTANCE_WEIGHT / (1.0 + skips)
skip_penalty = skips × 0.5
score = overdue + importance_boost - skip_penalty + order + membership + balance
```

The `importance_boost` field in `ScoredQuest` now reflects the skip-adjusted value (what the quest actually got, not the raw importance). This keeps the debug display honest — you can see how skips eroded importance.

**Three locations to update:**
1. `score_quests_due`
2. `score_quests_not_due`
3. Saga step scoring block in `get_quest_scores`

**Update existing skip penalty tests:**
- `skip_penalty_scales_with_importance` — now skip_penalty is 0.5 regardless of importance. Importance boost is halved instead. Update assertions.
- `skip_penalty_zero_importance` — unchanged (0.5 base still works).

**New test:**
1. `skip_diminishes_importance_not_craters` — Create a 5! quest. Skip it 3 times. Verify score is still positive. Verify importance_boost ≈ 150 / 4 = 37.5. Verify skip_penalty = 1.5.

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Done When

Skip penalty uses divisive model for importance. High-importance quests diminish gracefully with skips instead of cratering into negatives. `cargo test` passes.
