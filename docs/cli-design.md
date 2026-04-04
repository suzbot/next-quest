# Next Quest CLI — Design

**Status:** Draft
**Requirements:** [cli-requirements.md](cli-requirements.md)

---

## Architecture

### Cargo Workspace

The project becomes a Cargo workspace with three members:

```
nq/
  Cargo.toml              # workspace root
  nq-core/                # shared library crate
    Cargo.toml
    src/
      lib.rs              # re-exports db module
      db.rs               # moved from src-tauri/src/db.rs (all data logic, tests)
  src-tauri/              # Tauri GUI binary
    Cargo.toml            # depends on tauri + nq-core
    src/
      main.rs
      commands.rs          # Tauri command wrappers (imports nq_core::db)
      tray.rs
  src-cli/                # CLI binary
    Cargo.toml            # depends on clap + nq-core
    src/
      main.rs             # argument parsing, command dispatch, JSON output
  ui/                     # unchanged
```

**Why a workspace:** The CLI must share `db.rs` without pulling in Tauri as a dependency. A workspace with a shared library crate (`nq-core`) gives each binary exactly the dependencies it needs. The alternative — feature flags on a single crate — is messier and harder to reason about.

**What moves to nq-core:**
- `db.rs` — all data logic, types, migrations, seeds, tests
- Database path resolution (new)
- Bitmask name conversion (new)

**What stays in src-tauri:**
- `commands.rs` — Tauri command wrappers, timer state, tray state, skip state, settings
- `tray.rs` — system tray
- `main.rs` — app setup, Encounters polling thread

### Dependencies

| Crate | Member | What it does | Why |
|---|---|---|---|
| nq-core | nq-core | Shared library | Houses db.rs and shared logic |
| rusqlite 0.31 | nq-core | SQLite access (bundled) | Existing — moves with db.rs |
| serde 1 | nq-core | Serialization | Existing — moves with db.rs |
| serde_json 1 | nq-core | JSON parsing | Existing — moves with db.rs |
| uuid 1 | nq-core | Unique IDs | Existing — moves with db.rs |
| dirs 6 | nq-core | Platform data directory | Existing — moves with db.rs |
| libc 0.2 | nq-core | Timezone conversion | Existing — moves with db.rs |
| tauri 2 | src-tauri | App framework | Existing — stays |
| clap 4 | src-cli | CLI argument parsing | New — subcommands, value validation, derive macros |

**clap** is the standard Rust CLI framework. The `derive` feature lets us define commands as structs with attribute annotations — clap generates the parser, help text, and validation automatically.

---

## Database Access

### Path Resolution

New function in nq-core: `db_path() -> PathBuf`

Returns the path to the Next Quest database, using the same logic as the Tauri app:
```
{dirs::data_dir()}/com.next-quest/next-quest.db
```

Both the GUI and CLI call this to find the database. If the directory doesn't exist (CLI run before GUI has ever launched), the function creates it.

### WAL Mode

Added to `init_db()`:
```rust
conn.execute_batch("PRAGMA journal_mode=WAL;")
```

WAL (Write-Ahead Logging) allows the GUI to read while the CLI writes, and queues concurrent writes briefly instead of failing. Without it, simultaneous access from both binaries would produce "database is locked" errors.

### Busy Timeout

Added to `init_db()`:
```rust
conn.busy_timeout(std::time::Duration::from_secs(5))
```

If the database is locked by the other process, wait up to 5 seconds before returning SQLITE_BUSY. This covers the common case where the GUI is in the middle of a write when the CLI starts one.

---

## New Shared Functions (nq-core)

### Name-Based Lookups

```rust
/// Returns the skill matching `name` (case-insensitive), or Err if not found.
pub fn resolve_skill_by_name(conn: &Connection, name: &str) -> Result<Skill, String>

/// Returns the attribute matching `name` (case-insensitive), or Err if not found.
pub fn resolve_attribute_by_name(conn: &Connection, name: &str) -> Result<Attribute, String>

/// Returns the tag matching `name` (case-insensitive), creating it if it doesn't exist.
pub fn find_or_create_tag(conn: &Connection, name: &str) -> Result<Tag, String>
```

These use `LOWER(name) = LOWER(?)` queries. Skills and attributes must already exist (error if not found). Tags are auto-created with the provided casing if no match exists.

### Bitmask Conversion

```rust
/// Parses "morning,evening" → 5, "anytime" → 15. Case-insensitive.
/// Returns Err for unrecognized names.
pub fn parse_time_of_day(input: &str) -> Result<i32, String>

/// Parses "mon,wed,fri" → 21, "every" or "everyday" → 127. Case-insensitive.
/// Returns Err for unrecognized names.
pub fn parse_days_of_week(input: &str) -> Result<i32, String>
```

