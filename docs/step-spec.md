# Step Spec: Phase 5A-2 — Fuzzy search + difficulty + importance filters ✅

## Goal

Replace the attribute/skill filter dropdowns with a fuzzy text search field. Add difficulty and importance exact-match dropdowns. Search covers quest title, linked skill/attribute names, difficulty label, and importance marks.

---

## Substep 1: HTML — replace filter bar

Remove `filter-attr` and `filter-skill` selects. Add search input and two new dropdowns:

```html
<div id="filter-bar">
  <input type="text" id="filter-search" placeholder="Search quests..." oninput="applyFilters()">
  <select id="filter-difficulty" onchange="applyFilters()">
    <option value="">All Difficulties</option>
    <option value="trivial">Trivial</option>
    <option value="easy">Easy</option>
    <option value="moderate">Fair</option>
    <option value="challenging">Hard</option>
    <option value="epic">Epic</option>
  </select>
  <select id="filter-importance" onchange="applyFilters()">
    <option value="">All Importance</option>
    <option value="0">—</option>
    <option value="1">!</option>
    <option value="2">!!</option>
    <option value="3">!!!</option>
    <option value="4">!!!!</option>
    <option value="5">!!!!!</option>
  </select>
  <select id="filter-tod" onchange="applyFilters()">...</select>
  <select id="filter-dow" onchange="applyFilters()">...</select>
  <label><input type="checkbox" id="filter-due" onchange="applyFilters()"> Due</label>
  <button type="button" onclick="clearFilters()">Clear</button>
</div>
```

**CSS:** Search input gets `flex: 1` to fill available space. Font matches existing filter elements (Silkscreen 10px).

---

## Substep 2: JS — search text builder + filter logic

**Build searchable text per quest:**

```javascript
function buildSearchText(q) {
  const parts = [q.title];
  q.skill_ids.forEach(id => { if (skillNameMap[id]) parts.push(skillNameMap[id]); });
  q.attribute_ids.forEach(id => { if (attrNameMap[id]) parts.push(attrNameMap[id]); });
  parts.push(difficultyLabel(q.difficulty));
  if (q.importance > 0) parts.push("!".repeat(q.importance));
  return parts.join(" ").toLowerCase();
}
```

**Update `passesFilters`:**

Replace attribute/skill filter checks with:
```javascript
const searchVal = filterSearch.value.trim().toLowerCase();
if (searchVal && !buildSearchText(q).includes(searchVal)) return false;
if (filterDifficulty.value && q.difficulty !== filterDifficulty.value) return false;
if (filterImportance.value !== "" && q.importance !== parseInt(filterImportance.value)) return false;
```

Keep existing TOD, DOW, Due checks unchanged.

**Update `clearFilters`:**

Remove `filterAttr.value = ""` and `filterSkill.value = ""`. Add:
```javascript
filterSearch.value = "";
filterDifficulty.value = "";
filterImportance.value = "";
```

**Update filter state references:**

Remove `filterAttr` and `filterSkill` const declarations. Add `filterSearch`, `filterDifficulty`, `filterImportance`.

**Remove `populateFilterDropdowns` attr/skill logic** — no longer needed. The function can be simplified or removed if its only purpose was populating those dropdowns. Check if it does anything else (it also sets up `attrIndexById` etc. — keep that, just remove the dropdown population).

---

## Testing checkpoint

Build app. Quest list filter bar shows: search field, difficulty dropdown, importance dropdown, TOD, DOW, Due, Clear.

- Type "vac" → only Vacuuming visible
- Type "cook" → quests linked to Cooking skill visible
- Type "health" → quests linked to Health attribute visible
- Type "!!!" → quests with importance 3, 4, 5 visible
- Type "trivial" → trivial quests visible (via search)
- Select Trivial in difficulty dropdown → only trivial quests
- Select !!! in importance dropdown → only importance 3 quests (exact match)
- Combine: search "cat" + difficulty Trivial → only trivial cat-related quests
- Clear → all filters reset

---

## NOT in this step

- Category tags (step 2)

## Done When

Fuzzy search field works across quest title, skill names, attribute names, difficulty, importance. Difficulty and importance dropdowns filter by exact match. Attribute/skill dropdowns removed. All filters combine via AND. Clear resets all. No backend changes.
