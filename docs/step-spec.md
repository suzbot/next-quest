# Step Spec: Phase 5A-6c — Saga slot bug fixes + mixed reordering

## Goal

Fix saga slot action buttons on the quest list (done button, victory navigation). Then enable mixed drag-and-drop and keyboard reordering of quests and saga slots together.

---

## Substep 1: Fix saga slot done button on quest list

The ✓ button on quest list saga slots calls `completeSagaStep`, which is the saga tab's function — it updates saga tab DOM elements that don't exist on the quest list. The backend completion succeeds but the user sees no feedback and the quest list doesn't refresh.

**New function:** `completeListSagaStep(questId, sagaId, difficulty)`

Mirrors `completeQuest` but for saga slots on the quest list:

1. Call `invoke("complete_quest", { questId })`
2. Call `invoke("check_saga_completion", { sagaId })`
3. Find the row via `questList.querySelector('[data-id="${sagaId}"]')` (saga slot rows use saga ID as data-id)
4. Flash the row with difficulty color, show XP awards + level-ups inline (same pattern as `completeQuest`)
5. If saga completed, show saga celebration text inline (same pattern as `completeQuest` lines 1758–1768)
6. Check campaign progress for both the quest completion and (if saga completed) the saga completion — use `sagaId` directly instead of `quest?.saga_id`
7. Show campaign celebration if any
8. After 2500ms, call `loadAll()` — quest list refreshes and saga slot updates to show next step

**Wire up:** Change `renderSagaSlotRow` done button from:
```
onclick="completeSagaStep('${q.id}', '${slot.sagaId}')"
```
to:
```
onclick="completeListSagaStep('${q.id}', '${slot.sagaId}', '${q.difficulty}')"
```

**Testing checkpoint:** Build app. Complete a saga step from the quest list ✓ button. Verify: XP flash appears on the row, quest list refreshes to show next step. If saga run completes, verify celebration text appears inline. Verify saga tab's ✓ button still works as before.

---

## Substep 2: Fix victory flow navigation for saga steps

When ⚔ is clicked on a saga slot, `sagaStepQuestNow` duplicates the logic of `listQuestNow` + `startTimer` with minor differences. The victory celebration displays correctly, but the user isn't returned to the quest giver view afterward.

**Fix:** Replace `sagaStepQuestNow` to simply call `listQuestNow`:

```javascript
async function sagaStepQuestNow(questId) {
    await listQuestNow(questId);
}
```

This eliminates the duplicated timer setup code and ensures the exact same flow for saga steps and regular quests. `listQuestNow` calls `startTimer` → `showView("quest-giver")`, and the existing `timerDone` victory flow handles the rest.

**Secondary fix in `timerDone`:** Campaign progress for saga completions is skipped because `quest?.saga_id` is undefined (saga steps not in `cachedQuests`). Fix: use the saga ID from `check_saga_completion_for_quest` result instead:

```javascript
// Current (broken for saga steps):
if (sagaResult?.completed && quest?.saga_id) {
    const sagaCampaignResults = await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: quest.saga_id });

// Fixed:
if (sagaResult?.completed && sagaResult?.sagaId) {
    const sagaCampaignResults = await invoke("check_campaign_progress", { targetType: "saga_completions", targetId: sagaResult.sagaId });
```

Verify that `check_saga_completion_for_quest` returns `sagaId` in its response — if not, add it.

**Testing checkpoint:** Build app. Click ⚔ on a saga slot from the quest list. Timer starts, quest giver shows encounter. Click "Victorious!" — verify victory celebration displays, then fades to show the quest giver lanes with the next quest. Verify regular quest ⚔ from quest list still works identically.

---

## Substep 3: Backend — `reorder_list` replacing `reorder_quests`

**New struct:**

```rust
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListReorderItem {
    pub id: String,
    pub item_type: String, // "quest" or "saga"
    pub sort_order: i32,
}
```

**New function:** `reorder_list(conn, items: Vec<ListReorderItem>) -> Result<(), String>`

