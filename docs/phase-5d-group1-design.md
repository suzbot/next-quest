# Phase 5D Group 1 — Tech Design (Items 3 & 4)

Design for the two remaining Group 1 items: auto-accomplishment for one-off sagas (with accomplishment table refactor) and overlay lane fallback.

---

## Item 3: Auto-Accomplishment for One-Off Sagas + Schema Refactor

### Data model changes

**New `accomplishment` schema:**

| Column | Type | Notes |
|---|---|---|
| id | TEXT PRIMARY KEY | unchanged |
| source_type | TEXT NOT NULL | `"campaign"` or `"saga"` |
| source_id | TEXT | nullable — nulled when source is deleted |
| name | TEXT NOT NULL | snapshot at completion |
| completed_at | TEXT NOT NULL | unchanged |

Dropped: `campaign_id`, `campaign_name`, `bonus_xp`.

**Rust struct** (`nq-core/src/db.rs`, currently line 412):

```rust
pub struct Accomplishment {
    pub id: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub name: String,
    pub completed_at: String,
}
```

Drop `LevelUp` from the struct (it was never serialized here — the `level_ups` field exists on `CampaignCompletionResult`, not `Accomplishment`, so no change there).

### Migration strategy

SQLite doesn't support column rename/drop in place. Follow the existing pattern at `db.rs:589` (the `quest_completion` nullable migration): create new table, copy, drop, rename.

```sql
-- Only runs if the `source_type` column is missing
CREATE TABLE accomplishment_new (
    id           TEXT PRIMARY KEY,
    source_type  TEXT NOT NULL,
    source_id    TEXT,
    name         TEXT NOT NULL,
    completed_at TEXT NOT NULL
);

INSERT INTO accomplishment_new (id, source_type, source_id, name, completed_at)
SELECT id, 'campaign', campaign_id, campaign_name, completed_at FROM accomplishment;

DROP TABLE accomplishment;
ALTER TABLE accomplishment_new RENAME TO accomplishment;
```

Detection check: `prepare("SELECT source_type FROM accomplishment LIMIT 0").is_ok()`.

Also update both `CREATE TABLE IF NOT EXISTS accomplishment` statements (in `create_tables` at `db.rs:546` and the lazy campaign-tables block at `db.rs:852`) so fresh databases get the new schema directly.

### Auto-accomplishment hook

Hook lives inside `check_saga_completion` (`db.rs:1301`) — closest to the event, gets the dedup benefit for free since the saga's `last_run_completed_at` stamp is set in the same function and one-off sagas can only complete once.

After the existing `UPDATE saga SET last_run_completed_at` but before returning, add:

```rust
if saga.cycle_days.is_none() {
    // One-off saga — create accomplishment if it doesn't already exist.
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM accomplishment WHERE source_type = 'saga' AND source_id = ?1",
        rusqlite::params![saga_id],
        |row| row.get::<_, i32>(0).map(|c| c > 0),
    ).map_err(|e| e.to_string())?;

    if !exists {
        conn.execute(
            "INSERT INTO accomplishment (id, source_type, source_id, name, completed_at)
             VALUES (?1, 'saga', ?2, ?3, ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), saga_id, saga.name, now],
        ).map_err(|e| e.to_string())?;
    }
}
```

**Why dedup in the query not via unique index:** SQLite unique indexes with nullable columns treat NULLs as distinct, so a `UNIQUE(source_type, source_id)` index wouldn't prevent multiple orphaned rows. Explicit check-before-insert is simpler and matches the requirement.

### Campaign accomplishment creation

The existing insert at `db.rs:1948` also needs updating to use the new schema:

```rust
conn.execute(
    "INSERT INTO accomplishment (id, source_type, source_id, name, completed_at)
     VALUES (?1, 'campaign', ?2, ?3, ?4)",
    rusqlite::params![accomplishment_id, campaign_id, campaign_name, now],
)?;
```

Bonus XP is no longer persisted on the row, but the XP still gets awarded — the award logic is unchanged, only the accomplishment row no longer stores it.

### Source deletion orphaning

**Campaign deletion** (`db.rs:1766`) — update to use `source_id`:

```rust
conn.execute(
    "UPDATE accomplishment SET source_id = NULL WHERE source_type = 'campaign' AND source_id = ?1",
    rusqlite::params![id],
)?;
```

**Saga deletion** (`delete_saga` at `db.rs:1103`) — add equivalent orphan step:

```rust
conn.execute(
    "UPDATE accomplishment SET source_id = NULL WHERE source_type = 'saga' AND source_id = ?1",
    rusqlite::params![saga_id],
)?;
```

Both keep the `name` and `completed_at` snapshot so the accomplishment row survives source deletion.

### Read path

`get_accomplishments` (`db.rs:1790`) — update the SELECT and struct build to use the new columns. Query stays ordered by `completed_at DESC`. No API shape change beyond the field renames.

### Frontend changes

`ui/index.html` accomplishment rendering (line 2711–2722):

- Rename `a.campaignName` → `a.name`
- Drop the bonus XP line (`${a.bonusXp > 0 ? ...}`) — field no longer exists
- No other changes to the section

The Character tab lists sagas and campaigns interleaved by `completed_at DESC`. No visual distinction per requirements.

### Tests

Update in `nq-core/src/db.rs` tests:

