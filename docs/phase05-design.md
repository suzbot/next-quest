# Phase 0.5: "Table Stakes" — Tech Design

## Overview

Adds character progression to the existing quest list app. New entities
(character, attributes, skills) with their own XP and levels. Quests gain
difficulty and links to skills/attributes. XP is calculated and distributed
on completion. A new character view page displays all progression data.

## Data Layer

### New Tables

```sql
CREATE TABLE character (
    id    TEXT PRIMARY KEY,
    name  TEXT NOT NULL DEFAULT 'Adventurer',
    xp    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE attribute (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL,
    xp          INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE skill (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    attribute_id  TEXT NOT NULL REFERENCES attribute(id),
    sort_order    INTEGER NOT NULL,
    xp            INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE quest_skill (
    quest_id  TEXT NOT NULL REFERENCES quest(id),
    skill_id  TEXT NOT NULL REFERENCES skill(id),
    PRIMARY KEY (quest_id, skill_id)
);

CREATE TABLE quest_attribute (
    quest_id      TEXT NOT NULL REFERENCES quest(id),
    attribute_id  TEXT NOT NULL REFERENCES attribute(id),
    PRIMARY KEY (quest_id, attribute_id)
);
```

### Quest Table Changes

```sql
ALTER TABLE quest ADD COLUMN difficulty TEXT NOT NULL DEFAULT 'easy';
```

### Completion Table Changes

```sql
ALTER TABLE quest_completion ADD COLUMN xp_earned INTEGER NOT NULL DEFAULT 0;
```

### Seed Data

On first launch (when `character` table is empty), seed:
- One character row (name: "Adventurer", xp: 0)
- 5 attributes (Health, Pluck, Knowledge, Connection, Responsibility)
- 12 skills with attribute mappings (see mechanics.md for full list)

### Migrations

- Detect missing tables/columns and create them
- Detect empty character table and seed defaults
- Add `difficulty` column to `quest` (default: 'easy')
- Add `xp_earned` column to `quest_completion` (default: 0)

### Derived Values (computed, not stored)

- **Level** (character, attribute, or skill): derived from XP using the
  Fibonacci curve. Each entity type has its own scale — see mechanics.md.
- **XP to next level**: computed from current XP and the curve.
- **Linked skills/attributes for a quest**: queried from join tables.

## Backend (Rust)

### New Tauri Commands

| Command | Input | Returns | What it does |
|---|---|---|---|
| `get_character` | — | Character with level info | Returns character name, xp, level, xp_to_next |
| `update_character` | name | Updated character | Updates character name |
| `get_attributes` | — | List of attributes with levels | All attributes with xp, level, xp_to_next |
| `get_skills` | — | List of skills with levels | All skills with xp, level, attribute_id, xp_to_next |
| `get_quest_links` | quest_id | Skill IDs + attribute IDs | Returns linked skill and attribute IDs for a quest |
| `set_quest_links` | quest_id, skill_ids, attribute_ids | — | Replaces all links for a quest |

### Modified Commands

| Command | Change |
|---|---|
| `add_quest` | New param: `difficulty`. Returns quest with difficulty. |
| `update_quest` | New param: `difficulty?`. |
| `complete_quest` | Calculates XP, distributes to character + linked skills/attributes, detects skill level-ups and applies attribute bumps, returns completion with xp_earned. |
| `get_quests` | Returns difficulty and linked skill/attribute IDs per quest. |

### Return Shapes

Character:
```json
{
    "name": "Adventurer",
    "xp": 1250,
    "level": 4,
    "xp_for_current_level": 800,
    "xp_into_current_level": 450
}
```

Attribute / Skill:
```json
{
    "id": "uuid",
    "name": "Health",
    "xp": 340,
    "level": 5,
    "xp_for_current_level": 260,
    "xp_into_current_level": 80
}
```

Skill also includes `attribute_id`.

Quest (updated):
```json
{
    "id": "uuid",
    "title": "Take a shower",
    "quest_type": "recurring",
    "cycle_days": 1,
    "difficulty": "easy",
    "sort_order": 10,
    "active": true,
    "created_at": "...",
    "last_completed": "...",
    "is_due": false,
    "skill_ids": ["uuid", "uuid"],
    "attribute_ids": ["uuid"]
}
```

Completion (updated):
```json
{
    "id": "uuid",
    "quest_id": "uuid-or-null",
    "quest_title": "Take a shower",
    "completed_at": "...",
    "xp_earned": 20
}
```

### XP Engine

Lives in `db.rs` as pure functions:

- `calculate_xp(difficulty, quest_type, cycle_days) -> i32` — applies formula
- `level_from_xp(xp, scale) -> (level, xp_for_current_level, xp_into_current_level)` — Fibonacci curve lookup with configurable scale (character/attribute/skill)
- `award_xp(conn, quest_id, xp)` — distributes XP to character + all linked skills/attributes, detects skill level-ups and awards 70 XP attribute bumps

XP is permanent — `delete_completion` does NOT subtract XP.

## Frontend

### Navigation

Two views, toggled by buttons at the top of the page:
- **[Quests]** — current quest list (default)
- **[Character]** — character progression view

No router or URL changes. JavaScript show/hide on a container div.

### Quest List Changes

**Add form** gains:
- Difficulty dropdown (Trivial/Easy/Moderate/Challenging/Epic, default: Easy)
- Skill multi-select (checkboxes or multi-select of the 12 skills)
- Attribute multi-select (checkboxes or multi-select of the 5 attributes)

**Edit mode** gains same fields.

**Quest rows** show difficulty label in the meta section.

**On completion**: briefly display "+{xp} XP" next to the quest (text only,
fades or disappears on next render).

### Character View

Text-based display:

```
CHARACTER NAME          Level 4 (450/800 XP)

ATTRIBUTES
  Health         Lv 3   (45/100 XP)
  Pluck          Lv 2   (20/60 XP)
  Knowledge      Lv 1   (0/60 XP)
  Connection     Lv 2   (30/60 XP)
  Responsibility Lv 1   (15/60 XP)

SKILLS                              ATTRIBUTE
  Nature         Lv 1   (10/30 XP)  Connection
  Bureaucracy    Lv 2   (5/50 XP)   Responsibility
  ...
```

Character name is clickable to edit (or has a small edit button).

## Implementation Steps

Each step is a vertical slice — buildable, testable, and committable on its own.

### Step 1: Schema, Seeds, and Difficulty

- New tables: character, attribute, skill, quest_skill, quest_attribute
- Seed defaults on first launch
- Add difficulty column to quest (migration)
- Add xp_earned column to quest_completion (migration)
- `get_character`, `get_attributes`, `get_skills` commands
- `update_character` command
- Difficulty dropdown on quest add/edit forms
- Tests for schema, seeds, level curve calculation

### Step 2: Quest Links

- `get_quest_links` and `set_quest_links` commands
- Skill/attribute selection on quest add/edit forms
- Quest rows display linked skills/attributes
- `get_quests` returns link IDs
- Tests for link CRUD

### Step 3: XP Engine

- `calculate_xp` function
- `award_xp` on completion (character + linked skills/attributes)
- Skill level-up detection → 70 XP attribute bump
- `xp_earned` stored on completion record
- Completion rows show XP earned
- "+XP" feedback on quest completion
- Tests for XP calculation, distribution, level-ups, attribute bumps

### Step 4: Character View

- Navigation buttons (Quests / Character)
- Character view page with all progression data
- Character name editing
- Tests for level_from_xp at all three scales
