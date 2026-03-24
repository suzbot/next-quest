# Phase 2G.2: Campaigns and Honors — Design

## Overview

Campaigns track progress toward user-defined collections of criteria. Criteria are locked after creation. Completions increment stored counters. When all criteria are met, the campaign completes with a bonus and appears as an accomplishment on the Character tab.

## 1. Schema

### New table: `campaign`

| Column | Type | Default | Description |
|---|---|---|---|
| id | TEXT PK | — | UUID |
| name | TEXT NOT NULL | — | Campaign title |
| created_at | TEXT NOT NULL | — | ISO 8601. Completions before this don't count. |
| completed_at | TEXT | NULL | Stamped when all criteria met. |

### New table: `campaign_criterion`

| Column | Type | Default | Description |
|---|---|---|---|
| id | TEXT PK | — | UUID |
| campaign_id | TEXT NOT NULL | — | FK to campaign |
| target_type | TEXT NOT NULL | — | `"quest_completions"` or `"saga_completions"` (extensible string enum) |
| target_id | TEXT NOT NULL | — | FK to quest or saga |
| target_count | INTEGER NOT NULL | — | How many completions needed |
| current_count | INTEGER NOT NULL | 0 | Increments on qualifying completions |
| sort_order | INTEGER NOT NULL | — | Display order within campaign |

Unique constraint: `(campaign_id, target_type, target_id)` — no duplicate criteria for the same target and type within a campaign. Includes target_type so future criterion types (e.g., `"skill_level"` and `"skill_xp"`) can reference the same target_id without conflicting.

### New table: `accomplishment`

| Column | Type | Default | Description |
|---|---|---|---|
| id | TEXT PK | — | UUID |
| campaign_id | TEXT | NULL | FK to campaign. NULL if campaign deleted. |
| campaign_name | TEXT NOT NULL | — | Snapshot of name at completion time |
| completed_at | TEXT NOT NULL | — | ISO 8601 |
| bonus_xp | INTEGER NOT NULL | 0 | XP awarded |

### Migration

- Create all three tables (campaign, campaign_criterion, accomplishment)
- No changes to existing tables

## 2. Campaign CRUD

### Backend functions

- `get_campaigns(conn)` — returns all campaigns with their criteria and progress
- `create_campaign(conn, name, criteria: Vec<NewCriterion>)` — creates campaign + all criteria in one call. Criteria are locked from this point.
- `rename_campaign(conn, id, name)`
- `delete_campaign(conn, id)` — deletes campaign and its criteria. Orphans any accomplishment (sets campaign_id to NULL, same pattern as quest deletion orphaning completions).

### `NewCriterion` struct

```rust
pub struct NewCriterion {
    pub target_type: String,
    pub target_id: String,
    pub target_count: i32,
}
```

### `CampaignWithCriteria` return struct

```rust
pub struct CampaignWithCriteria {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub criteria: Vec<Criterion>,
}

pub struct Criterion {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub target_name: String,  // looked up at query time
    pub target_count: i32,
    pub current_count: i32,
}
```

The `target_name` is resolved by joining against the quest or saga table. If the target is deleted, returns "Deleted quest" or "Deleted saga".

### Accomplishment functions

- `get_accomplishments(conn)` — returns all accomplishments, ordered by completed_at descending
- `delete_accomplishment(conn, id)` — deletes the record, does NOT reduce XP

### Duplication

No backend command needed. The frontend reads an existing campaign's criteria and pre-fills the creation form. Saving calls `create_campaign` with the new name and adjusted criteria — it's just a regular creation.

## 3. Progress Tracking

### `check_campaign_progress(conn, target_type, target_id) -> Vec<CampaignCompletionResult>`

Called after each qualifying event:
1. Find all active campaigns (completed_at IS NULL) with a criterion matching `target_type` and `target_id`
2. Increment `current_count` for each matching criterion
3. For each affected campaign, check if all criteria are met (current_count >= target_count for all)
4. If campaign is complete: stamp `completed_at`, calculate bonus, return the result

Returns a list of newly completed campaigns (usually 0 or 1, but could be multiple if one completion satisfies the final criterion of multiple campaigns).

### Where it's called

**Frontend hooks (same pattern as `check_saga_completion`):**
- After `complete_quest`: call `check_campaign_progress` with `target_type: "quest_completions"` and `target_id: quest.id`
- After `check_saga_completion` returns completed=true: call `check_campaign_progress` with `target_type: "saga_completions"` and `target_id: saga_id`

These calls happen from:
- `completeSagaStep` in saga tab
- `qgDone` in quest giver
- `questDone` in overlay
- `timerDone` in quest giver
- `completeQuest` in quest list

### `CampaignCompletionResult` struct

```rust
pub struct CampaignCompletionResult {
    pub completed: bool,
    pub campaign_name: String,
    pub bonus_xp: i64,
}
```

When a campaign completes, `check_campaign_progress` also:
1. Stamps `completed_at` on the campaign
2. Awards bonus XP to character
3. Creates an accomplishment record (snapshots campaign name, records bonus XP and timestamp)

## 4. Campaign Completion Bonus

### Formula (Option B: percentage of constituent baseline XP)

```
per quest criterion:  calculate_xp(quest.difficulty, quest.quest_type, quest.cycle_days) × target_count
per saga criterion:   150 × target_count   (moderate one-off equivalent per saga run)
bonus = round(0.20 × sum across all criteria)
```

