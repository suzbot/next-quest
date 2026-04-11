# Phase 5D Group 1 — Requirements

Small-to-medium polish items. Items 1 (reset skips button) and 2 (show/hide lanes) have been implemented. Items 3 (auto-accomplishment for one-off sagas) and 4 (overlay lane fallback) are next.

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

---

## Item 4: Overlay Lane Fallback

### Context

The Encounters overlay (Call to Adventure) currently surfaces Lane 1 (Castle Duties) quests — daily-recurring things like pills, meals, exercise. When Lane 1 has nothing to offer (nothing due, or everything due has been "Not Today"'d), the overlay shows nothing. On those days, the overlay silently stops being useful until tomorrow.

### Goal

When Lane 1 is empty, the overlay falls through to Lane 2 (Adventures), then to Lane 3 (Royal Quests). The overlay remains themed as monster encounters regardless of which lane the quest came from.

### Behavior

**Trigger — each overlay poll evaluates lanes in order:** Lane 1 → Lane 2 → Lane 3.
- The overlay uses the first lane that has a quest to offer
- "Nothing to offer" = the lane's quest giver currently has zero quests to display (no due quests, or all due quests have been "Not Today"'d for the day)
- If all three lanes are empty, the overlay shows nothing (same as today)

**Per-poll re-evaluation:**
- Each overlay poll re-runs the Lane 1 → Lane 2 → Lane 3 evaluation from scratch
- Fallback state does not persist between polls. If a new Lane 1 quest becomes due between polls, the next poll picks it up and stops falling through.

**Something Else / Run in the overlay:**
- Cycles within the lane that was selected for the current poll
- Does not re-evaluate fallback mid-cycle; the next natural poll handles that

**Visuals:**
- The overlay is always themed as monster encounters, regardless of which lane the quest came from
- No lane-specific art or flavor text in the overlay
- No visual indicator in the overlay of which lane produced the quest

### Non-goals

- **No change to the main app's Next Quest tab.** It continues to show all three lanes stacked vertically with independent quest givers. Fallback is overlay-only.
- **No Lane 3 ceiling.** Royal Quests can be offered by the overlay as a fallback; there's no rule limiting how many appear per day.
- **No change to scoring within a lane.** Each lane still scores and selects quests exactly as today. Fallback only changes which lane's selection the overlay consumes.
- **No change to Quest Now / timer behavior.** Starting a quest from the overlay works the same regardless of which lane the quest came from.
