# Phase 5D Group 1 — Requirements

Three small-to-medium polish items to be implemented in order: reset skips button, show/hide lanes, then auto-accomplishment for one-off sagas.

---

## Item 1: Reset Skips Button

### Context

The quest giver's scoring system penalizes quests that have been skipped ("Something Else" / "Run"). Skip counts accumulate in memory and reset at local midnight or on app restart. Sometimes the quest giver gets into a bad state — you've skipped everything in a lane, or you want a fresh slate without waiting for midnight or restarting the app.

### Goal

A button on the Settings tab that manually resets all skip state immediately.

### Behavior

- **Location:** Settings tab, alongside other app controls
- **Label:** Something like "Reset Skip Counts" (final wording TBD in design)
- **Action when clicked:**
  1. Clear all skip counts across all three lanes
  2. Clear the "last skipped" quest ID (which excludes the just-skipped quest from the next pick)
  3. Re-score the quest giver so the next quest reflects the reset state
- **No confirmation dialog** — resetting skips is low-stakes (they'd reset at midnight anyway)
- **Visual feedback:** brief confirmation (e.g., button flash or toast) so the user knows it worked

### Non-goals

- Does not affect XP, completions, or any persistent data
- Does not affect "Not Today" dismissals (those are separate and persist)

---

## Item 2: Show/Hide Lanes

### Context

The Next Quest tab shows three quest giver lanes stacked vertically: Castle Duties (trivial), Adventures (easy), Royal Quests (moderate/challenging/epic). On a low-capacity day, seeing the harder lanes can be overwhelming — the user wants to focus on what they can handle and deal with the bigger stuff when they're ready.

### Goal

Each lane can be individually collapsed or expanded. Collapsed lanes show a themed prompt that invites the user to engage when ready. The state is session-sticky with a fresh-start default on each app launch.

### Behavior

**Collapse/expand control:**
- Each lane has a toggle to collapse/expand itself
- Clicking the toggle when expanded collapses the lane to its themed prompt
- Clicking the themed prompt (or a re-expand control) expands the lane back to the full quest giver

**Collapsed display — themed prompts:**
- Lane 1 (Castle Duties): "Attend to duties"
- Lane 2 (Adventures): "Ask around town"
- Lane 3 (Royal Quests): "Approach the throne"

**Default state (on app launch):**
- Lane 1 (Castle Duties): expanded
- Lane 2 (Adventures): collapsed
- Lane 3 (Royal Quests): collapsed

**Stickiness:**
- User's collapse/expand choices persist throughout the session — whatever state the user sets, it stays until they change it again or the app relaunches
- Not persisted to the database — resets to default on every app relaunch

### Non-goals

- **No effect on scoring or selection logic.** A collapsed lane still participates in the global sort_order for the list order bonus. The Encounters overlay still fires on Lane 1 regardless of whether Lane 1 is visually collapsed. Completions and XP still flow normally.
- **No per-user persistence.** The default is intentionally refreshed every session as an "ease into your day" pattern.

---

## Item 3: Auto-Accomplishment for One-Off Sagas + Accomplishment Refactor

### Context

Accomplishments are permanent records of completed campaigns, displayed on the Character tab. The table currently assumes a single source type (campaigns). Finishing a one-off saga is also a meaningful milestone (e.g., "Move to New Apartment"), but it doesn't create an accomplishment today — the saga's completion bonus XP fires, but there's no permanent record on the Character tab.

### Goal

1. Refactor the accomplishment table to support multiple source types generically
2. Automatically create an accomplishment when a one-off saga completes

### Scope: Accomplishment Table Refactor

**Current schema:**

| Field | Type | Description |
|---|---|---|
| id | UUID | |
| campaign_id | UUID (nullable) | FK to campaign |
| campaign_name | String | Snapshot at completion |
| completed_at | Timestamp | |
| bonus_xp | Integer | XP awarded on campaign completion |

**New schema:**

| Field | Type | Description |
|---|---|---|
| id | UUID | |
| source_type | String | "campaign" or "saga" (extensible string enum) |
| source_id | UUID (nullable) | FK to the source. Nullable — survives source deletion. |
| name | String | User-facing label. Snapshot at completion. |
| completed_at | Timestamp | |

**Changes:**
- Drop `campaign_id`, `campaign_name`, `bonus_xp`
- Add `source_type`, `source_id`, `name`
- Migrate existing accomplishments: `source_type = "campaign"`, `source_id = old campaign_id`, `name = old campaign_name`

**Why drop bonus_xp:** Accomplishments are about commemorating what was done, not tracking XP. XP is already awarded at the source completion (campaign bonus XP, saga completion bonus). The accomplishment row doesn't need to duplicate that state.

**Why source_type + source_id (vs. just name + date):** Enables dedup (don't create two accomplishments for the same source) and positions us for future source types like smart achievements.

### Scope: Auto-Accomplishment for One-Off Sagas

**Trigger:** When a one-off saga completes (last step is marked done).

- **One-off sagas only.** Recurring sagas do not trigger this — finishing a recurring run is the cycle turning over, not a permanent milestone.
- **One accomplishment per saga.** If the dedup key `(source_type, source_id)` already exists, do not create a duplicate.

**Accomplishment contents:**
- `source_type`: "saga"
- `source_id`: the saga's UUID
- `name`: the saga's name at completion (snapshot)
- `completed_at`: the completion timestamp

**User visibility:**
- The new accomplishment appears in the Accomplishments section on the Character tab, mixed with campaign accomplishments, ordered by `completed_at` descending
- No special visual treatment — sagas and campaigns look the same in the list (the goal is a unified "things I've done" view)

### Non-goals

- **No XP changes.** The saga completion bonus already fires separately and is unchanged.
- **No new display affordances.** Just one more row in the existing Accomplishments list.
- **Recurring sagas not affected.** They continue to work exactly as today.