Uses the existing `calculate_xp` function for quest criteria. Saga criteria use a flat 150 XP per run as a reasonable approximation. 20% matches the saga completion bonus rate. Based on baseline (time mult = 1.0), not actual earned XP — no procrastination reward.

### Distribution

Bonus XP awarded to character only (campaigns don't have skill/attribute links).

### Celebration

Same gold-accented pulsing notification as saga completion: "Spring Cleaning 2026 complete! +135 bonus XP"

Shown in context where the final criterion was met:
- Quest list: below the quest row
- Saga tab: below the step row
- Quest giver: appended to the completion feedback
- Overlay: appended to the completion feedback

## 5. Campaigns Tab UI

### Tab placement

Between Sagas and Character.

### Creation form

Triggered by "Add Campaign" button (or by duplicating an existing campaign).

```
Campaign name: [________________________]

Criteria:
  Laundry saga           ×4    [✕]
  Vacuuming              ×4    [✕]
  Mopping floors         ×2    [✕]

  [Quest/Saga dropdown ▾] [count] [Add]

  [Save]     [Cancel]
```

- Save is disabled until at least one criterion is added
- Cancel discards the draft (nothing saved to DB)
- Criteria can be added and removed freely before Save
- Duplicate criteria for the same target are prevented (show error or merge)

### Campaign list

Same expand/collapse pattern as sagas.

**Collapsed:** toggle, name, progress bar, criteria count (N/M met)
**Expanded:** read-only criteria checklist with ✓, target name, tally

### Actions on saved campaigns

- Click name → inline rename
- Duplicate → opens creation form pre-filled
- Delete → from edit mode, with confirm

## 6. Accomplishments

### New table: `accomplishment`

Parallels how completions relate to quests: the accomplishment is an independent record created when a campaign completes. It survives campaign deletion, just as completions survive quest deletion.

| Column | Type | Default | Description |
|---|---|---|---|
| id | TEXT PK | — | UUID |
| campaign_id | TEXT | NULL | FK to campaign. NULL if campaign has been deleted. |
| campaign_name | TEXT NOT NULL | — | Snapshot of campaign name at completion time |
| completed_at | TEXT NOT NULL | — | ISO 8601 timestamp |
| bonus_xp | INTEGER NOT NULL | — | XP awarded |

Deleting a campaign orphans its accomplishment (sets campaign_id to NULL) but does not delete it. Accomplishments are individually deletable (same as completions). Deleting an accomplishment does NOT reduce XP — XP only goes up.

### Character tab layout

The Character tab currently renders as a single column (character meter, attributes, skills, history). This adds a second column for accomplishments.

```
[Character meter                    ] | Accomplishments
[Attributes                         ] |   ★ New Year Setup
  Health  Lvl 2 [====------] (38/67)  |     Jan 15, 2026
  Pluck   Lvl 1 [==--------] (12/37)  |
[Skills                              ] |
  Cooking Lvl 1 [===-------] (15/37)  |
  ...                                  |
```

### Implementation

Wrap the existing character content in a flex container with `flex: 2`. Add an accomplishments column with `flex: 1`.

### Accomplishment display

Each entry shows:
- Campaign name (from snapshot — survives deletion/rename)
- Completion date
- Deletable (same pattern as completion deletion)

## Implementation Order

1. **2G.2-1: Schema + campaign CRUD + tab with creation form** — Migration for all three tables (campaign, campaign_criterion, accomplishment). Backend create/rename/delete/get. Campaigns tab with Add Campaign button, inline creation form (name + criteria builder + save), campaign list with expand/collapse, rename, delete. No progress tracking yet — criteria show 0/N. Testing: create a campaign with 3 criteria, verify it appears in the list. Expand to see criteria. Rename it. Delete it. Try saving with no criteria (blocked). Try adding duplicate criteria for same quest (blocked).

2. **2G.2-2: Progress tracking** — `check_campaign_progress` backend function. Frontend hooks after every completion point (quest list, saga tab, quest giver, overlay, timer). Criteria counts increment on qualifying completions. Campaign auto-completes when all criteria met (stamps completed_at, dims to inactive styling). Testing: create a campaign with "2 completions of Vacuuming." Complete Vacuuming twice — watch count go 0/2 → 1/2 → 2/2 and campaign dim. Create two campaigns referencing the same quest — verify one completion increments both. Complete a quest that existed before the campaign — verify it doesn't count (no retroactive credit).

3. **2G.2-3: Completion bonus + celebration + accomplishments** — Bonus XP calculation (20% of constituent baseline) and award to character. Gold celebration notification in context where final criterion was met. Accomplishment record created with name snapshot and bonus XP. New Accomplishments column on Character tab (second column alongside existing meters). Testing: complete a campaign, verify bonus XP and gold celebration. Check Character tab — accomplishment appears with name and date. Delete the campaign — accomplishment survives. Delete the accomplishment — XP unchanged.

4. **2G.2-4: Duplication** — Duplicate action on any campaign (active or completed). Opens creation form pre-filled with same name (" copy") and same criteria. User can rename, add/remove/adjust criteria before saving. Saving creates a fresh campaign. Testing: complete "Spring Cleaning 2026," duplicate it, rename to "Spring Cleaning 2027," remove one criterion, add a new one, save. New campaign starts at 0/N with a new created_at.
