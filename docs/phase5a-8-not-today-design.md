# Phase 5A-8: "Not Today" Button — Design

## Requirements Summary

A "Not Today" button on the quest giver lets the user dismiss a quest from the candidate pool for the rest of the day. For when you know you can't or won't do something today.

1. **Location:** Quest giver lanes only (not quest list, not encounters overlay)
2. **Scope:** Works on both regular quests and saga steps. Dismissing a saga step effectively dismisses the saga for the day (the next step can't become active until the current one is completed)
3. **Effect:** Quest is removed from the candidate pool entirely — behaves as "not due" rather than being pushed down the list. Does not count as a skip. Does not affect scoring.
4. **Quest list appearance:** Dismissed quests show ⏾ before the title with cooldown styling (dimmed), matching not-due quest appearance. Action buttons (⚔ ✓) remain available — if the user completes a dismissed quest from the quest list, it counts normally. The dismissal only affects the quest giver.
5. **Reset:** Midnight, local time. Persisted in the database so dismissals survive app restart.
6. **Encounters overlay:** Not affected for now — may still show dismissed quests.

## Data Model

### New table: `not_today`

| Field | Type | Description |
|---|---|---|
| quest_id | TEXT PRIMARY KEY | FK to Quest (the dismissed quest) |
| dismissed_date | TEXT | Local date (YYYY-MM-DD) when dismissed |

Simple: one row per dismissed quest. Primary key prevents duplicates. The `dismissed_date` column enables cleanup — rows with a date before today are stale.

## Backend

### Migration

Create the `not_today` table in `init_db`.

### New functions

1. **`dismiss_quest_today(conn, quest_id)`** — Insert or replace into `not_today` with today's local date. Returns Ok.

2. **`get_dismissed_today(conn) -> HashSet<String>`** — Returns quest IDs dismissed today. Query: `SELECT quest_id FROM not_today WHERE dismissed_date = ?` with today's local date.

3. **`clear_stale_dismissals(conn)`** — Delete rows where `dismissed_date < today`. Called during `init_db` (app startup) so stale dismissals are cleaned up on launch.

### Quest giver integration

In `get_quest_scores`, after building the eligible quest list and eligible saga steps, filter out any quest whose ID is in the dismissed set. This means:

1. Call `get_dismissed_today` at the start of `get_quest_scores`
2. Add a filter to `eligible`: `.filter(|q| !dismissed.contains(&q.id))`
3. Add a filter to `eligible_saga`: `.filter(|(q, ..)| !dismissed.contains(&q.id))`

The dismissed quests simply don't enter the candidate pool — no scoring changes needed.

### Quest list integration

`get_quest_list` needs to indicate which quests are dismissed so the frontend can apply ⏾ styling. Two options:

**Option A:** Add a `dismissed_today: bool` field to `QuestListItem`.
**Option B:** Return the dismissed set separately and let the frontend match.

**Recommend A** — it's simpler for the frontend. Add `dismissed_today: bool` to `QuestListItem`. In `get_quest_list`, load the dismissed set and set the flag for each item (checking the quest ID for regular quests, and the current step's quest ID for saga slots).

### Commands

- `dismiss_quest_today` — wrapper, takes `quest_id: String`
- No separate command for `get_dismissed_today` — it's embedded in `get_quest_list` and `get_quest_scores`

## Frontend

### Quest giver — "Not Today" button

Add a third button to each lane's quest display, alongside the existing "Quest Now" and "Something Else" buttons:

```
[Quest Now]  [Something Else]  [Not Today]
```

**Behavior:** Calls `invoke("dismiss_quest_today", { questId })`, then refreshes the lane (re-calls `get_next_quest` to show the next candidate). The dismissed quest won't appear again today in any lane.

**Styling:** Visually distinct from "Something Else" — more muted/secondary, since it's a less common action. A moon icon (⏾) in the button text would tie it to the quest list indicator.

**Empty lane after dismissal:** If dismissing the last eligible quest in a lane empties it, the lane shows its empty state ("The walls are secure" etc.).

### Quest list — dismissed quest styling

In `renderQuestRow` and `renderSagaSlotRow`, when the item's `dismissedToday` flag is true:

1. Apply `quest-cooldown` styling (dimmed) regardless of the quest's actual due state
2. Show ⏾ before the title: `<span class="not-today-icon">⏾</span>` before the quest title span
3. Action buttons remain (⚔ ✓) — user can still complete dismissed quests from the quest list

For saga slots: check the slot's step quest ID against the dismissed set (via `dismissedToday` on the `QuestListItem`).

### Quest giver — `get_next_quest` integration

`get_next_quest` calls `get_quest_scores` which already filters out dismissed quests. No changes needed to `get_next_quest` itself — the filtering happens upstream in the scoring.

## Implementation Steps

1. **Backend + quest giver button** — `not_today` table, `dismiss_quest_today` function, exclusion from `get_quest_scores`, cleanup on startup. "Not Today" button on quest giver lanes. Testing: dismiss a quest, verify it doesn't reappear in any lane. Restart app, verify dismissal persists. Wait for midnight (or manually clear), verify quest returns.

2. **Quest list styling** — `dismissed_today` flag on `QuestListItem`, ⏾ icon and cooldown styling on the quest list for dismissed quests. Testing: dismiss from quest giver, switch to quest list, verify ⏾ and dimmed styling. Complete the quest from quest list, verify it works normally.

### Summary

Two vertical slices. Step 1 delivers the core feature (dismiss from quest giver, persisted, excluded from candidate pool). Step 2 adds the visual indicator on the quest list. Both are small — could potentially be one step.
