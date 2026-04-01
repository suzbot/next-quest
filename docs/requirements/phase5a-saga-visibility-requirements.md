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

> I have several active sagas and I want to control their relative priority. Laundry should come before Taxes in the quest giver. I want to drag sagas among my regular quests to set their priority.

### Behavior

- Sagas appear as slots on the quest list (see section 3). Users drag them up and down among regular quests.
- The quest list is the single source of truth for priority ordering — both quests and sagas.
- Saga `sort_order` shares a unified namespace with quest `sort_order` and feeds into scoring (`saga_sort_order / global_max_sort`).
- The saga tab displays sagas in sort_order but does NOT allow reordering — it has less context about relative position vs quests.

### What changes

- Sort_orders already unified (5A-6a). Sagas already have sort_order that affects scoring.
- `reorder_list` backend function accepts mixed quest/saga items
- Frontend: drag-and-drop on quest list handles both row types

---

## 3. Saga steps on quest list (5A-6)

### User story

> I'm filtering the quest list for "outdoor afternoon quests" and I want to see saga steps too — maybe the next step of my Walking saga is relevant. I also want to see saga steps' scores and importance alongside regular quests.

### Behavior

- Sagas appear as slots on the quest list alongside regular quests, showing their current active step
- They are read-only — clicking the title navigates to the saga tab
- They show the same info as regular quests: title, importance, difficulty, TOD, DOW, last done, tags, debug scoring
- They have a `[Saga: Name]` badge before the step title
- They are filterable and searchable — difficulty filter, importance filter, fuzzy search (including saga name), TOD, DOW, due filter all apply
- The ⚔ and ✓ buttons work (Quest Now starts timer, Done completes the step)
- When a saga run completes, the slot dims and shows step 1
- Recurring sagas between runs display like not-due recurring quests
- One-off completed sagas display like completed one-offs

### What changes

- `get_quests` gains an option or a separate call to include active saga steps in the returned list
- Frontend merges saga steps into the quest list rendering
- Saga steps are marked with their saga name for identification
- Edit click navigates to saga tab instead of opening inline edit

---

## Implementation order

1. ~~**5A-5: Debug scoring on saga tab**~~ ✅
2. ~~**5A-6a: Unified sort_order + scoring**~~ ✅ — sort_orders share namespace, saga step scoring uses saga sort_order
3. **5A-6b: Saga slots on quest list** — backend get_quest_list, frontend rendering with saga badge, filtering, ⚔/✓ buttons
4. **5A-7: Saga reordering on quest list** — backend reorder_list for mixed quest/saga items, frontend drag-and-drop handles both types

---

## Future (deferred)

- **Expandable queue per lane** — show full ranked candidate list on quest giver. May not be needed if quest list filtering covers the use case. Revisit after 5A-7.
