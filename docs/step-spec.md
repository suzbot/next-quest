# Completion History Snapshots

**Goal:** Capture quest difficulty and linked skills/attributes/tags at completion time so completion history is self-contained for external analysis.

No backfill — existing completions get NULL for the new fields.

---

## Schema Change

Four nullable TEXT columns added to `quest_completion` via ALTER TABLE:

| Column | Type | Content | Example |
|---|---|---|---|
| `difficulty` | TEXT | Difficulty enum string | `"easy"` |
| `skills` | TEXT | JSON array of skill names | `["Cooking", "Healing"]` |
| `attributes` | TEXT | JSON array of attribute names | `["Health"]` |
| `tags` | TEXT | JSON array of tag names | `["Computer", "Outside"]` |

All nullable. NULL means either a bonus completion (no quest) or a pre-migration record.

Format matches the CLI's `QuestOutput` pattern: difficulty as a plain string, skills/attributes/tags as flat name arrays. No IDs — names are snapshotted so the record is self-contained even if entities are renamed or deleted later.

---

## Migration

In `nq-core/src/db.rs`, in the migration section (after existing ALTER TABLE blocks).

**Detection:** Check for `difficulty` column on `quest_completion` using the existing `prepare("SELECT difficulty FROM quest_completion LIMIT 0").is_ok()` pattern.

**If missing, run:**

```sql
ALTER TABLE quest_completion ADD COLUMN difficulty TEXT;
ALTER TABLE quest_completion ADD COLUMN skills TEXT;
ALTER TABLE quest_completion ADD COLUMN attributes TEXT;
ALTER TABLE quest_completion ADD COLUMN tags TEXT;
```

No DEFAULT needed — nullable columns default to NULL.

---

## Completion Creation Paths

Three INSERT sites in `db.rs`:

### 1. `complete_quest` (~line 2669) — gets snapshot data

This is the only path that changes. The function already reads the quest row for XP calculation and has `quest_id` in scope.

**New lookups** (three queries, after the existing quest row read, before the INSERT):

```rust
// Look up linked skill names
let skill_names: Vec<String> = conn.prepare(
    "SELECT s.name FROM quest_skill qs JOIN skill s ON s.id = qs.skill_id WHERE qs.quest_id = ?1"
)?.query_map(params![quest_id], |row| row.get(0))?
.collect::<Result<Vec<_>, _>>()?;

// Look up linked attribute names
let attr_names: Vec<String> = conn.prepare(
    "SELECT a.name FROM quest_attribute qa JOIN attribute a ON a.id = qa.attribute_id WHERE qa.quest_id = ?1"
)?.query_map(params![quest_id], |row| row.get(0))?
.collect::<Result<Vec<_>, _>>()?;

// Look up linked tag names
let tag_names: Vec<String> = conn.prepare(
    "SELECT t.name FROM quest_tag qt JOIN tag t ON t.id = qt.tag_id WHERE qt.quest_id = ?1"
)?.query_map(params![quest_id], |row| row.get(0))?
.collect::<Result<Vec<_>, _>>()?;
```

**Serialize to JSON strings:**

```rust
let skills_json = serde_json::to_string(&skill_names).ok();
let attrs_json = serde_json::to_string(&attr_names).ok();
let tags_json = serde_json::to_string(&tag_names).ok();
```

**Updated INSERT:**

```rust
tx.execute(
    "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at, xp_earned, difficulty, skills, attributes, tags)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    rusqlite::params![
        completion_id, quest_id, quest_title, completed_at, xp_earned,
        difficulty.as_str(), skills_json, attrs_json, tags_json
    ],
)
```

### 2. Saga bonus (~line 1437) — no change

INSERT explicitly lists columns and passes NULL for `quest_id`. The four new columns are nullable with no NOT NULL constraint, so SQLite fills them as NULL automatically. No code change needed.

### 3. Campaign bonus (~line 2021) — no change

Same situation as saga bonus.

---

## Struct Update

`Completion` struct (`db.rs` ~line 277):

```rust
#[derive(Serialize, Debug)]
pub struct Completion {
    pub id: String,
    pub quest_id: Option<String>,
    pub quest_title: String,
    pub completed_at: String,
    pub xp_earned: i64,
    pub level_ups: Vec<LevelUp>,
    pub xp_awards: Vec<XpAward>,
    pub difficulty: Option<String>,
    pub skills: Option<Vec<String>>,
    pub attributes: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}
```

`skills`, `attributes`, and `tags` are `Option<Vec<String>>` — serde serializes these as JSON arrays (matching the CLI pattern) or null. The raw column is a JSON string, deserialized on read.

---

## Read Path

`get_completions` (~line 2392):

```rust
"SELECT id, quest_id, quest_title, completed_at, xp_earned, difficulty, skills, attributes, tags
 FROM quest_completion
 ORDER BY completed_at DESC"
```

In the row mapper:

