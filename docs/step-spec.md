# Step Spec: Phase 2D, Step 1 — Quest List Reorganization

## Goal

Reorganize the quest list into a cleaner, more scannable layout with expandable detail rows, icon action buttons, and fixed-width meta columns.

## Scope

### Quest Row Layout (collapsed, single line)

Left to right: **▸ | Last Done | Title | Difficulty | Cycle | ⚔ | ✓**

- **▸** expand toggle — clicks to reveal/hide detail row
- **Last Done** — fixed-width, right-aligned (e.g. "Today 9am", "Mar 14", blank if never)
- **Title** — flex, click to enter edit mode (same as today)
- **Difficulty** — fixed-width, right-aligned, color-coded when due
- **Cycle** — fixed-width, right-aligned (e.g. "Every 3 days", "One-off")
- **⚔** — Quest Now button (replaces text "Quest Now")
- **✓** — Done button (replaces text "Done")

### Quest Row Detail (expanded)

Toggled by ▸. Shows below the collapsed row:

- Skills and attributes linked to the quest
- Indented to align with title column

### Delete Button

- Remove from display row
- Add to edit mode row (alongside Save and Esc)

### Styling

- Last Done, Difficulty, Cycle: fixed widths, right-aligned so they form clean columns across rows
- Cooldown rows: greyed out (same as today)
- Inactive rows: dimmed (same as today)
- Difficulty colors: same color scheme as today, only when due

### What Changes

**ui/index.html:**
- `renderQuestRow()` — new layout with expand toggle, fixed-width meta, icon buttons
- `renderEditMode()` — add Del button
- CSS: new classes for fixed-width columns, expand toggle, detail row

### What Doesn't Change

- Drag-to-reorder behavior (pointer-down on row)
- Edit mode flow (click title → inline edit)
- Quest state logic (due/cooldown/inactive)
- Backend — no Rust changes

## NOT in this step

- Changing what meta is displayed (same fields as today, just reorganized)
- Adding new meta fields (that's 2F)
- Tooltips on buttons

## Done When

- Quest list rows show: ▸ | Last Done | Title | Difficulty | Cycle | ⚔ | ✓
- Difficulty and Cycle columns are visually aligned across rows
- ▸ expands to show skills/attributes below the row
- ⚔ starts Quest Now, ✓ completes quest (same behavior as today)
- Del button only appears in edit mode
- Drag reorder still works
- All existing tests pass
