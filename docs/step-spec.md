# Step Spec: Phase 0.5, Step 1 — "Schema, Seeds, and Difficulty"

## Goal

Lay the foundation for character progression. Create all new tables, seed
default character/attributes/skills on first launch, and add difficulty to
quests.

## Scope

**Schema:**
- Create `character` table (id, name, xp)
- Create `attribute` table (id, name, sort_order, xp)
- Create `skill` table (id, name, attribute_id, sort_order, xp)
- Create `quest_skill` join table (quest_id, skill_id)
- Create `quest_attribute` join table (quest_id, attribute_id)
- Add `difficulty` column to `quest` (TEXT NOT NULL DEFAULT 'easy')
- Add `xp_earned` column to `quest_completion` (INTEGER NOT NULL DEFAULT 0)

**Seed data (runs once, when character table is empty):**
- One character: "Adventurer", 0 XP
- 5 attributes: Health, Pluck, Knowledge, Connection, Responsibility
- 12 skills with attribute mappings (see mechanics.md)

**Backend commands:**
- `get_character` → returns character name, xp, level, xp_for_current_level,
  xp_into_current_level
- `update_character(name)` → updates character name, returns updated character
- `get_attributes` → returns all attributes with xp and level info
- `get_skills` → returns all skills with xp, level info, and attribute_id
- `level_from_xp(xp, scale)` → pure function, Fibonacci curve lookup.
  Three scales: character (seeds 300/500), attribute (60/100), skill (30/50).

**Frontend:**
- Add difficulty dropdown to quest add form (default: Easy)
- Add difficulty dropdown to quest edit form
- Show difficulty label on quest rows
- Migration for existing quests: all get difficulty 'easy'

## NOT in this step

- Quest-to-skill/attribute links (Step 2)
- XP calculation or distribution on completion (Step 3)
- Character view page (Step 4)
- Navigation between pages (Step 4)

## Done When

- App launches with character, attributes, and skills seeded in the database
- `get_character` returns level info derived from XP
- `get_attributes` and `get_skills` return all defaults with level info
- `update_character` changes the name
- `level_from_xp` correctly computes levels at all three scales
- Quest add/edit shows difficulty dropdown, default Easy
- Existing quests gain difficulty 'easy' via migration
- Quest rows display difficulty
- Tests cover: seed data, level curve at all three scales, difficulty migration,
  character CRUD

## Next Step Preview

Step 2: "Quest Links" — link quests to skills and attributes, with UI for
selecting links during add/edit.
