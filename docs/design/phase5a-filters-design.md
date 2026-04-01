# Phase 5A-2/3/4: Quest List Filters — Design

## Overview

Three changes to the quest list filter bar: replace attribute/skill dropdowns with a fuzzy search field, add difficulty and importance dropdowns, and add a category tag system. All frontend-first — tags require a new entity, but difficulty/importance filters and fuzzy search are purely frontend.

## 1. Filter bar HTML

### Current
```
[All Attributes ▾] [All Skills ▾] [All Times ▾] [All Days ▾] [☑ Due] [Clear]
```

### New
```
[🔍 Search quests...          ] [All Difficulties ▾] [All Importance ▾] [All Times ▾] [All Days ▾] [☑ Due] [Clear]
```

Remove `filter-attr` and `filter-skill` selects. Add `filter-search` text input, `filter-difficulty` select, `filter-importance` select.

## 2. Fuzzy search

### Search text construction

For each quest, build a searchable string at render time:

```javascript
function buildSearchText(q) {
  const parts = [q.title];
  q.skill_ids.forEach(id => { if (skillNameMap[id]) parts.push(skillNameMap[id]); });
  q.attribute_ids.forEach(id => { if (attrNameMap[id]) parts.push(attrNameMap[id]); });
  if (q.tag_names) parts.push(...q.tag_names);
  parts.push(difficultyLabel(q.difficulty));
  if (q.importance > 0) parts.push("!".repeat(q.importance));
  return parts.join(" ").toLowerCase();
}
```

### Filter logic

In `passesFilters`, add a search check:

```javascript
const searchVal = filterSearch.value.trim().toLowerCase();
if (searchVal && !buildSearchText(q).includes(searchVal)) return false;
```

Real-time filtering via `oninput="applyFilters()"` on the search field.

## 3. Difficulty and importance dropdowns

### Difficulty filter

```html
<select id="filter-difficulty" onchange="applyFilters()">
  <option value="">All Difficulties</option>
  <option value="trivial">Trivial</option>
  <option value="easy">Easy</option>
  <option value="moderate">Fair</option>
  <option value="challenging">Hard</option>
  <option value="epic">Epic</option>
</select>
```

Filter: `if (filterDifficulty.value && q.difficulty !== filterDifficulty.value) return false;`

### Importance filter

```html
<select id="filter-importance" onchange="applyFilters()">
  <option value="">All Importance</option>
  <option value="0">—</option>
  <option value="1">!</option>
  <option value="2">!!</option>
  <option value="3">!!!</option>
  <option value="4">!!!!</option>
  <option value="5">!!!!!</option>
</select>
```

Filter: `if (filterImportance.value !== "" && q.importance !== parseInt(filterImportance.value)) return false;`

## 4. Category tags

### Schema

**New table: `tag`**

```sql
CREATE TABLE IF NOT EXISTS tag (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL
);
```

**New table: `quest_tag`**

```sql
CREATE TABLE IF NOT EXISTS quest_tag (
    quest_id  TEXT NOT NULL REFERENCES quest(id),
    tag_id    TEXT NOT NULL REFERENCES tag(id),
    PRIMARY KEY (quest_id, tag_id)
);
```

### Backend functions

- `get_tags(conn) -> Vec<Tag>` — all tags ordered by sort_order
- `add_tag(conn, name) -> Tag` — create new tag (auto sort_order)
- `delete_tag(conn, id)` — delete tag and its quest_tag links
- `set_quest_tags(conn, quest_id, tag_ids: Vec<String>)` — replace all tags for a quest (same pattern as `set_quest_links`)
- `load_all_quest_tags(conn) -> HashMap<String, Vec<String>>` — batch load tag IDs per quest (same pattern as `load_all_quest_skills`)

### Quest struct changes

Add `tag_ids: Vec<String>` to `Quest` struct. Populated via batch load in `get_quests` (same as skill_ids and attribute_ids).

Add `tag_names: Vec<String>` — resolved from tag_ids using a tag name map. This avoids an extra lookup in the frontend search. Alternatively, the frontend can maintain a `tagNameMap` like `skillNameMap`.

**Decision: use frontend `tagNameMap`** — consistent with how skills and attributes work. The Quest struct gets `tag_ids`, the frontend resolves names.

### Commands

- `get_tags`, `add_tag`, `delete_tag`, `set_quest_tags` — command wrappers, registered in main.rs

### Frontend — tag management

**Tag name map:** Loaded alongside quests. `let tagNameMap = {}; cachedTags.forEach(t => tagNameMap[t.id] = t.name);`

**Quest edit mode:** Add a tag selector below the existing skill/attribute tags. Same UI pattern — clickable tag chips with ✕ to remove, plus an input to add new tags.

**Inline tag creation:** If the user types a tag name that doesn't exist, create it via `add_tag` then apply it.

**Search text:** Tag names included in `buildSearchText` via `tagNameMap`.

### Frontend — tag display on quest rows

Show tags after the quest title (or in the detail row), styled as small muted chips:

```
Take vitamins !!!!! [Morning] [Health]   MO  ↻1d  Trivial
```

Wait — that could get noisy. Better: tags only appear in the expanded detail row (same as skill/attribute links), and are searchable regardless of visibility.

### Cleanup on quest/saga deletion

`delete_quest` already cleans up `quest_skill` and `quest_attribute`. Add `DELETE FROM quest_tag WHERE quest_id = ?1`. Same for saga deletion (which deletes its steps' links).

## Implementation order

1. **Fuzzy search + difficulty + importance filters** — Frontend only. Replace attr/skill dropdowns with search field, add difficulty/importance dropdowns, update `passesFilters` and `clearFilters`. Search covers title, skill names, attribute names, difficulty labels, importance marks. No backend changes. Testing: search "vac" finds Vacuuming, search "cook" finds quests linked to Cooking, search "!!!" finds importance 3+, filter Trivial shows only trivials, filter !!! shows only importance 3. Clear resets all.

2. **Category tags (full vertical slice)** — Backend: migration (tag + quest_tag tables), CRUD functions, commands, Quest struct gets tag_ids, batch loading, cleanup on deletion. Frontend: tag name map, edit mode tag selector, inline tag creation, tags included in fuzzy search, tag display in expanded detail row. Testing: create a tag "Computer", apply it to quests in edit mode, search "computer" finds them, delete a quest with tags (no orphans).

### Summary

Two vertical slices. Step 1 delivers search and filter upgrade immediately. Step 2 adds tags as a complete feature that plugs into the existing search.
