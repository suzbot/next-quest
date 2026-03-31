# Phase 5A-2/3/4: Quest List Filters — Requirements

**Status:** Draft

**Goal:** Make it easy to find quests in a growing list. Add a general-purpose search field, difficulty and importance filters, and user-defined category tags.

---

## 1. Fuzzy string search

### User story

> I have 70+ quests. Sometimes I want to find a specific quest to check off, especially if I got it done early and it's not due. A general search field is a more natural way to find things than dropdowns for specific attributes or skills — and it covers those cases too.

### Behavior

- Single text input replaces the attribute and skill dropdown filters. General search is a more likely use case, and covers the same filtering (searching "Cooking" or "Health" finds the same quests the old dropdowns did) while using the real estate better.
- Searches against: quest title, linked attribute names, linked skill names, category tags (see section 3), difficulty label, and importance marks
- Case-insensitive substring match ("vac" finds "Vacuuming", "cook" finds quests linked to Cooking, "!!!" finds quests with importance 3+, "trivial" finds trivial quests)
- Filters in real-time as the user types (no submit button)
- Empty search field = no text filter (show all)
- Combined with other filters (difficulty, importance, TOD, DOW, due) via AND logic

## 2. Difficulty and importance filters

### User story

> I want to see only trivial quests I could knock out right now. Or I want to see everything marked 3! or higher to review my priorities.

### Difficulty filter

Dropdown with options: All Difficulties, Trivial, Easy, Fair, Hard, Epic. Filters quest list to only show quests matching the selected difficulty. Default: All Difficulties.

### Importance filter

Dropdown with options: All Importance, — (0), ! (1), !! (2), !!! (3), !!!! (4), !!!!! (5). Exact match, same pattern as difficulty. Default: All Importance.

For ">=" style filtering (e.g., "everything importance 3 and above"), the fuzzy search covers this — searching "!!!" matches quests with 3, 4, or 5 exclamation marks since it's a substring match.

### Difficulty and importance follow the same pattern

Both are single-select exact-match dropdowns. Both combine with other filters via AND logic. Both are also searchable via the fuzzy search field (searching "trivial" or "!!!" works).

## 3. Category tags

### User story

> I'm about to sit at my computer. I want to see all quests I can do at the computer. I'm going outside — show me outdoor tasks. Categories are personal and fluid — I want to create my own.

### Entity

A lightweight tag system. Tags are user-created strings (e.g., "Computer", "Outside", "Phone", "Errands").

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Tag display name |
| sort_order | Integer | Display/creation order |

### Quest-tag relationship

Many-to-many, same pattern as quest-skill and quest-attribute links.

| Field | Type | Description |
|---|---|---|
| quest_id | UUID (text) | FK to Quest |
| tag_id | UUID (text) | FK to Tag |

### Tag management

- Tags are created inline when first applied (type a new tag name → it's created)
- Tags can be applied to quests in the quest edit mode (similar to skill/attribute tags)
- Tags can also be applied to saga steps
- Unused tags can be deleted from a tag management UI (Settings tab, or a dedicated section)

### Tag filtering

Tags are included in the fuzzy search only — no dedicated dropdown. Searching "computer" matches quests tagged "Computer". This is consistent with the "one search covers everything" philosophy that motivates replacing the old attribute/skill dropdowns.

---

## Filter bar layout (after all three items)

The filter bar replaces the current attribute/skill dropdowns with a search field and adds difficulty/importance dropdowns:

```
[🔍 Search quests...          ] [All Difficulties ▾] [All Importance ▾] [All Times ▾] [All Days ▾] [☑ Due] [Clear]
```

- Search field is wider, takes up available space
- Difficulty and importance dropdowns are new
- Attribute and skill dropdowns are removed
- TOD, DOW, Due, and Clear remain as-is
- Tag dropdown placement TBD based on open question above

---

## What doesn't change

- Quest add form (unchanged)
- Quest edit mode (gains tag selector, similar to existing skill/attribute tags)
- Saga tab (saga steps can have tags, applied in step edit mode)
- Quest giver / scoring (tags don't affect scoring)

---

