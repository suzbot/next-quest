# CLI Step 3: add-quest

**Goal:** Create quests from the command line with full skill/attribute/tag linking. First write command.

**Design:** [cli-design.md](cli-design.md)

---

## What We're Building

```
nq add-quest \
  --title "Scoop Kitty Litter (Front)" \
  --difficulty easy \
  --type recurring \
  --cycle-days 2 \
  --importance 2 \
  --time-of-day "evening" \
  --skills "Cleaning" \
  --tags "Inside"
```

Output:
```json
{
  "ok": true,
  "id": "uuid-here",
  "title": "Scoop Kitty Litter (Front)",
  "skills": ["Cleaning"],
  "attributes": [],
  "tags": ["Inside"]
}
```

---

## Changes

### 1. Strict enum parsing (nq-core)

Add `try_from_str()` methods to `Difficulty` and `QuestType` that return `Result` instead of silently defaulting. The existing `from_str()` methods stay unchanged (used by the GUI).

```rust
impl Difficulty {
    pub fn try_from_str(s: &str) -> Result<Self, String>
    // Err: "Invalid difficulty 'hard'. Valid values: trivial, easy, moderate, challenging, epic"
}

impl QuestType {
    pub fn try_from_str(s: &str) -> Result<Self, String>
    // Err: "Invalid quest type 'daily'. Valid values: recurring, one_off"
}
```

### 2. Bitmask name parsing (nq-core)

```rust
pub fn parse_time_of_day(input: &str) -> Result<i32, String>
pub fn parse_days_of_week(input: &str) -> Result<i32, String>
```

- Comma-separated, case-insensitive: `"Morning,Evening"` → 5
- Special values: `"anytime"` → 15, `"everyday"` → 127
- Error on unrecognized names: `"Invalid time-of-day 'noon'. Valid values: morning, afternoon, evening, night, anytime"`

### 3. Name-based resolution (nq-core)

```rust
pub fn resolve_skill_by_name(conn: &Connection, name: &str) -> Result<Skill, String>
pub fn resolve_attribute_by_name(conn: &Connection, name: &str) -> Result<Attribute, String>
pub fn find_or_create_tag(conn: &Connection, name: &str) -> Result<Tag, String>
```

- Skills and attributes: case-insensitive `LOWER(name) = LOWER(?)` query. Error if not found, listing all available names so the caller can self-correct.
- Tags: case-insensitive lookup. If no match, creates a new tag with the provided casing.

Error example: `"Skill 'Baking' not found. Available: Self Care, Movement, Medical, Cleaning, Cooking, Handiness, ..."`

### 4. Composite quest creation (nq-core)

```rust
pub fn add_quest_full(
    conn: &Connection,
    quest: NewQuest,
    tag_names: Vec<String>,
    skill_names: Vec<String>,
    attribute_names: Vec<String>,
) -> Result<Quest, String>
```

- Wraps everything in a single transaction (`conn.transaction()`)
- Resolves all names to IDs first (skill/attribute must exist, tags auto-created)
- Calls `add_quest()` to create the quest
- Calls `set_quest_tags()` and `set_quest_links()` to wire up the links
- If any resolution fails, the transaction rolls back — nothing is created
- Returns the quest with `skill_ids`, `attribute_ids`, `tag_ids` populated

### 5. add-quest CLI command

**Required flags:**
- `--title` — quest title (string)
- `--difficulty` — validated via `Difficulty::try_from_str()`
- `--type` — validated via `QuestType::try_from_str()`

**Optional flags:**
- `--importance` — 0–5 (default: 0). Validated in CLI code.
- `--cycle-days` — integer. Required if type is recurring (error if missing), ignored if one_off.
- `--time-of-day` — comma-separated names (default: not provided → uses NewQuest default of 15/anytime). Parsed via `parse_time_of_day()`.
- `--days-of-week` — comma-separated names (default: not provided → uses NewQuest default of 127/everyday). Parsed via `parse_days_of_week()`.
- `--tags` — comma-separated tag names.
- `--skills` — comma-separated skill names.
- `--attributes` — comma-separated attribute names.

**Flow:**
1. Parse CLI flags (clap)
2. Validate difficulty, quest type, importance
3. Parse bitmask names if provided
4. Build `NewQuest` struct
5. Open database (`db_path()` → `init_db()`)
6. Call `add_quest_full()` with the quest and name lists
7. On success: print JSON to stdout with id, title, and resolved link names
8. On error: print JSON to stderr, exit 1

**Success output:**
```json
{
  "ok": true,
  "id": "uuid-here",
  "title": "Scoop Kitty Litter (Front)",
  "skills": ["Cleaning"],
  "attributes": [],
  "tags": ["Inside"]
}
```

**Error output:**
```json
{"ok": false, "error": "Skill 'Baking' not found. Available: Self Care, Movement, Medical, Cleaning, Cooking, Handiness, ..."}
```

### 6. Tests (nq-core)

- `parse_time_of_day`: valid single, valid multi, anytime, invalid name, case insensitivity
- `parse_days_of_week`: valid single, valid multi, everyday, invalid name, case insensitivity
- `resolve_skill_by_name`: found, not found (error includes available names), case insensitive match
- `resolve_attribute_by_name`: found, not found, case insensitive
- `find_or_create_tag`: existing tag matched case-insensitively, new tag created with provided casing
- `add_quest_full`: quest created with links, bad skill name rolls back (no quest created), tag auto-creation within transaction
- `Difficulty::try_from_str`: all five valid values, invalid value
- `QuestType::try_from_str`: both valid values, invalid value

---

## Verification

1. **Tests pass:** `cd nq-core && cargo test`
2. **Create a quest with all options:**
   ```
   ./target/debug/nq add-quest \
     --title "Test Quest" \
     --difficulty easy \
     --type recurring \
     --cycle-days 3 \
     --importance 2 \
     --time-of-day "morning,evening" \
     --days-of-week "mon,wed,fri" \
     --skills "Cleaning" \
     --tags "Inside"
   ```
   Open app — quest appears with correct metadata, Cleaning linked, Inside tag linked.
3. **Create with new tag:** use `--tags "NewTag"` — tag auto-created, visible in app.
4. **Error on bad skill:** `--skills "Baking"` — error message lists available skills, no quest created.
5. **Error on bad difficulty:** `--difficulty hard` — clear error with valid values.
6. **Recurring without cycle-days:** error.
7. **GUI still works:** rebuild and launch the app.
