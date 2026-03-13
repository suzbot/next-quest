# Step Spec: Phase 1, Step 1 — "The Quest Giver Appears"

## Goal

The app gets its core identity shift: instead of opening to a list, it opens
to a quest giver that picks one quest and says "do this." Backend gets local
midnight reset and link code cleanup. Frontend gets the new Quest Giver view
with selection logic, Done, and Something Else.

## Scope

### Backend: Link Consolidation

`get_quest_links` (line 802) duplicates the queries in `load_quest_link_ids`
(line 937). Replace the body of `get_quest_links` with a call to
`load_quest_link_ids`:

```rust
pub fn get_quest_links(conn: &Connection, quest_id: String) -> Result<QuestLinks, String> {
    let (skill_ids, attribute_ids) = load_quest_link_ids(conn, &quest_id)?;
    Ok(QuestLinks { skill_ids, attribute_ids })
}
```

Existing tests should continue to pass — behavior is identical.

### Backend: Local Midnight Reset

**Problem:** `compute_is_due` compares calendar days using UTC timestamps.
A daily quest completed at 11pm local time won't show as due until the next
UTC day boundary, not the local midnight.

**Fix:**
- Add a `local_date_to_days()` function that converts a UTC Unix timestamp
  (or ISO string) to local calendar days using `libc::localtime_r`.
- Change `compute_is_due` to compare local calendar days instead of UTC
  calendar days.
- `chrono_now()` and stored timestamps stay UTC — only the due comparison
  changes.

`libc` is not a new dependency — it's C standard library bindings, already
transitively present. Add it as an explicit dependency in Cargo.toml only if
the compiler requires it.

**Tests:** Add/update tests for `compute_is_due` covering:
- Quest completed late evening, checked after local midnight
- Existing `is_due_logic` and `is_due_month_boundary` tests still pass

Note: tests that need to verify local-time behavior may need to use
the local-date functions directly rather than mocking timezone. Keep tests
simple — if timezone-dependent tests are awkward, test the conversion
functions in isolation and keep the integration behavior tested via the app.

### Frontend: Quest Giver View

**Navigation** becomes three tabs: `[Quest Giver]` `[Quests]` `[Character]`.
App defaults to Quest Giver on launch.

**Quest Giver view layout (selection mode):**
```
[Quest Giver] [Quests] [Character]

         QUEST GIVER

    Take a shower

    [Done]  [Something Else]
```

Centered, prominent quest name. Minimal chrome — this is the "one thing
to do right now" view.

**Selection logic (in JavaScript):**
1. On `loadAll()`, build `dueQuests` = active quests where `is_due === true`,
   in existing sort order
2. Display `dueQuests[nextQuestIndex]`
3. **Done** — calls `complete_quest`, shows XP flash, refreshes data,
   resets `nextQuestIndex` to 0
4. **Something Else** — increments `nextQuestIndex`. If past the end of
   `dueQuests`, fall back to the active quest with the oldest
   `last_completed`. If that's also exhausted, wrap to index 0.
5. When `loadAll()` rebuilds the list and `nextQuestIndex` is out of bounds,
   reset to 0

**Empty states:**
- No quests at all: "No quests yet. Add some in the Quests tab."
- No quests due: "All caught up. Rest well, adventurer." (or similar)

**Frontend state:**
```javascript
let dueQuests = [];          // rebuilt on each loadAll()
let nextQuestIndex = 0;      // position in dueQuests
```

## NOT in this step

- Break timer / Quest Now button (Step 2)
- Quest Now from list view (Step 2)
- Menu bar / system tray (Group B)
- Overlay interruption (Group C)
- XP tuning

## Done When

- `get_quest_links` delegates to `load_quest_link_ids` — no duplicate queries
- Due calculation uses local calendar days, not UTC
- App opens to Quest Giver tab
- Quest Giver shows the first due quest in list order
- Done completes the quest and shows XP earned
- Something Else cycles through due quests, falls back to longest-ago, wraps
- Empty states display when no quests exist or none are due
- All existing tests pass, new local-time tests pass
- `cargo test` clean, app builds and runs