1. `check_campaign_progress_creates_accomplishment` (line 5694) — update assertions: `campaign_name` → `name`, drop `bonus_xp` assertion, add `source_type == "campaign"` assertion.
2. `delete_campaign_orphans_accomplishment` (line 5790) — update to check `source_id.is_none()` + `source_type == "campaign"` + `name` preserved.
3. **New test:** `one_off_saga_completion_creates_accomplishment` — create a one-off saga, complete all steps, call `check_saga_completion`, assert accomplishment exists with `source_type = "saga"`, correct `source_id`, correct `name`.
4. **New test:** `recurring_saga_completion_does_not_create_accomplishment` — create a recurring saga, complete it, assert no accomplishment row.
5. **New test:** `one_off_saga_dedup` — call `check_saga_completion` twice on the same one-off saga (contrived — direct re-call), assert only one accomplishment row.
6. **New test:** `delete_one_off_saga_orphans_accomplishment` — complete a one-off saga, delete the saga, assert accomplishment row survives with `source_id = NULL` and `name` intact.

---

## Item 4: Overlay Lane Fallback

### Where the logic lives

Entirely frontend, in `ui/overlay.html`. The backend already exposes `get_next_quest(lane)` returning `Option<ScoredQuest>`, which returns `None` when the lane has no quest to offer. No new Rust commands needed.

### Current overlay quest-fetch pattern

Three call sites in `overlay.html` hardcode `lane: "castle_duties"`:

- Line 283: initial fetch on overlay show
- Line 390: "Something Else" advance (with exclude_quest_id)
- Line 392: "Something Else" advance (no exclude)

### Fallback implementation

Replace each direct `get_next_quest` call with a helper that walks lanes in order:

```javascript
async function fetchQuestWithFallback(excludeQuestId) {
  const lanes = ["castle_duties", "adventures", "royal_quests"];
  for (const lane of lanes) {
    const result = await invoke("get_next_quest", {
      excludeQuestId: excludeQuestId ?? undefined,
      lane,
    });
    if (result) {
      return { result, lane };
    }
  }
  return { result: null, lane: null };
}
```

Store the `lane` the overlay landed on for the current poll in a module-level variable (e.g., `currentLane`). On "Something Else" cycling, pass the same `currentLane` back into `get_next_quest` — do NOT re-walk the fallback chain. This keeps the cycling scoped to one lane per poll, matching the requirement.

On the next overlay-refresh event (next poll, or after dismiss+reshow), `fetchQuestWithFallback` is called fresh and re-walks from Lane 1, so fallback state doesn't persist between polls.

### Something Else / Run flow

The two "Something Else" call sites currently look like:

```javascript
currentResult = await invoke("get_next_quest", { excludeQuestId: questId, lane: "castle_duties" });
```

Change to:

```javascript
currentResult = await invoke("get_next_quest", { excludeQuestId: questId, lane: currentLane });
```

Where `currentLane` is set by the initial `fetchQuestWithFallback` call and remains fixed until the next poll.

### Visuals

No change. Overlay monster/encounter theming is lane-agnostic already.

### Tests

No backend tests for the overlay. The fallback behavior is exercisable manually:

1. Complete everything in Lane 1 → overlay fires → Lane 2 quest appears
2. Complete everything in Lane 1 and Lane 2 → overlay fires → Lane 3 quest appears
3. Complete everything → overlay fires → shows nothing (existing behavior)
4. Within a Lane 2 fallback, click Something Else → cycles Lane 2 candidates only

---

## Implementation Steps (Vertical Slices)

### Slice 1: Accomplishment schema refactor (no new behavior)

Rename columns, drop `bonus_xp`, update all read/write sites and tests. Existing behavior (campaign auto-accomplishment) continues to work with the new schema. Frontend renders `name` instead of `campaignName`, drops bonus XP line.

**Testable end-state:** User can complete a campaign, see the accomplishment appear on the Character tab with name + date, delete the campaign and still see the accomplishment. Existing functionality preserved.

### Slice 2: Auto-accomplishment hook for one-off sagas

Add the saga completion hook in `check_saga_completion`, add `delete_saga` orphan step, add tests.

**Testable end-state:** User can complete a one-off saga and see it appear in the Accomplishments list on the Character tab, alongside campaign accomplishments, ordered by completion time. Deleting the saga keeps the accomplishment row with the name preserved. Recurring sagas don't create accomplishments.

### Slice 3: Overlay lane fallback

Frontend-only change in `overlay.html`. Add `fetchQuestWithFallback` helper, track `currentLane`, wire cycling to use it.

**Testable end-state:** User with nothing due in Lane 1 triggers the overlay (or waits for poll) and sees a Lane 2 quest. Can Fight / Cast Completion / Run / Hide normally. The quest was actually from the second quest giver, but the overlay looks and feels identical to a Lane 1 encounter.

---

## Summary

**Approach:** Refactor the `accomplishment` table to a generic `source_type` + `source_id` + `name` shape in one slice (preserving existing campaign behavior), then layer the one-off saga trigger on top in a second slice. Overlay fallback is a small, independent frontend change and goes in a third slice.

**Three slices, each independently testable:**

1. Schema refactor with no behavior change — safe groundwork
2. One-off saga auto-accomplishment — the actual new feature
3. Overlay lane fallback — independent, frontend-only

Each slice is small enough to build, test, and commit in one session. Slices 1 and 2 are sequential (2 depends on the schema from 1). Slice 3 is independent and can be done at any time.
