# Step Spec: Debug scoring on quest list rows

## Goal

When debug scoring is enabled in settings, expanded quest list rows show their score breakdown (same components as the quest giver debug display).

---

## Substep 1: Backend — `get_quest_scores` command

Extract the scoring logic from `get_next_quest` into a reusable helper that returns all scored quests, not just the top one.

**New function:** `get_quest_scores(conn, skip_counts) -> Result<Vec<ScoredQuest>, String>`

Reuses the same logic as `get_next_quest`: loads quests, applies hard filters, scores due + saga + not-due pools, but returns ALL scored entries instead of picking the top one. No `exclude_quest_id` parameter needed.

**Implementation approach:** Extract the scoring body of `get_next_quest` into `get_quest_scores`. Then `get_next_quest` calls `get_quest_scores`, sorts, and picks the top.

**New command:** `get_quest_scores` in commands.rs, reads skip state same as `get_next_quest`. Register in main.rs.

**Tests:** Existing `get_next_quest` tests continue to pass (it delegates to the new function).

**Testing checkpoint:** `cargo test` passes.

---

## Substep 2: Frontend — show scores on expanded quest rows

**Load scores:** When debug mode is on and the quest list is displayed, call `get_quest_scores` and store as a map (`questId → ScoredQuest`).

**Display:** When a quest row is expanded (detail section visible), show the score breakdown below the tags line:

```
Score: 32.50 (overdue: 2.00 | importance: +30.0 | order: +0.50 | member: +0.0 | skips: -0.0) [due]
```

If the quest has no score (not in the eligible pool — e.g., filtered by TOD/DOW), show nothing.

**When to refresh:** Scores are fetched alongside `loadAll`. Only fetched when `debugScoring` is true.

**Testing checkpoint:** Build app. Enable debug scoring. Expand a quest — see score breakdown. Disable debug — breakdown disappears.

---

## Done When

Both substeps complete. Debug scoring shows per-quest breakdown on expanded list rows. `cargo test` passes.
