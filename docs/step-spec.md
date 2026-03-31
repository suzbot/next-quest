# Step Spec: Phase 5A-3 — Category tags ✅

## Goal

Users can create custom tags (e.g., "Computer", "Outside"), apply them to quests and saga steps, and find tagged quests via the fuzzy search. Full vertical slice: backend schema + CRUD, frontend tag selector in add and edit mode, search integration.

---

## Substep 1: Backend — schema, CRUD, Quest struct

**Migration** (`db.rs` → `migrate()`):

```sql
CREATE TABLE IF NOT EXISTS tag (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS quest_tag (
    quest_id  TEXT NOT NULL REFERENCES quest(id),
    tag_id    TEXT NOT NULL REFERENCES tag(id),
    PRIMARY KEY (quest_id, tag_id)
);
```

Detection: check if tag table exists.

**Schema** — add to `create_tables` for test DB.

**Tag struct:**

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
}
```

**CRUD functions:**

- `get_tags(conn) -> Vec<Tag>` — all tags ordered by sort_order
- `add_tag(conn, name: String) -> Tag` — create with auto sort_order. Returns error if name already exists.
- `delete_tag(conn, id: String)` — deletes tag and all quest_tag links
- `set_quest_tags(conn, quest_id: String, tag_ids: Vec<String>)` — replace all tags for a quest (DELETE + INSERT, same pattern as `set_quest_links`)

**Batch loading:**

- `load_all_quest_tags(conn) -> HashMap<String, Vec<String>>` — same pattern as `load_all_quest_skills`. Maps quest_id → vec of tag_ids.

**Quest struct:** Add `tag_ids: Vec<String>`. Populate in `get_quests` via batch load (same as skill_ids, attribute_ids). Also populate in `get_saga_steps` and `query_single_quest`.

**`add_quest` and `add_saga_step` return values:** Set `tag_ids: Vec::new()` (tags applied via `set_quest_tags` after creation, same as links).

**Cleanup on deletion:**

- `delete_quest`: add `DELETE FROM quest_tag WHERE quest_id = ?1`
- `delete_saga` (which deletes saga steps): add `DELETE FROM quest_tag WHERE quest_id IN (SELECT id FROM quest WHERE saga_id = ?1)`

**Commands:** `get_tags`, `add_tag`, `delete_tag`, `set_quest_tags` wrappers. Register in main.rs.

**Tests:**

1. `add_and_get_tags` — Create two tags, verify get_tags returns both in order.
2. `add_duplicate_tag_errors` — Create a tag, try to create same name again, verify error.
3. `set_quest_tags_and_load` — Create tags, apply to a quest, verify tag_ids on the quest.
4. `delete_tag_removes_links` — Apply tag to quest, delete the tag, verify quest's tag_ids no longer includes it.
5. `delete_quest_removes_tag_links` — Apply tag to quest, delete quest, verify quest_tag rows gone.

**Testing checkpoint:** `cargo test` passes.

---

## Substep 2: Frontend — tag loading, add/edit tag selector, search integration

**Tag data loading:**

In `loadAll`, add `invoke("get_tags")` to parallel fetches. Store in `cachedTags`. Build `tagNameMap` (id → name), same pattern as skillNameMap/attrNameMap.

**Search integration:**

Update `buildSearchText` to include tag names:

```javascript
(q.tag_ids || []).forEach(id => { if (tagNameMap[id]) parts.push(tagNameMap[id]); });
```

Tags immediately work in fuzzy search.

**Category selector in the Tags row (quest add form, quest edit mode, saga step add/edit):**

The existing "Tags" button toggles a row that shows attribute and skill selectors. Add a third section for categories:

```
[Tags button]
  Attributes: Health ✕  Pluck ✕  | + attribute dropdown
  Skills: Cooking ✕  Cleaning ✕  | + skill dropdown
  Categories: Computer ✕  Outside ✕  | [add category...]
```

- Shows current categories as chips with ✕ to remove
- Text input for adding: type a name, press Enter
  - If name matches an existing category (case-insensitive) → apply it
  - If new → call `add_tag` to create, then apply
- On quest save: call `set_quest_tags` with current category IDs (same as `set_quest_links` call)
- "Category" is a type of tag — the entity is `tag` in the database, displayed as "Categories" in the UI

**Quest detail row display:**

Append tag names to the existing links text in the expanded detail row.

**Testing checkpoint:** Build app. Add a quest with tag "Computer" from the add form. Edit another quest, add tag "Outside". Search "computer" — first quest appears. Search "outside" — second quest appears. Remove a tag from a quest — no longer matches that search.

---

## NOT in this step

- Saga reordering (5A-4)
- Tag management UI in Settings (future — tags created inline, orphaned tags harmless)

## Done When

Tags can be created inline, applied to quests/saga steps from add and edit forms, and found via fuzzy search. Tag data loads with quests. Cleanup on quest/tag deletion. `cargo test` passes.