1. Begin transaction
2. For each item:
   - If `item_type == "quest"`: `UPDATE quest SET sort_order = ?1 WHERE id = ?2 AND saga_id IS NULL`
   - If `item_type == "saga"`: `UPDATE saga SET sort_order = ?1 WHERE id = ?2`
   - Error if no rows updated (invalid ID or type)
3. Commit

**Remove:** `reorder_quests` function from db.rs, its command wrapper from commands.rs, and its registration from main.rs.

**New command:** `reorder_list` wrapper in commands.rs. Register in main.rs.

**Tests:**

1. `reorder_list_swaps_quest_and_saga` — Create a quest (sort_order 2) and a saga (sort_order 1). Call `reorder_list` to swap them. Verify quest has sort_order 1 and saga has sort_order 2.
2. `reorder_list_invalid_id_errors` — Pass a nonexistent ID. Verify error, transaction rolled back.
3. `reorder_list_invalid_type_errors` — Pass a quest ID with type "saga". Verify error (no rows updated).

**Remove old tests:** `reorder_quests_swaps_order` and `reorder_quests_invalid_id_errors`.

**Testing checkpoint:** `cargo test` passes.

---

## Substep 4: Frontend — mixed reordering (drag + keyboard)

**Cached list for reordering:**

Add `cachedListItems` array, populated in `loadAll` from the `get_quest_list` results. Each entry:

```javascript
{
    id: item.quest?.id || item.sagaSlot?.sagaId,
    itemType: item.itemType,  // "quest" or "saga"
    sortOrder: item.sortOrder,
    isReorderable: /* active quest OR due saga */
}
```

For quests: `isReorderable = quest.active`. For saga slots: `isReorderable = slot.isSagaDue && !slot.isCompleted`.

**Keyboard reordering (`moveQuest`):**

Update to use `cachedListItems`:

1. Find moving item and neighbor in `cachedListItems` by ID (read from `dataset.id`)
2. Skip if either is not reorderable
3. Swap sort_orders
4. Call `invoke("reorder_list", { items: [{id, itemType, sortOrder}, {id, itemType, sortOrder}] })`
5. `loadAll()` and refocus

**Drag-and-drop (`onPointerDown` / `onUp`):**

Update to use `cachedListItems`:

1. `onUp`: build reordered list from `cachedListItems.filter(i => i.isReorderable)` instead of `cachedQuests.filter(q => q.active)`
2. Find dragged/target indices by ID in this list
3. Splice to new position
4. Redistribute sort_orders (same algorithm — collect existing sort_orders desc, assign to new positions)
5. Call `invoke("reorder_list", { items })` with full reordered list
6. `loadAll()`

**Saga slot row handlers:**

In `renderSagaSlotRow`, add `onpointerdown` and `onkeydown` to saga slot `<li>` when reorderable:

```javascript
const pointerHandler = slot.isSagaDue && !slot.isCompleted
    ? `onpointerdown="onPointerDown(event, '${slot.sagaId}')"` : '';
```

```javascript
onkeydown="handleQuestKey(event, '${slot.sagaId}', '${escapeAttr(slot.sagaName)}')"
```

`handleQuestKey` already delegates to `moveQuest` for shift+arrow — no changes needed to `handleQuestKey` itself.

**Testing checkpoint:** Build app.

1. Drag a saga slot above a regular quest — verify both sort_orders update, list re-renders in new order
2. Drag a regular quest below a saga slot — same verification
3. Shift+arrow a saga slot up past a quest — verify swap works
4. Shift+arrow a quest down past a saga slot — verify swap works
5. Verify not-due/completed saga slots can't be dragged or keyboard-reordered
6. Verify scoring reflects new positions (if debug scoring enabled)

---

## NOT in this step

- Saga tab reordering (quest list is source of truth — saga tab reflects it, read-only)
- Any changes to the quest giver or scoring algorithm

## Done When

Saga slot ✓ button on quest list shows XP feedback and refreshes the list. Victory flow after saga ⚔ from quest list returns to quest giver correctly. Quests and saga slots can be reordered together via drag-and-drop and keyboard. Not-due/completed items sit at bottom, not reorderable. `cargo test` passes.
