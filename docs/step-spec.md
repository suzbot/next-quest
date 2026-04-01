# Step Spec: Phase 5A-8 — "Not Today" Button

## Goal

A "Not Today" button on the quest giver removes a quest from the candidate pool for the rest of the day. Persisted across app restart, resets at midnight. Dismissed quests show ⏾ with cooldown styling on the quest list.

---

## Substep 1: Backend + quest giver button

### Migration

Add to `init_db`:

```sql
CREATE TABLE IF NOT EXISTS not_today (
    quest_id TEXT PRIMARY KEY,
    dismissed_date TEXT NOT NULL
);
```

Add stale dismissal cleanup after table creation:

```rust
conn.execute(
    "DELETE FROM not_today WHERE dismissed_date < ?1",
    rusqlite::params![local_today_str()],
).ok();
```

`local_today_str()` returns today's local date as `YYYY-MM-DD`. Add this helper alongside the existing `local_today_days()`.

### New functions in db.rs

1. **`dismiss_quest_today(conn, quest_id: &str)`** — `INSERT OR REPLACE INTO not_today (quest_id, dismissed_date) VALUES (?1, ?2)` with today's local date.

2. **`get_dismissed_today(conn) -> HashSet<String>`** — `SELECT quest_id FROM not_today WHERE dismissed_date = ?1` with today's local date.

### Quest giver exclusion

In `get_quest_scores`, after building the eligible and eligible_saga lists, load the dismissed set and filter both:

```rust
let dismissed = get_dismissed_today(conn).unwrap_or_default();

// Add to existing filter chains:
.filter(|q| !dismissed.contains(&q.id))          // regular quests
.filter(|(q, ..)| !dismissed.contains(&q.id))    // saga steps
```

### Command

`dismiss_quest_today` wrapper in commands.rs taking `quest_id: String`. Register in main.rs.

### Frontend — "Not Today" button

Add a third button to each lane's action area in `renderLane`:

```html
<button onclick="laneNotToday('${lane.key}', '${quest.id}')">⏾ Not Today</button>
```

**New function** `laneNotToday(laneKey, questId)`:

1. Call `invoke("dismiss_quest_today", { questId })`
2. Call `renderLane(lane, true)` to refresh the lane with the next candidate (force refresh since the quest changed)

Style the button more muted than "Quest Now" and "Something Else" — secondary/tertiary appearance.

### Tests

1. `dismiss_quest_today_persists` — Dismiss a quest. Call `get_dismissed_today`. Verify the quest ID is in the set.
2. `dismiss_quest_today_idempotent` — Dismiss the same quest twice. No error, set still contains it once.
3. `stale_dismissals_cleaned` — Insert a dismissal with yesterday's date directly. Call cleanup. Verify it's gone but today's dismissals remain.
4. `dismissed_quest_excluded_from_scores` — Create two due quests. Dismiss one. Call `get_quest_scores`. Verify only the non-dismissed quest appears.
5. `dismissed_saga_step_excluded_from_scores` — Create a saga with a due step. Dismiss the step. Call `get_quest_scores`. Verify the saga step doesn't appear.

**Testing checkpoint:** Build app. On the quest giver, click "⏾ Not Today" on a quest. Verify it disappears from the lane and a new quest appears. Verify it doesn't reappear in any lane. Restart the app — verify it's still dismissed. Verify midnight reset (or manually delete the row and reload).

---

## Substep 2: Quest list ⏾ styling

### Backend — `dismissed_today` flag

Add `dismissed_today: bool` to `QuestListItem`:

```rust
pub struct QuestListItem {
    pub item_type: String,
    pub quest: Option<Quest>,
    pub saga_slot: Option<SagaSlot>,
    pub sort_order: i32,
    pub dismissed_today: bool,
}
```

In `get_quest_list`, load the dismissed set once, then set `dismissed_today` for each item:
- Regular quests: `dismissed.contains(&quest.id)`
- Saga slots: `dismissed.contains(&display_step.id)`

### Frontend — quest list rendering

In `renderQuestRow`, when the item's `dismissedToday` is true:

1. Override `stateClass` to `"quest-cooldown"` (dimmed regardless of actual due state)
2. Add `<span class="not-today-icon">⏾</span>` before the quest title span

In `renderSagaSlotRow`, same treatment when the parent `QuestListItem.dismissedToday` is true. This requires passing the flag through — either add it to the `SagaSlot` struct or pass it alongside.

Simplest: pass `dismissedToday` as a parameter to `renderSagaSlotRow`:

```javascript
return renderSagaSlotRow(item.sagaSlot, item.dismissedToday);
```

CSS for the icon — subtle, matching the muted cooldown text color:

```css
.not-today-icon { color: #888; margin-right: 3px; font-size: 12px; }
```

### Frontend — filter integration

`passesFilters` for dismissed quests: the "Due" filter should exclude dismissed quests (they're functionally not due). Check `dismissedToday` in `passesFilters` — if the due filter is active and the item is dismissed, filter it out.

### Tests

No new backend tests — the `dismissed_today` flag is derived from `get_dismissed_today` which is already tested.

**Testing checkpoint:** Build app. Dismiss a quest from the quest giver. Switch to quest list. Verify ⏾ appears before the title, row is dimmed. Verify action buttons (⚔ ✓) still work on the dismissed quest. Complete the dismissed quest — verify XP awards normally. Turn on "Due" filter — verify dismissed quests are hidden.

---

## NOT in this step

- "Not Today" on the encounters overlay
- "Not Today" on the quest list (direct dismiss without going through quest giver)
- Undo/un-dismiss mechanism

## Done When

"Not Today" button appears on quest giver lanes. Clicking it removes the quest from the candidate pool for the day. Dismissals persist across app restart and reset at midnight. Dismissed quests show ⏾ with cooldown styling on the quest list. "Due" filter excludes dismissed quests. `cargo test` passes.
