# Step Spec: Phase 2H.1-3d — Campaign membership bonus ✅

## Goal

Regular quests that are criteria in active campaigns get a +1.0 scoring bonus. Boolean — does not stack across multiple campaigns. Saga steps do not get this bonus (they already have 1.0 from list-order priority).

---

## Substep 1: Backend — precompute campaign quest set + scoring

**Precompute campaign quest IDs** at the start of `get_next_quest`, after loading quests:

```rust
let campaign_quest_ids: std::collections::HashSet<String> = {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT cc.target_id FROM campaign_criterion cc
         JOIN campaign c ON c.id = cc.campaign_id
         WHERE c.completed_at IS NULL AND cc.target_type = 'quest_completions'"
    ).map_err(|e| e.to_string())?;
    stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect()
};
```

**Pass to scoring functions:** `score_quests_due` and `score_quests_not_due` accept `&HashSet<String>` parameter.

**Scoring formula update:**

```rust
let membership_bonus = if campaign_quest_ids.contains(&q.id) { 1.0 } else { 0.0 };
let score = overdue_ratio + importance_boost + list_order_bonus + membership_bonus - skip_penalty;
```

Saga step scoring in `get_next_quest` does NOT add membership_bonus (they already have 1.0 list-order). Set `membership_bonus = 0.0` for saga steps.

**Scored tuple:** Grows by one f64 for membership_bonus. Update type from `(f64, f64, f64, f64, f64, Quest, Option<String>)` to `(f64, f64, f64, f64, f64, f64, Quest, Option<String>)` — score, overdue, importance, skip, order, membership, quest, saga_name.

**`ScoredQuest` struct:** Add `membership_bonus: f64`. Update construction and destructuring.

**Debug display:** Add membership component:
```
Score: ... | member: +1.0 | ...
```

**Tests:**

1. `campaign_membership_boosts_score` — Create two equivalent quests. Create an active campaign referencing one. Verify it scores 1.0 higher.
2. `campaign_membership_no_stacking` — Quest referenced in two active campaigns. Verify it still gets only 1.0, not 2.0.
3. `completed_campaign_no_bonus` — Complete a campaign (stamps completed_at). Verify its quest criterion no longer gets the bonus.
4. `saga_step_no_membership_bonus` — Create a saga step referenced in a campaign. Verify its membership_bonus is 0.0 (it gets 1.0 from list-order instead).

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## NOT in this step

- Attribute/skill balancing (3e)

## Done When

Regular quests in active campaigns get +1.0 membership bonus. No stacking. Saga steps excluded. Debug scoring shows the component. `cargo test` passes.
