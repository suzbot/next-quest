# Phase 5A-5/6/7: Saga Visibility and Reordering — Requirements

**Status:** Draft

**Goal:** Make saga steps visible, filterable, and scorable alongside regular quests. Give users control over saga priority order.

---

## Pain points

1. Can't see how saga steps will be scored until the quest giver shows them
2. Can't see saga steps when filtering the quest list (e.g., "show me outdoor afternoon quests") — active saga steps are invisible outside the saga tab and quest giver
3. Want to see the full ranked list of what the quest giver will offer, to verify priorities and pick by mood — currently have to skip through one at a time

---

## 1. Debug scoring on saga tab (5A-5)

### User story

> I want to see the same score breakdown on saga steps that I see on quest list rows, so I can understand why certain saga steps surface before others.

### Behavior

When debug scoring is enabled, expanded saga steps show their score breakdown — same format as the quest list debug display. This requires calling `get_quest_scores` for the relevant lane and matching saga step IDs.

---

## 2. Saga reordering + scoring (5A-6)

### User story

> I have several active sagas and I want to control their relative priority. Laundry should come before Taxes in the quest giver. I want to drag sagas into my preferred order, and have that order affect scoring.

### Behavior

- Sagas gain drag-and-drop reordering on the saga tab (same pattern as quest reordering and saga step reordering)
- Saga `sort_order` feeds into the scoring algorithm for saga steps. Currently saga steps get a fixed `list_order_bonus = 1.0`. Instead, use the saga's sort_order relative to all sagas, same way quest sort_order works relative to all quests.
- Higher sort_order (top of saga list) = higher list_order_bonus for that saga's active step

### What changes

- Sagas already have a `sort_order` field — it just isn't user-editable (set at creation time)
- Add `reorder_sagas` backend function (same pattern as `reorder_quests`)
- Frontend: drag-and-drop + Alt+Arrow on the saga tab
- Scoring: saga step `list_order_bonus` changes from fixed 1.0 to `saga.sort_order / global_max_saga_sort_order`

### Open question

Should the saga list_order_bonus range be 0–1.0 (relative to other sagas) or remain at 1.0 (matching top-of-quest-list)? If relative, a bottom-of-list saga scores lower than a top-of-list regular quest, which might be undesirable. If fixed at 1.0, reordering sagas doesn't actually affect scoring.

Possible approach: `0.5 + 0.5 * (sort_order / max_sort_order)` — ranges from 0.5 to 1.0. Bottom saga still gets a meaningful boost (they're committed work), but top saga gets more.

---

## 3. Saga steps on quest list (5A-7)

### User story

> I'm filtering the quest list for "outdoor afternoon quests" and I want to see saga steps too — maybe the next step of my Walking saga is relevant. I also want to see saga steps' scores and importance alongside regular quests.

### Behavior

- Active saga steps (one per active saga with an active run) appear on the quest list alongside regular quests
- They are read-only — clicking the title or entering edit mode navigates to the saga tab (or shows a "view on Saga tab" link)
- They show the same info as regular quests: title, importance, difficulty, TOD, DOW, last done, tags, debug scoring
- They are filterable and searchable — difficulty filter, importance filter, fuzzy search, TOD, DOW, due filter all apply
- They have a visual indicator showing they're saga steps (e.g., saga name badge, or different row styling)
- The ⚔ and ✓ buttons work (Quest Now starts timer, Done completes the step)

### What changes

- `get_quests` gains an option or a separate call to include active saga steps in the returned list
- Frontend merges saga steps into the quest list rendering
- Saga steps are marked with their saga name for identification
- Edit click navigates to saga tab instead of opening inline edit

---

## Implementation order

1. **5A-5: Debug scoring on saga tab** — quick win, no backend changes
2. **5A-6: Saga reordering** — backend reorder function, frontend drag-and-drop, scoring adjustment
3. **5A-7: Saga steps on quest list** — backend merge, frontend rendering with read-only rows

---

## Future (deferred)

- **Expandable queue per lane** — show full ranked candidate list on quest giver. May not be needed if quest list filtering covers the use case. Revisit after 5A-7.
