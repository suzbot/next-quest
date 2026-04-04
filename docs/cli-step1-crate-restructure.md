# CLI Step 1: Crate Restructure

**Goal:** Extract shared data logic into a library crate (`nq-core`) so both the Tauri GUI and the CLI binary can depend on it. No new functionality — just restructuring.

**Design:** [cli-design.md](cli-design.md)

---

## What We're Building

A Cargo workspace with two members (third member `src-cli` added in Step 2):

```
nq/
  Cargo.toml              # NEW — workspace root
  nq-core/                # NEW — shared library crate
    Cargo.toml
    src/
      lib.rs              # re-exports db module as public
      db.rs               # MOVED from src-tauri/src/db.rs
  src-tauri/              # MODIFIED — now depends on nq-core
    Cargo.toml            # updated dependencies
    src/
      main.rs             # updated imports (db:: → nq_core::db::)
      commands.rs          # updated imports (crate::db → nq_core::db)
      tray.rs             # updated imports (crate::db → nq_core::db)
```

---

## Changes

### 1. Create workspace root Cargo.toml

New file at project root: `Cargo.toml`

```toml
[workspace]
members = ["nq-core", "src-tauri"]
resolver = "2"
```

### 2. Create nq-core library crate

**nq-core/Cargo.toml:**
```toml
[package]
name = "nq-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
dirs = "6"
libc = "0.2.183"
```

**nq-core/src/lib.rs:**
```rust
pub mod db;
```

**nq-core/src/db.rs:**
Moved from `src-tauri/src/db.rs` — no content changes. All tests come along with it.

### 3. Update src-tauri/Cargo.toml

Remove dependencies that moved to nq-core. Add nq-core as a path dependency.

```toml
[package]
name = "next-quest"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
nq-core = { path = "../nq-core" }

[build-dependencies]
tauri-build = { version = "2", features = [] }
serde_json = "1"
```

Note: `serde` and `serde_json` stay in src-tauri because `commands.rs` uses them directly for Tauri command serialization. The other crates (rusqlite, uuid, dirs, libc) move entirely to nq-core.

### 4. Update src-tauri/src/main.rs

Three changes:

1. Remove `mod db;` — db is no longer a local module
2. Replace `db::` calls with `nq_core::db::`
3. The `dirs` and `db_path` logic stays here for now (moves to nq-core in Step 2)

Affected lines:
- Line 4: remove `mod db;`
- Line 23: `db::init_db` → `nq_core::db::init_db`
- Line 26: `db::get_settings_db` → `nq_core::db::get_settings_db`
- Line 194: `db::get_next_quest` → `nq_core::db::get_next_quest`
- Line 194: `db::Lane::CastleDuties` → `nq_core::db::Lane::CastleDuties`

Alternatively, add `use nq_core::db;` at the top and leave the `db::` call sites unchanged.

### 5. Update src-tauri/src/commands.rs

- Line 6: `use crate::db;` → `use nq_core::db;`

No other changes — all `db::` call sites remain the same.

### 6. Update src-tauri/src/tray.rs

- Line 8: `use crate::db;` → `use nq_core::db;`

No other changes.

### 7. Update rust-analyzer.toml

Point rust-analyzer at the workspace root instead of the single crate:

```toml
linkedProjects = ["Cargo.toml"]
```

---

## Verification

1. **Tests pass:** `cd nq-core && cargo test`
2. **App builds:** `cargo tauri build --debug` (from `src-tauri/`)
3. **App runs normally:** launch and verify quests load, quest giver works, can complete a quest

---

## Risk

This is the riskiest step in the CLI project — it touches every source file in `src-tauri/src/` and reorganizes the build. If the workspace configuration or Tauri build integration has issues, it'll surface here. That's why it's isolated as its own step before adding any new code.
