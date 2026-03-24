# Step Spec: Phase 2G.2-3/4 — Completion Bonus, Celebration, Accomplishments, Duplication

## Goal

When a campaign completes: calculate bonus XP, award it to character, show a gold celebration notification, and create an accomplishment record. Accomplishments appear in a new column on the Character tab. Campaigns can be duplicated to create a new version with adjusted criteria.

---

## Substep 1: Backend + frontend — bonus calculation, accomplishment CRUD, celebration notifications ✅

### Backend

**Bonus calculation** — add to `check_campaign_progress` (db.rs), replacing the `bonus_xp: 0` placeholder:

When all criteria are met, before returning the result:

```
per quest criterion:
  look up quest difficulty, quest_type, cycle_days
  calculate_xp(difficulty, quest_type, cycle_days) × target_count

per saga criterion:
  150 × target_count

bonus_xp = round(0.20 × sum across all criteria)
```

If the quest has been deleted, use 0 for that criterion's contribution (can't look up difficulty). Saga criteria use a flat 150 XP per run.

Award bonus XP to character only (`UPDATE character SET xp = xp + bonus_xp`). Detect character level-up (snapshot before/after, same pattern as saga completion bonus).

Add `level_ups: Vec<LevelUp>` to `CampaignCompletionResult`.

Create accomplishment record: `INSERT INTO accomplishment (id, campaign_id, campaign_name, completed_at, bonus_xp)` with a snapshot of the campaign name and the calculated bonus.

**Accomplishment CRUD:**

- `get_accomplishments(conn) -> Vec<Accomplishment>` — returns all accomplishments ordered by `completed_at` descending.
- `delete_accomplishment(conn, id)` — deletes the record. Does NOT reduce XP.

**Accomplishment struct:**

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Accomplishment {
    pub id: String,
    pub campaign_id: Option<String>,
    pub campaign_name: String,
    pub completed_at: String,
    pub bonus_xp: i64,
}
```

**Commands** (commands.rs): Wrappers for `get_accomplishments` and `delete_accomplishment`. Register in `main.rs`.

**Tests:**

1. `check_campaign_progress_awards_bonus_xp` — Create campaign with 2 quest criteria (easy recurring daily ×2, moderate recurring 7-day ×1). Complete to trigger campaign completion. Verify bonus_xp > 0, matches formula: `round(0.20 × (calculate_xp(easy, recurring, 1) × 2 + calculate_xp(moderate, recurring, 7) × 1))`. Verify character XP increased by bonus amount.

2. `check_campaign_progress_creates_accomplishment` — Complete a campaign. Verify accomplishment record exists via `get_accomplishments`: correct campaign_name snapshot, bonus_xp, completed_at set.

3. `check_campaign_progress_saga_criterion_bonus` — Campaign with saga criterion (×2). Complete twice. Verify bonus includes `150 × 2` for the saga portion.

4. `check_campaign_progress_deleted_quest_bonus` — Campaign with criterion for a quest that has been deleted. The deleted criterion contributes 0 to the bonus sum. Verify the campaign still completes and bonus is calculated from remaining valid criteria only.

5. `check_campaign_progress_level_up_detection` — Set character XP just below a level threshold. Complete a campaign whose bonus pushes character over. Verify `level_ups` in the result contains the character level-up.

6. `delete_accomplishment_preserves_xp` — Complete a campaign (character gains bonus XP). Delete the accomplishment. Verify character XP is unchanged.

7. `delete_campaign_orphans_accomplishment` — Complete a campaign (accomplishment created). Delete the campaign. Verify accomplishment still exists with `campaign_id: None`.

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

### Frontend — celebration notifications at all completion paths

**CampaignCompletionResult now has `bonusXp` and `levelUps`.**

At each of the five completion paths, `check_campaign_progress` already returns `Vec<CampaignCompletionResult>`. When any result has `completed: true`, show a gold celebration notification (same `.saga-celebration.pulsing` style):

```
Campaign Name complete! +N bonus XP
```

Plus any level-ups from the bonus.

**Where to show:**

The campaign progress calls are already wired at all five paths. After each call, check the results and append celebration HTML if any campaigns completed. The celebration is appended after any saga celebration (campaigns are a higher-level grouping).

Specific locations:
- `completeQuest()` (quest list) — append to feedbackDiv, below saga celebration if present
- `qgDone()` (quest giver) — append to textHtml, after saga celebration block
- `timerDone()` — append to textHtml, after saga celebration block
- `completeSagaStep()` (saga tab) — insert after saga row (same as saga celebration, but below it)
- `questDone()` (overlay) — append to textHtml, after saga celebration block

**Helper function** (both index.html and overlay.html):

```javascript
function campaignCelebrationHtml(results) {
  return results
    .filter(r => r.completed)
    .map(r => {
      let html = `<div class="saga-celebration pulsing" style="margin-top: 8px;">`;
      html += `<strong>${escapeHtml(r.campaignName)} complete!</strong>`;
      if (r.bonusXp > 0) {
        html += ` <span class="xp-flash" style="color: #7a5500">+${r.bonusXp} bonus XP</span>`;
      }
      if (r.levelUps?.length > 0) {
        html += levelUpHtml(r.levelUps);
      }
      html += `</div>`;
      return html;
    }).join("");
}
```

**Change to campaign progress call pattern:** Currently each path fires `check_campaign_progress` and ignores the return value. Change to capture the results and pass them to the celebration helper. Where there are two calls (quest + saga), collect results from both.

**Testing checkpoint:** Build app. Create a campaign with 1 easy quest criterion. Complete the quest. Verify gold celebration shows with bonus XP. Check that character XP increased. Navigate to Character tab — accomplishment should appear (substep 2).

---

## Substep 2: Frontend — accomplishments column on Character tab ✅

**Layout change:**

Wrap the existing character content in a two-column flex container. Existing content gets `flex: 2`. New accomplishments column gets `flex: 1`.

```
<div style="display: flex; gap: 16px;">
  <div style="flex: 2;">
    <!-- existing: char header, meter, attributes, skills -->
  </div>
  <div style="flex: 1;">
    <h2>Accomplishments</h2>
    <!-- accomplishment list -->
  </div>