```rust
Ok(Completion {
    id: row.get(0)?,
    quest_id: row.get(1)?,
    quest_title: row.get(2)?,
    completed_at: row.get(3)?,
    xp_earned: row.get(4)?,
    level_ups: Vec::new(),
    xp_awards: Vec::new(),
    difficulty: row.get(5)?,
    skills: row.get::<_, Option<String>>(6)?.and_then(|s| serde_json::from_str(&s).ok()),
    attributes: row.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()),
    tags: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
})
```

---

## CLI: `list-history` Command

The CLI has no way to query completion history today. Add `list-history` so external analysis tools can access completions through the same interface they use for quests, skills, etc.

### Command definition

In `src-cli/src/main.rs`, add to the `Commands` enum:

```rust
/// List completion history as JSON
ListHistory,
```

### Output struct

```rust
#[derive(Serialize)]
struct CompletionOutput {
    id: String,
    quest_id: Option<String>,
    quest_title: String,
    completed_at: String,
    xp_earned: i64,
    difficulty: Option<String>,
    skills: Option<Vec<String>>,
    attributes: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    level_ups: Vec<LevelUpOutput>,
    xp_awards: Vec<XpAwardOutput>,
}

#[derive(Serialize)]
struct LevelUpOutput {
    name: String,
    new_level: i32,
}

#[derive(Serialize)]
struct XpAwardOutput {
    name: String,
    xp: i64,
    award_type: String,
}
```

`level_ups` and `xp_awards` are currently only populated at completion time for the GUI — they'll be empty arrays in history results today. Included for schema completeness so external tools pick them up if we ever persist them.

### Handler

```rust
fn list_history(conn: &nq_core::rusqlite::Connection) -> Result<String, String> {
    let completions = db::get_completions(conn)?;

    let output: Vec<CompletionOutput> = completions
        .iter()
        .map(|c| CompletionOutput {
            id: c.id.clone(),
            quest_id: c.quest_id.clone(),
            quest_title: c.quest_title.clone(),
            completed_at: c.completed_at.clone(),
            xp_earned: c.xp_earned,
            difficulty: c.difficulty.clone(),
            skills: c.skills.clone(),
            attributes: c.attributes.clone(),
            tags: c.tags.clone(),
            level_ups: c.level_ups.iter().map(|l| LevelUpOutput {
                name: l.name.clone(),
                new_level: l.new_level,
            }).collect(),
            xp_awards: c.xp_awards.iter().map(|a| XpAwardOutput {
                name: a.name.clone(),
                xp: a.xp,
                award_type: a.award_type.clone(),
            }).collect(),
        })
        .collect();

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}
```

Wire into the `match` in `run()`:

```rust
Commands::ListHistory => list_history(&conn),
```

---

## What Does NOT Change

- **Frontend** — no display changes. The new fields are available via `get_completions` but the history list in `ui/index.html` doesn't need to render them. This is for external analysis.
- **Tauri commands** — `get_completions` and `complete_quest` already return `db::Completion`. The struct change flows through automatically.

---

## Tests

In `nq-core/src/db.rs` tests:

### 1. `complete_quest_snapshots_difficulty`

Create a quest with `Difficulty::Moderate`, complete it, read completions, assert `difficulty == Some("moderate")`.

### 2. `complete_quest_snapshots_skills_and_attributes`

Create a quest, link it to skills (Cooking, Healing) and an attribute (Health), complete it, read completions, assert `skills == Some(vec!["Cooking", "Healing"])` and `attributes == Some(vec!["Health"])`.

### 3. `complete_quest_snapshots_tags`

Create a quest, link it to tags (Computer, Outside), complete it, read completions, assert `tags == Some(vec!["Computer", "Outside"])`.

### 4. `complete_quest_no_links_snapshots_empty_arrays`

Create a quest with no linked skills/attributes/tags, complete it, assert `skills == Some(vec![])`, `attributes == Some(vec![])`, `tags == Some(vec![])`.

### 5. `bonus_completion_has_null_snapshots`

Complete a one-off saga (triggering a bonus completion), read completions, find the bonus entry (`quest_id.is_none()`), assert `difficulty.is_none()`, `skills.is_none()`, `attributes.is_none()`, `tags.is_none()`.

### 6. `old_completions_have_null_snapshots`

Manually INSERT a completion without the new columns (simulating pre-migration data), read it back via `get_completions`, assert all four new fields are `None`.

---

## Documentation Updates

**`DATA_MODEL.md`** — Add the four new fields to the Completion entity table. Add a note that these are snapshots captured at completion time and may be NULL for bonus completions or pre-migration records.

**`docs/cli-guide.md`** — Add `list-history` command with usage, example output, and field descriptions.

---

## Verification

1. `cargo test` — all existing tests pass, new tests pass
2. Build the app: `cargo tauri build --debug`
3. Complete a quest that has linked skills, attributes, and tags
4. Check the completion history (visible in GUI) — no visual change expected
5. Query the SQLite database directly to confirm the new columns are populated with correct snapshot values
