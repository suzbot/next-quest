# Step Spec: Phase 2G.1-7 — Saga Step Resequencing

## Goal

Full drag-and-drop and keyboard resequencing for saga steps. Any step can be moved to any position within its saga.

---

## Keyboard resequencing

**Alt+ArrowUp / Alt+ArrowDown** moves a step one position within the saga.

Implementation:
- `handleStepKey(event, stepId, sagaId)` on each step row
- On Alt+Arrow: rebuild the full step ID list with the moved step shifted one position, call `reorder_saga_steps`, reload steps, re-render, re-focus the moved step
- Uses `reorder_saga_steps` (takes full ordered list), NOT sort_order pair swaps — avoids the neighbor-swap-back bug from previous attempt
- Repeated presses keep moving in the same direction

**ArrowUp / ArrowDown** (without Alt) moves focus between step rows within the saga. Does not jump to saga rows or other sagas.

## Drag-and-drop resequencing

**PointerDown on a step row** initiates a drag (after 5px movement threshold).

Implementation:
- `onStepPointerDown(event, stepId, sagaId)` on each step row
- Drop target finder scoped to `.step-list` within the expanded saga — only step rows are valid targets, not saga rows
- On drop: rebuild the full step ID list with the dragged step inserted at the drop position, call `reorder_saga_steps`, reload, re-render
- Visual feedback: reuse existing `.dragging`, `.drop-above`, `.drop-below` CSS classes
- `stopPropagation` on the pointer event to prevent saga-level handlers from firing

## Step row requirements

- Each step `<li>` needs `tabindex="0"` for keyboard focus
- `onkeydown` handler for keyboard resequencing
- `onpointerdown` handler for drag initiation
- Buttons within the row (✓, ⚔, expand toggle) must not trigger drag — check `e.target.closest("button")` same as quest list

## After reorder

- Call `reorder_saga_steps` with saga_id and the new ordered list of step IDs
- Reload `sagaSteps[sagaId]` from backend
- Re-render sagas (step numbers update to reflect new order)
- Re-focus the moved step (keyboard) or clear drag state (drag)

## Scoping

- All interactions are scoped to the expanded saga's step list
- `getStepDropTarget(sagaId, clientY)` searches only within the step list of the specified saga
- Keyboard focus navigation (`focusAdjacentStep`) only moves between siblings in the same step list

## Testing

- Add 4+ steps to a saga
- Alt+ArrowDown on step 1 → becomes step 2, step numbers update
- Alt+ArrowDown again → becomes step 3
- Alt+ArrowUp → back to step 2
- Drag step 4 to position 1 → step numbers update, order persists after tab switch
- Buttons (✓, ⚔) still work after reorder
- Focus stays on the moved step after keyboard reorder
