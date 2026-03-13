# Phase 1 Group A: "Quest Giver Core" — Design

Covers: quest selection, local midnight reset, Next Quest view, break timer,
Quest Now from list, and link code consolidation.

## Quest Selection

**State lives in the frontend.** The backend already returns all quests with
`is_due` computed. The frontend filters and iterates — no new backend endpoints
needed for selection.

### Algorithm

```
1. Filter active quests where is_due === true
2. Iterate in sort_order (existing list order)
3. "Something Else" advances the index
4. When all due quests exhausted: fall back to the active quest with the
   oldest last_completed (regardless of cycle)
5. Wrap around when the index exceeds the full list
```

Frontend state:
- `dueQuests` — filtered list of due quests, rebuilt on each `loadAll()`
- `nextQuestIndex` — current position in dueQuests (or fallback)
- Resets to 0 when the list changes (quest completed, new quest added, etc.)

## Local Midnight Reset

**Problem:** `chrono_now()` produces UTC. Due calculations compare calendar
days, so a quest completed at 11pm local time may show as "today" in UTC
when it's already "tomorrow" locally.

**Fix:** Add a `local_date_today()` function using `libc::localtime_r` to
get local time components. Pass the local date (not UTC) to `compute_is_due`.

- **Stored timestamps stay UTC.** `completed_at` is a point in time — UTC
  is correct for storage and display.
- **Due comparison uses local dates.** Both "now" and "last completed" are
  converted to local calendar dates before comparing.

### Changes to `compute_is_due`

Currently receives `now` as a UTC ISO string. Change to receive a
`local_today_days: i64` (pre-computed local date as day count). For
`last_completed`, convert the stored UTC timestamp to local date before
extracting days.

This means `date_to_days` needs a local-time variant, or we convert the
UTC timestamp to local time before extracting the date.

**No new crates.** `libc::localtime_r` is C standard library — zero-cost
binding, already transitively present.

## Next Quest View (Frontend)

**Third nav tab.** Nav becomes: [Quest Giver] [Quests] [Character].
App defaults to Quest Giver tab on launch.

### Two modes

**Selection mode** (no timer running):
- Quest name displayed prominently
- [Done] — completes the quest, awards XP, refreshes selection
- [Quest Now] — starts the break timer, switches to timer mode
- [Something Else] — advances to next quest in selection order
- If no quests exist: empty state message

**Timer mode** (Quest Now was pressed):
- Quest name displayed
- Elapsed timer (MM:SS, advancing to HH:MM:SS)
- [Done] — stops timer, completes quest, awards XP, returns to selection mode
- [Cancel] — stops timer, no completion, returns to selection mode

### State

```javascript
let nextQuestIndex = 0;      // position in due quest list
let timerQuestId = null;      // quest being worked on (null = selection mode)
let timerStart = null;        // Date.now() when Quest Now pressed
let timerInterval = null;     // setInterval handle
```

### Refresh behavior

When `loadAll()` runs (after any data change), the due quest list is
rebuilt. If the current `nextQuestIndex` is out of bounds, reset to 0.
If the timer quest was completed or deleted, clear the timer.

## Break Timer

**Pure frontend.** `setInterval` at 1-second resolution, displays elapsed
time since `timerStart`.

- Not persisted — closing and reopening the app loses the timer. Acceptable
  for now.
- No XP implications — just a clock.

## Quest Now from List View

Add a `[Quest Now]` button to each active quest row in the list view.

Clicking it:
1. Sets `timerQuestId` to that quest's ID
2. Sets `timerStart` to `Date.now()`
3. Starts the timer interval
4. Switches to the Quest Giver tab (timer mode)

Same Done/Cancel flow as if Quest Now were pressed from the Quest Giver view.

## Link Code Consolidation

`get_quest_links` (public, returns `QuestLinks` struct) and
`load_quest_link_ids` (private, returns tuple) run identical SQL queries.

**Fix:** Have `get_quest_links` delegate to `load_quest_link_ids`:

```rust
pub fn get_quest_links(conn: &Connection, quest_id: String) -> Result<QuestLinks, String> {
    let (skill_ids, attribute_ids) = load_quest_link_ids(conn, &quest_id)?;
    Ok(QuestLinks { skill_ids, attribute_ids })
}
```

Removes ~18 lines of duplicated query code.

## What's NOT Changing

- **Backend quest APIs** — no new endpoints for selection. Existing
  `get_quests`, `complete_quest` are sufficient.
- **Data model** — no schema changes, no new tables.
- **XP engine** — no changes to calculation or distribution.
- **Character view** — unchanged.

## Implementation Order

Suggested step sequence (each step is one buildable, testable slice):

1. **Link consolidation + local midnight reset** — backend-only, with tests.
   Clean foundation before building new frontend.
2. **Next Quest view (selection mode)** — new tab, selection logic, Done and
   Something Else. No timer yet.
3. **Break timer + Quest Now** — timer mode in Quest Giver view, Quest Now
   button in list view.