Accepted time-of-day names: `morning`, `afternoon`, `evening`, `night`, `anytime`
Accepted day names: `mon`, `tue`, `wed`, `thu`, `fri`, `sat`, `sun`, `everyday`

These live in nq-core because the bitmask values are defined by the data model.

### Composite Quest Creation

```rust
/// Creates a quest and links tags, skills, and attributes in one transaction.
/// Resolves names to IDs: skills/attributes by case-insensitive match (error if not found),
/// tags by case-insensitive match (auto-created if not found).
pub fn add_quest_full(
    conn: &Connection,
    quest: NewQuest,
    tag_names: Vec<String>,
    skill_names: Vec<String>,
    attribute_names: Vec<String>,
) -> Result<Quest, String>
```

Wraps `add_quest` + `set_quest_tags` + `set_quest_links` in a single transaction. If any name resolution fails, the entire operation rolls back. Returns the created quest with linked IDs populated.

A parallel function for saga steps:

```rust
pub fn add_saga_step_full(
    conn: &Connection,
    step: NewSagaStep,
    tag_names: Vec<String>,
    skill_names: Vec<String>,
    attribute_names: Vec<String>,
) -> Result<Quest, String>
```

---

## CLI Binary (src-cli)

### Command Structure (clap)

```
nq <command> [options]

Commands:
  add-quest       Create a new quest
  add-saga-step   Add a step to an existing saga
  add-batch       Create quests from JSON on stdin
  list-quests     List quests as JSON
  list-sagas      List sagas as JSON
  list-tags       List tags as JSON
  list-skills     List skills as JSON
  list-attributes List attributes as JSON
```

### Output Format

All commands write JSON to stdout. Errors write JSON to stderr with a non-zero exit code.

**Success (single item):**
```json
{"ok": true, "id": "uuid-here", "title": "Quest title"}
```

**Success (batch):**
```json
{"ok": true, "created": [{"id": "uuid-1", "title": "Quest 1"}, {"id": "uuid-2", "title": "Quest 2"}]}
```

**Success (list):**
```json
[{"id": "uuid", "title": "...", "difficulty": "easy", "is_due": true, ...}]
```

**Error (single item):**
```json
{"ok": false, "error": "Skill 'Baking' not found. Available skills: Cooking, Cleaning, ..."}
```

**Error (batch):**
```json
{"ok": false, "errors": [{"index": 0, "error": "Missing title"}, {"index": 2, "error": "Invalid difficulty 'hard'"}]}
```

**Dry-run (batch):**
```json
{"ok": true, "dry_run": true, "would_create": [{"index": 0, "title": "Quest 1", "difficulty": "easy"}, ...]}
```

### add-quest Flow

1. Parse CLI flags via clap
2. Convert `--time-of-day` and `--days-of-week` names to bitmasks (error on invalid names)
3. Validate `--importance` is 0–5
4. Build `NewQuest` struct
5. Open database connection (db_path → init_db)
6. Call `add_quest_full()` with quest + name lists
7. Print success JSON to stdout, or error JSON to stderr with exit code 1

### add-saga-step Flow

Same as add-quest, but:
1. Validates `--saga` ID exists (query the saga table)
2. Builds `NewSagaStep` instead of `NewQuest`
3. Calls `add_saga_step_full()`

### add-batch Flow

1. Read JSON array from stdin
2. Deserialize into `Vec<BatchQuestInput>` (a struct matching NewQuest fields + tag/skill/attribute name lists)
3. Validate every item (bitmask names, importance range, skill/attribute name resolution, difficulty)
4. If any validation fails → print all errors to stderr, exit 1, nothing created
5. If `--dry-run` → print what would be created to stdout, exit 0
6. Wrap all `add_quest_full()` calls in a single transaction
7. Print created IDs to stdout

### list-quests Flow

