# Step Spec: Phase 0, Step 4 — "Reorder Quests"

## Goal

Active quests can be reordered via keyboard and drag-and-drop. Sort order
persists across restarts.

## Scope

**Backend:**
- Implement `reorder_quests` command — accepts a list of `{id, sort_order}`
  pairs and batch-updates sort_order on the quest table.
- Validate: all provided IDs must exist. Reject the batch if any are missing.

**Frontend — Keyboard reordering:**
- Alt+Up / Alt+Down moves the focused quest up/down in the active list.
- Focus follows the moved quest (it stays focused after the move).
- Arrow keys (Up/Down) move focus between quest rows without reordering.
- Only active quests are reorderable. Inactive (greyed-out) one-offs cannot
  be moved.

**Frontend — Drag-and-drop reordering:**
- Drag an active quest row to a new position in the active list.
- Drop indicator shows where the quest will land.
- Only active quests are draggable. Inactive one-offs are not draggable.
- No external drag-and-drop library — vanilla HTML5 drag events.

**Frontend — Focus navigation:**
- Up/Down arrow keys move focus between quest rows (tab already works but
  arrows are more natural for a list).
- Focus does not leave the active quest section via arrows (stops at top/bottom).

## NOT in this step

- Reordering completions (they stay reverse chronological, always)
- Drag between sections (active ↔ completions)
- Touch/mobile drag support
- Undo reorder

## Implementation Notes

- `sort_order` is already stored as an integer on each quest. Higher = more
  prominent (displayed first). The reorder command just reassigns these values.
- After a reorder, the frontend calls `get_quests` to re-render with the
  updated order (same pattern as all other mutations).
- Keyboard reorder: on Alt+Up/Down, swap sort_order values of the focused quest
  and its neighbor, then call `reorder_quests` with the two affected IDs.
- Drag reorder: on drop, recalculate sort_order for all active quests based on
  their new DOM positions, then call `reorder_quests` with the full list.

## Done When

- Alt+Up / Alt+Down moves an active quest up/down and persists the new order
- Focus follows the moved quest after reorder
- Arrow keys navigate between quest rows
- Dragging a quest to a new position works and persists
- Inactive one-off quests cannot be moved or dragged
- Completions section is unaffected
- Tests cover: reorder_quests command, invalid ID rejection

## Next Step Preview

Step 5: Final cleanup — update design doc, mark Phase 0 complete, review for
any remaining gaps before Phase 0.5.
