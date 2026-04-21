# Tech Debt

Items that work but could be cleaner. Not blocking anything — just things to address when convenient. Each item has trigger conditions that describe when it becomes worth fixing.

## 1. renderCheckDropdown: DOM manipulation → inline HTML

**Where:** `renderCheckDropdown` in `ui/index.html`

**Problem:** The TOD/DOW multi-select dropdown is built via DOM manipulation (`createElement`, `appendChild`) after the HTML is on the page. Every other form element renders as inline HTML in template strings. This inconsistency means TOD/DOW requires a separate post-render step to populate empty containers, which is fragile — it already broke once when the data source for the post-render lookup changed.

**Fix:** Convert `renderCheckDropdown` to return an HTML string (like `renderQuestRow`, `renderEditMode`, etc.) with inline `onclick`/`onchange` handlers. Then TOD/DOW can be embedded directly in template strings alongside all other form elements. The post-render population blocks can be removed entirely.

**Scope:** 10 call sites across 5 contexts (quest add, quest add reset, quest edit, saga step add, saga step edit). Each context has a TOD + DOW pair.

**Trigger conditions — implement when ANY of:**
- (a) Next bug traced to the DOM-vs-inline-HTML inconsistency (post-render population fails or races with another render)
- (b) A 6th form context is added that needs TOD/DOW dropdowns
- (c) A new form-field type needs the same multi-select pattern and the current approach can't absorb it cleanly

## 2. Quest add form rendered twice

**Where:** Lines ~1014 and ~2050 in `ui/index.html`

**Problem:** The quest add form's TOD/DOW dropdowns are initialized in two places: once during initial page load, and again after adding a quest (to reset to defaults). These are identical calls. If the add form were a function that returned HTML (or a single `resetAddForm` function), this duplication would collapse.

**Fix:** Extract a `resetAddForm()` function that sets all add-form fields to defaults, including TOD/DOW. Call it on load and after adding a quest. Would become simpler if combined with item 1 above.

**Trigger conditions — implement when ANY of:**
- (a) The add form reset logic diverges between the two call sites (one gets updated, the other doesn't)
- (b) A new default field is added to the add form and must be initialized in both places
- (c) Item 1 is being addressed (combine them — the `resetAddForm` extraction becomes trivial once `renderCheckDropdown` returns HTML)

## 3. Saga step add/edit form duplication

**Where:** Saga step add form (~line 3078) and saga step edit form (~line 3068) in `ui/index.html`

**Problem:** The saga step add and edit forms duplicate the same TOD/DOW/difficulty/importance rendering logic that exists in the quest add and edit forms. The field sets are nearly identical — both have title, difficulty, importance, TOD, DOW. The only difference is saga steps don't have quest_type/cycle_days.

**Fix:** Extract shared form-field rendering helpers that both quest and saga step forms use. This would reduce the surface area for bugs like the one where quest edit TOD/DOW broke but saga step edit didn't (different code paths for the same UI).

**Trigger conditions — implement when ANY of:**
- (a) A bug in one form type (quest or saga step) that doesn't reproduce in the other, caused by the parallel code paths
- (b) A new shared field is added to both quest and saga step forms (increasing the duplication surface)
- (c) Items 1 and 2 are being addressed (natural time to unify since the form rendering is already being reworked)