1. Open database, call `get_quest_list()` — returns the same view as the GUI: standalone quests interleaved with saga slots (each showing the current active step). Saga steps are not shown as individual rows; use `list-sagas` for saga-level detail.
2. Apply filters in Rust (not SQL — quest list is small enough):
   - `--active`: filter to active items only
   - `--difficulty`: filter by difficulty (matches quest difficulty or saga slot's current step difficulty)
   - `--due`: filter to due items only
3. Resolve linked skill/attribute/tag IDs to names for the JSON output
4. Print JSON array to stdout

Note: the `--saga` filter from the requirements is dropped from `list-quests` — saga-level queries belong in `list-sagas`. The quest list view doesn't expose individual saga steps.

### list-sagas Flow

1. Call `get_sagas_with_progress()` — returns sagas with step counts and current step info
2. Print JSON array to stdout

### list-tags, list-skills, list-attributes

1. Call the corresponding `get_*()` function
2. Print JSON array to stdout

---

## Implementation Steps

Each step is a vertical slice that can be built, tested, and committed independently.

### Step 1: Crate Restructure

Extract shared code into a library crate. No new functionality.

**What changes:**
- Create workspace `Cargo.toml` at project root
- Create `nq-core/` with `Cargo.toml` and `src/lib.rs`
- Move `src-tauri/src/db.rs` → `nq-core/src/db.rs`
- Update `src-tauri/Cargo.toml` to depend on `nq-core` (path dependency)
- Update all `crate::db` imports in `commands.rs` and `main.rs` to `nq_core::db`
- Move shared dependencies (rusqlite, serde, uuid, dirs, libc) to nq-core

**Verify:** `cd src-tauri && cargo test` passes. `cargo tauri build --debug` succeeds. App runs normally.

### Step 2: CLI Scaffold + List Commands

The CLI binary exists and can query the database.

**What changes:**
- Create `src-cli/` with `Cargo.toml` (depends on nq-core + clap) and `src/main.rs`
- Add `src-cli` to workspace members
- Add `db_path()` function to nq-core
- Update `src-tauri/src/main.rs` to use the shared `db_path()` instead of inline path logic (single source of truth — prevents the GUI and CLI from silently diverging on where the database lives)
- Add WAL mode pragma and busy timeout to `init_db()`
- Implement: `list-quests`, `list-sagas`, `list-tags`, `list-skills`, `list-attributes`
- `list-quests` uses `get_quest_list()` to match the GUI view (standalone quests + saga slots), not raw quest rows
- Resolve linked IDs to names in list-quests output

**Verify:** Build CLI with `cargo build -p nq`. Run `./target/debug/nq list-tags` — see tags matching the app. Run `./target/debug/nq list-quests --due` — see due quests with skill/attribute names. Rebuild and run the GUI app to verify `db_path()` change didn't break anything.

### Step 3: add-quest

Create quests from the command line with full linking.

**What changes:**
- Add `parse_time_of_day()` and `parse_days_of_week()` to nq-core
- Add `resolve_skill_by_name()`, `resolve_attribute_by_name()`, `find_or_create_tag()` to nq-core
- Add `add_quest_full()` to nq-core (transactional quest + links)
- Implement `add-quest` command in CLI
- Tests for name resolution and bitmask parsing

**Verify:** Run `nq add-quest --title "Test Quest" --difficulty easy --type recurring --cycle-days 3 --skills "Cleaning" --tags "Inside"`. Open the app — quest appears on the quest list with Cleaning skill and Inside tag linked.

### Step 4: add-saga-step

Add steps to existing sagas from the command line.

**What changes:**
- Add `add_saga_step_full()` to nq-core (transactional step + links)
- Implement `add-saga-step` command in CLI

**Verify:** Get a saga ID with `nq list-sagas`. Run `nq add-saga-step --saga <id> --title "New step" --difficulty trivial`. Open the app — step appears in the saga in the correct position.

### Step 5: add-batch

Bulk quest creation from JSON input.

**What changes:**
- Define `BatchQuestInput` struct (NewQuest fields + tag/skill/attribute name lists)
- Implement `add-batch` command: stdin parsing, validation pass, transactional creation
- Implement `--dry-run` flag

**Verify:** Create a `test-batch.json` with 3 quests (varying difficulties, some with tags/skills). Run `cat test-batch.json | nq add-batch --dry-run` — see preview. Run without `--dry-run` — quests appear in app. Test error case: include one quest with invalid skill name → nothing created, all errors shown.

---

## Summary

The CLI is a thin layer over shared business logic. The bulk of the work is structural (workspace extraction) and adding name-based interfaces to the existing ID-based API. The CLI itself is straightforward: parse arguments, call shared functions, print JSON.

Five implementation steps, each independently testable:
1. **Crate restructure** — workspace + library extraction (verify: app still builds)
2. **CLI scaffold + list commands** — read-only CLI that queries the database (verify: JSON output matches app)
3. **add-quest** — create quests with full linking (verify: quest appears in app)
4. **add-saga-step** — add saga steps (verify: step appears in saga)
5. **add-batch** — bulk creation with dry-run (verify: batch of quests appears in app)
