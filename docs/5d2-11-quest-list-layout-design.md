# 5D.2.11 Quest List Layout — Tech Design

Collapsible add form to separate the "add quests" and "filter quests" workflows.

---

## Problem

The add form and filter bar are both full-width horizontal rows with similar controls (text inputs, dropdowns, buttons). They sit adjacent with no visual distinction. This causes mis-clicks — reaching for a filter but typing in the add form, or vice versa.

The user's actual usage pattern: filters are used constantly, adding quests is rare except during batch-add sessions.

## Approach

Move the add form behind a toggle. Filter bar is always visible and gets sole ownership of the top-of-list position. The add form opens into a visually distinct panel below the filter bar when needed, and supports keyboard-first batch adding.

---

## HTML changes

### Filter bar: add the toggle button

Append a `+ New` button to the end of the existing `#filter-bar`:

```html
<button type="button" id="add-toggle-btn" onclick="toggleAddForm()">+ New</button>
```

Styled to match existing filter-bar buttons (10px font, small padding) but visually distinct — sits at the right end, separated by `margin-left: auto`.

### Add form: wrap in collapsible panel

Wrap the existing `#add-form` and `#add-links-row` in a new container div:

```html
<div id="add-panel" class="hidden">
  <form id="add-form">
    <!-- existing form contents unchanged -->
  </form>
  <div id="add-links-row" class="hidden"></div>
  <div class="add-panel-hint">Enter to add · Esc to close · Tab through fields</div>
</div>
```

The `#add-panel` div sits between the filter bar and the quest list. When hidden (default), the filter bar sits directly above the quest list with no gap.

### DOM order (quests-view)

```
#filter-bar          ← always visible
#add-panel           ← hidden by default, contains #add-form + #add-links-row + hint
#quest-list          ← always visible
```

This is a reorder from the current layout where `#add-form` precedes `#filter-bar`.

---

## CSS changes

### Add panel

```css
#add-panel {
  background: #a8a8a8;
  border: 1px solid #888;
  padding: 8px;
  margin-bottom: 4px;
}

.add-panel-hint {
  font-size: 9px;
  color: #666;
  margin-top: 4px;
}
```

The darker background (`#a8a8a8` vs the `#b4b4b4` page background) visually separates the add form from the filter bar. This is the same pattern used by the existing quest-edit inline rows.

### Toggle button

```css
#add-toggle-btn {
  margin-left: auto;
}

#add-toggle-btn.active {
  background: #999;
  border-style: inset;
}
```

`margin-left: auto` pushes the button to the right end of the filter bar's flex row, separating it from the filter controls. The `.active` state uses inset border (existing pattern from button:active) to show the panel is open.

### Filter bar spacing

Update `#filter-bar` margin-bottom from `4px` to `4px` (unchanged) — the add panel's own margin handles the gap when it's open. When closed, the filter bar's existing margin provides the gap to the quest list.

No other CSS changes needed. The `#add-form` styles remain as-is since the form itself doesn't change.

---

## JavaScript changes

### Toggle function

```javascript
function toggleAddForm() {
  const panel = document.getElementById("add-panel");
  const btn = document.getElementById("add-toggle-btn");
  const isOpen = !panel.classList.contains("hidden");

  panel.classList.toggle("hidden");
  btn.classList.toggle("active");

  if (!isOpen) {
    titleInput.focus();
  }
}
```

Follows existing toggle patterns (`classList.toggle("hidden")` used throughout the codebase).

### Keyboard shortcuts

Add a `keydown` listener on `document`:

```javascript
document.addEventListener("keydown", (e) => {
  // Only when quests view is active and no input/select is focused
  if (questsView.classList.contains("hidden")) return;
  const tag = document.activeElement?.tagName;
  if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;

  if (e.key === "n" || e.key === "N") {
    e.preventDefault();
    const panel = document.getElementById("add-panel");
    if (panel.classList.contains("hidden")) {
      toggleAddForm();
    } else {
      titleInput.focus();
    }
  }
});
```

For Escape (close the add panel), add handling inside the existing form context — only when focus is within the add panel:

```javascript
// Inside the add-panel or on the form
document.getElementById("add-panel").addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    const panel = document.getElementById("add-panel");
    if (!panel.classList.contains("hidden")) {
      toggleAddForm();
    }
  }
});
```

This scoping prevents Escape from closing the add form when the user is interacting with something else (e.g., a dropdown menu elsewhere).

### Submit behavior (unchanged)

The existing submit handler already clears the title, resets fields, and refocuses `titleInput` — this is already batch-add friendly. No changes needed. After adding a quest, focus returns to the name field and the panel stays open.

### View switching

When the user switches away from the quests view (via `showView()`), close the add panel if it's open. Add to the existing `showView` function:

```javascript
// At the start of showView(), before the toggle calls:
const addPanel = document.getElementById("add-panel");
const addBtn = document.getElementById("add-toggle-btn");
if (addPanel && !addPanel.classList.contains("hidden")) {
  addPanel.classList.add("hidden");
  addBtn.classList.remove("active");
}
```

This prevents stale state — returning to the quests view always starts with the panel closed.

---

## What doesn't change

1. **Add form internals** — all inputs, selects, dropdowns, tags button, and submit logic are untouched
2. **Filter bar internals** — all filter controls and `applyFilters()` wiring are untouched
3. **Quest list rendering** — no changes
4. **Saga add form** — stays as-is (sagas view has its own form with different usage patterns)
5. **Backend** — no Rust changes, no new commands

---

## Implementation steps

### Slice 1: Collapsible add form ✅

Reorder DOM (filter bar above add form), wrap add form in `#add-panel` with `hidden` class, add `+ New` button to filter bar, add `toggleAddForm()`, add panel CSS. The form opens/closes via click. Also: sized add form fields to match filter bar (10px), action buttons (+ New, Add) styled with white text on darker background.

### Slice 2: Keyboard shortcuts ✅

Add `N` key listener (opens panel + focuses title input), `Escape` listener within add-panel (closes panel, with guard for open TOD/DOW dropdowns), view-switch cleanup.

---

## Summary

Two slices, both frontend-only. Slice 1 is the structural change (reorder DOM, add wrapper + toggle). Slice 2 adds keyboard shortcuts for the power-user batch-add workflow. Each is independently testable and committable. No backend changes.