</div>
```

**Data loading:**

Add `invoke("get_accomplishments")` to `loadCharacterView`'s parallel fetches. Pass to `renderCharacterView`.

**Accomplishment display:**

Each entry shows:
- Campaign name (from snapshot)
- Completion date (formatted same as completion dates elsewhere)
- Bonus XP
- Delete button (×) — same pattern as completion deletion, with confirm

Empty state: "No accomplishments yet."

**Testing checkpoint:** Build app. Complete a campaign — verify accomplishment appears on Character tab with name, date, and bonus XP. Delete the campaign — accomplishment survives. Delete the accomplishment — it disappears, XP unchanged.

---

## Substep 3: Frontend — campaign duplication ✅

**Duplicate action:**

Add a "Dup" button to the campaign edit mode row (alongside Save, Del, ✕). Available on any campaign — active or completed.

```javascript
function renderCampaignEdit(c) {
  return `<li class="quest-edit" data-id="${c.id}">
    <input ... >
    <button onclick="saveCampaignEdit('${c.id}')">Save</button>
    <button onclick="duplicateCampaign('${c.id}')">Dup</button>
    <button class="del-btn" onclick="deleteCampaign('${c.id}')">Del</button>
    <span class="close-x" onclick="cancelCampaignEdit()">✕</span>
  </li>`;
}
```

**`duplicateCampaign(id)` function:**

1. Find the campaign in `cachedCampaigns` by ID
2. Pre-fill the creation form:
   - Name: `campaign.name + " copy"`
   - Criteria: copy each criterion's `targetType`, `targetId`, `targetName`, `targetCount` into `campaignDraftCriteria`
3. Open the creation form (same as `showCampaignForm` but with pre-filled data)
4. Close edit mode

The user can then rename, add/remove/adjust criteria before saving. Saving calls `create_campaign` — it's just a regular creation with a new `created_at` and all counts at 0.

**Testing checkpoint:** Build app. Complete a campaign. Enter edit mode, click Dup. Verify form opens with "Name copy" and same criteria pre-filled. Remove one criterion, add a new one, rename, save. New campaign appears with fresh 0/N counts. Original campaign unchanged.

---

## NOT in this step

- Campaign nudging in quest giver (future)
- Level/XP criterion types (future)
- Time-bounded criteria (future)
- Richer accomplishment display — titles, honors, badges (future)

## Done When

All three substeps complete. Campaigns award bonus XP on completion with gold celebration at all five paths. Accomplishments appear on Character tab and survive campaign deletion. Campaigns can be duplicated. `cargo test` passes.
