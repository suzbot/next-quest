# CLI Step 2: Scaffold + List Commands

**Goal:** A working CLI binary that can query the Next Quest database and return JSON output. Read-only — no writes yet.

**Design:** [cli-design.md](cli-design.md)

---

## What We're Building

A new binary (`nq`) with five list commands:

```
nq list-quests [--active] [--difficulty <level>] [--due]
nq list-sagas
nq list-tags
nq list-skills
nq list-attributes
```

Plus shared infrastructure: database path resolution, WAL mode, busy timeout.

---

## Changes

### 1. Add `db_path()` to nq-core

New public function in `nq-core/src/lib.rs` (not in db.rs — this is app-level config, not data logic):

```rust
use std::path::PathBuf;

/// Returns the path to the Next Quest database.
/// Creates the parent directory if it doesn't exist.
pub fn db_path() -> PathBuf {
    let dir = dirs::data_dir()
        .expect("Could not find data directory")
        .join("com.nextquest.desktop");
    std::fs::create_dir_all(&dir).expect("Failed to create data directory");
    dir.join("next-quest.db")
}
```

### 2. Update src-tauri/src/main.rs to use `db_path()`

Replace the inline path logic (lines 13–21) with:

```rust
let db_path = nq_core::db_path();
```

Single source of truth — both binaries find the database the same way.

### 3. Add WAL mode and busy timeout to `init_db()`

In `nq-core/src/db.rs`, inside `init_db()`, after opening the connection:

```rust
conn.execute_batch("PRAGMA journal_mode=WAL;").expect("Failed to enable WAL mode");
conn.busy_timeout(std::time::Duration::from_secs(5)).expect("Failed to set busy timeout");
```

WAL mode allows the GUI to read while the CLI writes. Busy timeout waits up to 5 seconds if the database is locked before returning an error. Both are harmless on in-memory connections (tests).

### 4. Create src-cli/ binary crate

**src-cli/Cargo.toml:**
```toml
[package]
name = "nq"
version = "0.1.0"
edition = "2021"

[dependencies]
nq-core = { path = "../nq-core" }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Add `"src-cli"` to workspace members in the root `Cargo.toml`.

### 5. CLI output types

The existing db structs have mixed serde naming conventions (some camelCase, some snake_case). Rather than changing those (which could break the GUI), the CLI defines its own thin output structs that serialize consistently as snake_case. These transform the db types, resolving IDs to names where appropriate.

Defined in `src-cli/src/main.rs` (or a separate `output.rs` if it gets long):

**Quest list item output:**
```rust
#[derive(Serialize)]
struct QuestOutput {
    id: String,
    title: String,
    item_type: String,          // "quest" or "saga"
    quest_type: String,         // "recurring" or "one_off"
    difficulty: String,
    is_due: bool,
    active: bool,
    cycle_days: Option<i32>,
    importance: i32,
    sort_order: i32,
    created_at: String,
    time_of_day: Vec<String>,   // ["morning", "evening"] instead of bitmask
    days_of_week: Vec<String>,  // ["mon", "wed", "fri"] instead of bitmask
    last_completed: Option<String>,
    dismissed_today: bool,
    skills: Vec<String>,        // names, not IDs
    attributes: Vec<String>,    // names, not IDs
    tags: Vec<String>,          // names, not IDs
    // saga-specific (only populated for item_type "saga")
    saga_id: Option<String>,
    saga_name: Option<String>,
}
```

**Saga output:**
```rust
#[derive(Serialize)]
struct SagaOutput {
    id: String,
    name: String,
    cycle_days: Option<i32>,
    active: bool,
    is_due: bool,
    total_steps: usize,
    completed_steps: usize,
    created_at: String,
    last_run_completed_at: Option<String>,
}
```

**Tag/Skill/Attribute outputs:**
```rust
#[derive(Serialize)]
struct TagOutput {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct SkillOutput {
    id: String,
    name: String,
    attribute: Option<String>,  // attribute name, not ID
    xp: i64,
    level: i32,
    xp_for_current_level: i64,
    xp_into_current_level: i64,
}

#[derive(Serialize)]
struct AttributeOutput {
    id: String,
    name: String,
    xp: i64,
    level: i32,
    xp_for_current_level: i64,
    xp_into_current_level: i64,
}
```

These expose the full computed state through consistent snake_case. The only transformation vs. raw db structs is resolving IDs to names and bitmasks to readable name arrays.

### 6. Bitmask-to-name conversion helpers

In the CLI (not nq-core — these are display-only, not business logic):

```rust
fn time_of_day_names(mask: i32) -> Vec<String>   // 5 → ["morning", "evening"]
fn days_of_week_names(mask: i32) -> Vec<String>   // 21 → ["mon", "wed", "fri"]
```

If all bits set (15 for TOD, 127 for DOW), returns `["anytime"]` / `["everyday"]` for readability.

### 7. Command implementations

**list-quests:**
1. Open db (`db_path()` → `init_db()`)
2. Call `get_quest_list()` — returns the GUI view: standalone quests + saga slots
3. Load all skills, attributes, tags (one call each) to build ID→name maps
4. Transform each `QuestListItem` into `QuestOutput`, resolving IDs to names
5. Apply filters:
   - `--active`: keep items where `active == true`
   - `--difficulty <level>`: match quest difficulty (or saga slot's current step difficulty)
   - `--due`: keep items where `is_due == true` (or saga slot's `is_saga_due == true`)
6. Print JSON array to stdout

**list-sagas:**
1. Call `get_sagas_with_progress()` — returns sagas with step counts and due status
2. Transform each `SagaWithProgress` into `SagaOutput`
3. Print JSON array to stdout

**list-tags:**
1. Call `get_tags()`
2. Transform to `TagOutput` (just drops `sort_order`)
3. Print JSON array to stdout

**list-skills:**
1. Call `get_skills()` and `get_attributes()`
2. Transform to `SkillOutput`, resolving `attribute_id` → attribute name
3. Print JSON array to stdout

**list-attributes:**
1. Call `get_attributes()`
2. Transform to `AttributeOutput`
3. Print JSON array to stdout

### 8. Error handling

All commands follow the same pattern:
- Success: JSON to stdout, exit 0
- Error: JSON to stderr (`{"ok": false, "error": "message"}`), exit 1

Database errors (can't open file, can't read) are the main failure mode for list commands.

---

## Verification

1. **nq-core tests pass:** `cd nq-core && cargo test`
2. **CLI builds:** `cargo build -p nq`
3. **List commands work:**
   ```
   ./target/debug/nq list-tags
   ./target/debug/nq list-skills
   ./target/debug/nq list-attributes
   ./target/debug/nq list-sagas
   ./target/debug/nq list-quests
   ./target/debug/nq list-quests --due
   ./target/debug/nq list-quests --difficulty easy
   ./target/debug/nq list-quests --active
   ```
   Compare output against what's visible in the app.
4. **GUI still works:** `cargo tauri build --debug && ./target/debug/next-quest` — verify quests load, quest giver works. This confirms the `db_path()` and WAL mode changes didn't break anything.
