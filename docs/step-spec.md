# Step Spec: Phase 4-1 — Bonus XP in completion history ✅

## Goal

Saga completion bonuses and campaign completion bonuses appear in the quest completion history. This ensures they show up in the history list on the Character tab and are included in daily XP stats (built in step 2).

---

## Substep 1: Backend — insert completion records for bonuses

**`check_saga_completion`** (`db.rs`): After awarding bonus XP and detecting level-ups, insert a quest_completion record:

```rust
let completion_id = Uuid::new_v4().to_string();
conn.execute(
    "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at, xp_earned) VALUES (?1, NULL, ?2, ?3, ?4)",
    rusqlite::params![completion_id, format!("{} complete!", saga.name), now, bonus_xp],
).map_err(|e| e.to_string())?;
```

Only insert if `bonus_xp > 0`. Uses `quest_id: NULL` (same pattern as orphaned completions). Title format: "Morning Routine complete!" (saga name + " complete!").

**`check_campaign_progress`** (`db.rs`): Same pattern, after awarding bonus XP and creating the accomplishment record:

```rust
let completion_id = Uuid::new_v4().to_string();
conn.execute(
    "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at, xp_earned) VALUES (?1, NULL, ?2, ?3, ?4)",
    rusqlite::params![completion_id, format!("{} complete!", campaign_name), now, bonus_xp],
).map_err(|e| e.to_string())?;
```

Only insert if `bonus_xp > 0`.

**Tests:**

1. `saga_completion_creates_history_entry` — Create a saga, complete all steps to trigger saga completion. Verify a quest_completion record exists with `quest_id: NULL`, title containing the saga name + "complete!", and `xp_earned` equal to the bonus.

2. `campaign_completion_creates_history_entry` — Create a campaign, complete criteria to trigger campaign completion. Verify a quest_completion record exists with `quest_id: NULL`, title containing the campaign name + "complete!", and `xp_earned` equal to the bonus.

**Testing checkpoint:** `cargo test` passes. Build app. Complete a saga — verify the bonus appears in the History section on the Character tab with the descriptive title.

---

## NOT in this step

- XP stats computation (step 2)
- XP distribution on celebrations (step 3)

## Done When

Saga and campaign completion bonuses appear as entries in quest_completion history with descriptive titles and correct XP amounts. `cargo test` passes.
