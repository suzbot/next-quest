# Step Spec: Phase 2E — Editable Attributes and Skills

## Goal

Add, rename, and delete skills and attributes from the Character tab without code changes.

---

## Substep 1: Backend CRUD

All new db.rs functions, commands.rs wrappers, and tests. No UI changes.

**db.rs — new functions:**
- `add_attribute(conn, name) -> Attribute`
- `add_skill(conn, name, attribute_id: Option<String>) -> Skill`
- `rename_attribute(conn, id, name)`
- `rename_skill(conn, id, name)`
- `update_skill_attribute(conn, skill_id, attribute_id: Option<String>)`
- `delete_attribute(conn, id)` — deletes row + quest_attribute links + unsets skills mapped to it
- `delete_skill(conn, id)` — deletes row + quest_skill links

**commands.rs — new Tauri commands** wrapping each db function.

**Tests:**
- Add attribute, verify returned with 0 XP
- Add skill with and without attribute mapping
- Rename attribute, verify name changed
- Rename skill, verify name changed
- Update skill attribute mapping
- Delete attribute cleans up quest links and unsets mapped skills
- Delete skill cleans up quest links
- Delete nonexistent attribute/skill errors gracefully

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Color Palette

Replace hardcoded name-to-color maps with index-based cycling palette.

**Changes:**
- Define ordered palette arrays (5 fill colors, then 5 text colors, cycling)
- Attribute color determined by index in the attributes list, not by name
- Skill color determined by its mapped attribute's color, or gray if unmapped
- Remove `attrFillColors` and `attrTextColors` name-keyed objects
- Level-up notification colors also driven by palette

**Testing checkpoint:** Build app, verify Character tab looks the same as before (colors in same order for default attributes). Level-up notifications still show correct colors.

---

## Substep 3: Attribute UI

Add/rename/delete for attributes on the Character tab.

**Changes:**
- Click attribute name → inline rename input with Save and ✕
- Delete button on each attribute row with confirm pattern ("Sure?" → 2s timeout)
- "Add Attribute" button below attribute list → inline input for name
- Refresh character view after each operation

**Testing checkpoint:** Build app. Add a new attribute — appears with meter at 0 XP and a color from the palette. Rename it. Delete it with confirmation. Verify quest links to deleted attribute are gone.

---

## Substep 4: Skill UI

Add/rename/delete for skills, plus attribute mapping dropdown.

**Changes:**
- Click skill name → inline rename input with Save and ✕
- Attribute mapping shown as dropdown (current attribute selected, "None" for unmapped)
- Changing dropdown calls update_skill_attribute
- Delete button with confirm pattern
- "Add Skill" button below skill list → inline input for name + attribute dropdown
- Unmapped skills show gray meters

**Testing checkpoint:** Build app. Add a new skill mapped to an attribute — appears with correct color. Change its mapping — color updates. Set to None — turns gray. Delete it with confirmation. Verify quest links cleaned up.

---

## NOT in this phase

- User-selectable color picker for attributes
- Reset behavior changes
- Seed data guard rework

## Done When

All four substeps complete and tested.
