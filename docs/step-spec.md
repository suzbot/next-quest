# Step Spec: Phase 2G.2-2 — Progress Tracking

## Goal

Quest and saga completions increment matching campaign criteria. When all criteria are met, the campaign auto-completes (stamps `completed_at`, dims in list). Also fixes three pre-existing saga completion gaps so that all five completion paths correctly handle saga completion and can chain into campaign progress.

---

## Substep 1: Backend — `check_campaign_progress` function + tests

**New struct** (`db.rs`):

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCompletionResult {
    pub completed: bool,
    pub campaign_name: String,
    pub bonus_xp: i64,  // always 0 in this step — wired in 2G.2-3
}
```

**New function** (`db.rs`):

`check_campaign_progress(conn, target_type: &str, target_id: &str) -> Result<Vec<CampaignCompletionResult>, String>`

Logic:
1. Find all active campaigns (`completed_at IS NULL`) that have a criterion matching `target_type` and `target_id`
2. For each matching criterion, increment `current_count` by 1
3. For each affected campaign, check if ALL its criteria are now met (`current_count >= target_count` for every criterion)
4. If a campaign is fully met: stamp `completed_at` to now
5. Return a list of campaigns that just completed (usually 0 or 1). Include `completed: true`, the campaign name, and `bonus_xp: 0` (placeholder for step 3)

No bonus XP award, no accomplishment creation — those are step 2G.2-3.

**New command** (`commands.rs`):

```rust
#[tauri::command]
pub fn check_campaign_progress(
    state: State<DbState>,
    target_type: String,
    target_id: String,
) -> Result<Vec<db::CampaignCompletionResult>, String>
```

Register in `invoke_handler` in `main.rs`.

**Bugfix — SagaCompletionResult serde:**

Add `#[serde(rename_all = "camelCase")]` to `SagaCompletionResult` (line 1041 of `db.rs`). Without this, the frontend accesses `sagaResult.sagaName`, `sagaResult.bonusXp`, and `sagaResult.levelUps` but the struct serializes as `saga_name`, `bonus_xp`, `level_ups` — causing saga celebrations to display "undefined complete!" with no bonus XP or level-ups.

**Tests:**

1. `check_campaign_progress_increments_quest_criterion` — Create campaign with quest criterion (target_count: 2). Call `check_campaign_progress("quest_completions", quest_id)`. Verify current_count is now 1 (via `get_campaigns`). Returns empty vec (not yet complete).

2. `check_campaign_progress_completes_campaign` — Campaign with quest criterion (target_count: 1). Call once. Returns vec with one result: `completed: true`, correct campaign name, `bonus_xp: 0`. Verify campaign's `completed_at` is now set (via `get_campaigns`).

3. `check_campaign_progress_multiple_campaigns_same_quest` — Two campaigns both referencing the same quest. One call to `check_campaign_progress` increments the criterion in both campaigns.

4. `check_campaign_progress_saga_criterion` — Campaign with saga criterion. Call `check_campaign_progress("saga_completions", saga_id)`. Verify saga criterion count incremented.

5. `check_campaign_progress_no_match` — Call with a quest_id that no campaign references. Returns empty vec. No campaigns modified.

6. `check_campaign_progress_skips_completed_campaign` — Campaign that already has `completed_at` set. New call to `check_campaign_progress` with a matching target does NOT increment its criteria.

7. `check_campaign_progress_multi_criteria_partial` — Campaign with 2 criteria (quest A ×2, quest B ×1). Complete quest A once — campaign not complete (1/2 + 0/1). Complete quest B once — campaign not complete (1/2 + 1/1). Complete quest A again — campaign completes (2/2 + 1/1).

8. `check_campaign_progress_full_saga_flow` — Integration test: Create saga with 2 steps. Create campaign with quest_completions criterion for step 1 (×1) and saga_completions criterion for the saga (×1). Complete step 1 → call `check_campaign_progress("quest_completions", step1_id)` → quest criterion increments. Complete step 2 → call `check_saga_completion` → saga completes. Call `check_campaign_progress("quest_completions", step2_id)` then `check_campaign_progress("saga_completions", saga_id)` → saga criterion increments → campaign completes.

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Frontend — saga completion bugfixes + campaign progress hooks

### Bugfix: `completeQuest()` (quest list, `index.html` ~line 1420)

Currently does NOT check saga completion when completing a saga step. Fix:

```
After complete_quest:
  → look up quest.saga_id from cachedQuests
  → if saga_id: call check_saga_completion({ sagaId })
  → if saga completed: show saga celebration below the quest row
    (same gold-accented pulsing div as qgDone/completeSagaStep)
```

### Bugfix: `timerDone()` (`index.html` ~line 1848)

Currently calls `check_saga_completion_for_quest` but discards the result. Fix:

```
Store the saga result.
If completed: show saga celebration in the completion feedback
  (append gold div after XP/level-up text, same pattern as qgDone)
```

