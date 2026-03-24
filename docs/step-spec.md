# Step Spec: Phase 2G.2-1 — Schema + Campaign CRUD + Tab with Creation Form

## Goal

User can create, view, rename, and delete campaigns with criteria from a new Campaigns tab. No progress tracking yet — criteria show 0/N.

---

## Substep 1: Backend — schema, structs, CRUD

**Migration** (`db.rs` → `migrate()`):

Create three tables (campaign, campaign_criterion, accomplishment). Detection: check if campaign table exists.

```sql
CREATE TABLE IF NOT EXISTS campaign (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS campaign_criterion (
    id            TEXT PRIMARY KEY,
    campaign_id   TEXT NOT NULL REFERENCES campaign(id),
    target_type   TEXT NOT NULL,
    target_id     TEXT NOT NULL,
    target_count  INTEGER NOT NULL,
    current_count INTEGER NOT NULL DEFAULT 0,
    sort_order    INTEGER NOT NULL,
    UNIQUE(campaign_id, target_type, target_id)
);

CREATE TABLE IF NOT EXISTS accomplishment (
    id             TEXT PRIMARY KEY,
    campaign_id    TEXT,
    campaign_name  TEXT NOT NULL,
    completed_at   TEXT NOT NULL,
    bonus_xp       INTEGER NOT NULL DEFAULT 0
);
```

**Structs:**

```rust
pub struct NewCriterion {
    pub target_type: String,  // "quest_completions" or "saga_completions"
    pub target_id: String,
    pub target_count: i32,
}

pub struct Criterion {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub target_name: String,  // looked up from quest/saga, "Deleted quest/saga" if missing
    pub target_count: i32,
    pub current_count: i32,
}

pub struct CampaignWithCriteria {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub criteria: Vec<Criterion>,
}
```

**Backend functions:**

- `get_campaigns(conn) -> Vec<CampaignWithCriteria>` — returns all campaigns with criteria. Resolves target_name by looking up quest title or saga name. Falls back to "Deleted quest" / "Deleted saga" if target not found.
- `create_campaign(conn, name: String, criteria: Vec<NewCriterion>) -> CampaignWithCriteria` — creates campaign + all criteria atomically. Sets created_at to now. Assigns sort_order from the order criteria are provided (first added = 1, second = 2, etc.). Validates: at least one criterion, no duplicate target_ids, all target_ids exist, and each target_type matches its target_id (quest_completions must reference a quest, saga_completions must reference a saga).
- `rename_campaign(conn, id: String, name: String)`
- `delete_campaign(conn, id: String)` — deletes campaign + criteria. Orphans any accomplishment (UPDATE accomplishment SET campaign_id = NULL WHERE campaign_id = ?).

**Commands** (`commands.rs`):

Wrappers for each function. `create_campaign` accepts `name: String` and `criteria: Vec<NewCriterion>` (NewCriterion derives Deserialize with camelCase).

**Tests:**

- Create campaign with 2 criteria — verify returned with correct name, criteria count, all current_count = 0
- Create campaign with no criteria — error
- Create campaign with duplicate target_id — error
- Create campaign with mismatched target_type (e.g., quest_completions pointing to a saga) — error
- Get campaigns returns all with criteria
- Rename campaign — verify name changed
- Delete campaign — verify campaign and criteria gone
- Target name resolution — create campaign referencing a quest, verify target_name is quest title

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Frontend — Campaigns tab with creation form and campaign list

**Tab:**

- New "Campaigns" tab between Sagas and Character in the nav bar
- Added to `showView`, tab locking during battle, etc. (same pattern as Sagas tab)

**Creation form:**

Triggered by "Add Campaign" button at the top. Appears inline (same position as saga/quest add forms).

```
[Campaign name input                    ]

  Vacuuming              ×4    [✕]
  Laundry saga           ×2    [✕]

  [Quest/Saga dropdown ▾]  [count input]  [Add Criterion]

  [Save]  [Cancel]
```

- Quest/saga dropdown populated from `cachedQuests` (active recurring + active one-off) and `cachedSagas`
- Grouped in dropdown: quests first, then sagas, labeled "(quest)" / "(saga)" to distinguish
- Count input defaults to 1, min 1
- Adding a criterion appends to a JS-side list (not saved to DB yet)
- Each criterion in the list has a remove (✕) button
- Adding a criterion for a target_id already in the list shows an error / is prevented
- Save button disabled until at least one criterion exists
- Save calls `create_campaign` with name + criteria list, clears the form, reloads campaigns
- Cancel discards the draft and hides the form

**Campaign list:**

Expand/collapse pattern consistent with sagas.

*Collapsed row:*
- Expand toggle (▸/▾), campaign name, progress bar, criteria met count (N/M)
- Active campaigns: normal styling
- Completed campaigns: dimmed/inactive styling, full progress bar, "Done" (not functional yet — no campaigns can complete until step 2)

*Expanded row:*
- Criteria checklist: ✓ when current_count >= target_count, target name, tally (current/target)
- Read-only — no edit controls for criteria

**Campaign actions:**

- Click name to rename (inline edit, same pattern as sagas — Enter to save, Escape to cancel)
- Delete from edit mode (Del button, same pattern as sagas)

**Keyboard accessible:**

- Arrow keys navigate between campaign rows
- Enter/E to edit name

**Testing checkpoint:** Build app. Create a campaign with 3 criteria — appears in list with 0/3 and empty progress bar. Expand to see criteria at 0/N each. Rename it. Delete it. Try saving with no criteria (Save disabled). Try adding the same quest twice (prevented). Cancel discards draft.

---

## NOT in this step

- Progress tracking (2G.2-2)
- Completion bonus + celebration (2G.2-3)
- Accomplishments on Character tab (2G.2-3)
- Duplication flow (2G.2-4)

## Done When

Both substeps complete. Campaigns can be created with locked criteria, viewed, renamed, and deleted. The Campaigns tab shows the campaign list with expand/collapse. All criteria show 0/N. `cargo test` passes.