Also need to resolve saga_id for campaign progress: look up quest from `cachedQuests` using `completion.quest_id`.

### Bugfix: `questDone()` (overlay, `overlay.html` ~line 274)

Currently calls `check_saga_completion` but discards the result. Fix:

```
Store the saga result.
If completed: append saga celebration to completion feedback
  (gold div with saga name + bonus XP, same pattern as qgDone)
```

### Campaign progress hooks — all five completion paths

After each quest completion, call `check_campaign_progress("quest_completions", questId)`.
If a saga also completed, additionally call `check_campaign_progress("saga_completions", sagaId)`.

The saga_id is available at each path via:
- `completeQuest()` — `cachedQuests.find(q => q.id === id).saga_id`
- `qgDone()` — `sagaId` parameter
- `timerDone()` — `cachedQuests.find(q => q.id === completion.quest_id).saga_id`
- `completeSagaStep()` — `sagaId` parameter
- `questDone()` (overlay) — `quest.saga_id`

No celebration notification for campaign completion yet (that's 2G.2-3). The campaign list will show updated counts and dimmed styling on next refresh.

### Add campaigns to `loadAll()`

Add `invoke("get_campaigns")` to the parallel fetches in `loadAll()` and store result in `cachedCampaigns`. Then call `renderCampaigns()` if the campaigns view is visible. This ensures campaign progress is reflected after every completion.

### Specific changes per function

**`completeQuest(id)`** (`index.html`):
```
const quest = cachedQuests.find(q => q.id === id);
const completion = await invoke("complete_quest", { questId: id });
// ... existing XP/level-up display ...

// NEW: saga completion check
let sagaResult = null;
if (quest?.saga_id) {
  sagaResult = await invoke("check_saga_completion", { sagaId: quest.saga_id });
  if (sagaResult?.completed) {
    // show saga celebration div below quest row
  }
}

// NEW: campaign progress
await invoke("check_campaign_progress", { targetType: "quest_completions", targetId: id });
if (sagaResult?.completed && quest?.saga_id) {
  await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: quest.saga_id });
}
```

**`qgDone(questId, difficulty, sagaId)`** (`index.html`):
```
// existing code already handles saga completion + celebration
// ADD after saga check:
await invoke("check_campaign_progress", { targetType: "quest_completions", targetId: questId });
if (sagaResult?.completed && sagaId) {
  await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: sagaId });
}
```

**`timerDone()`** (`index.html`):
```
const completion = await invoke("complete_timer");
// FIX: use saga result instead of discarding
let sagaResult = null;
const quest = cachedQuests.find(q => q.id === completion.quest_id);
if (completion.quest_id) {
  sagaResult = await invoke("check_saga_completion_for_quest", { questId: completion.quest_id });
}
// ... existing display code ...
// ADD saga celebration if completed (same pattern as qgDone)

// NEW: campaign progress
if (completion.quest_id) {
  await invoke("check_campaign_progress", { targetType: "quest_completions", targetId: completion.quest_id });
}
if (sagaResult?.completed && quest?.saga_id) {
  await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: quest.saga_id });
}
```

**`completeSagaStep(questId, sagaId)`** (`index.html`):
```
// existing code already handles saga completion + celebration
// ADD in the saga completion check callback (after the setTimeout):
await invoke("check_campaign_progress", { targetType: "quest_completions", targetId: questId });
if (result.completed) {
  await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: sagaId });
}
```

**`questDone()`** (`overlay.html`):
```
const completion = await invoke("complete_quest", { questId: quest.id });
// FIX: use saga result instead of discarding
let sagaResult = null;
if (quest.saga_id) {
  sagaResult = await invoke("check_saga_completion", { sagaId: quest.saga_id });
}
// ... existing display code ...
// ADD saga celebration if completed

// NEW: campaign progress
await invoke("check_campaign_progress", { targetType: "quest_completions", targetId: quest.id });
if (sagaResult?.completed && quest.saga_id) {
  await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: quest.saga_id });
}
```

**Testing checkpoint:** Build app. Create a campaign with "Vacuuming ×2." Complete Vacuuming — expand campaign to see 1/2. Complete again — see 2/2, campaign dims. Create two campaigns referencing the same quest — verify one completion increments both. Complete a saga step from each of the five paths — verify saga celebration shows correctly at all. Create campaign with saga criterion — complete the saga — verify saga criterion increments.

---

## NOT in this step

- Completion bonus XP calculation (2G.2-3)
- Gold celebration notification for campaign completion (2G.2-3)
- Accomplishment record creation (2G.2-3)
- Accomplishments on Character tab (2G.2-3)
- Duplication flow (2G.2-4)

## Done When

Both substeps complete. Campaign criteria increment on qualifying completions. Campaigns auto-complete when all criteria met (completed_at stamped, dimmed in list). Saga celebration displays correctly from all five completion paths. `cargo test` passes.
