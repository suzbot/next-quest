# Phase 2F.5: "Cleanup" — Design

## Overview

Code quality pass and UI polish. Three areas: parameter structs to reduce blast radius of adding fields, a test helper to cut boilerplate, and resequencing for attributes and skills on the Character tab.

## 1. Parameter Structs

### Problem

`add_quest` has 7 positional params, `update_quest` has 8, `add_saga_step` has 6, `get_next_quest` has 3. Every time a field is added, every call site (60+ for add_quest) must be updated.

### Approach

Replace positional params with structs that have `Default` implementations.

**`NewQuest` struct:**
```rust
pub struct NewQuest {
    pub title: String,
    pub quest_type: QuestType,
    pub cycle_days: Option<i32>,
    pub difficulty: Difficulty,
    pub time_of_day: i32,
    pub days_of_week: i32,
}
```

`add_quest(conn, quest: NewQuest)` — single struct param after `conn`.

**`QuestUpdate` struct:**
```rust
pub struct QuestUpdate {
    pub title: Option<String>,
    pub quest_type: Option<QuestType>,
    pub cycle_days: Option<i32>,
    pub difficulty: Option<Difficulty>,
    pub time_of_day: Option<i32>,
    pub days_of_week: Option<i32>,
}
```

`update_quest(conn, quest_id, update: QuestUpdate)` — all optional fields in one struct.

**`NewSagaStep` struct:**
```rust
pub struct NewSagaStep {
    pub saga_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub time_of_day: i32,
    pub days_of_week: i32,
}
```

**`QuestSelectionParams` struct:**
```rust
pub struct QuestSelectionParams {
    pub skip_counts: HashMap<String, i32>,
    pub exclude_quest_id: Option<String>,
}
```

### Tauri command wrappers

The Tauri commands already deserialize from JSON, so they can accept the same structs directly (with `Deserialize` derived). The wrapper layer becomes trivial.

### Migration strategy

1. Define structs with `Default` where sensible
2. Update `db.rs` functions to accept structs
3. Update `commands.rs` wrappers
4. Update all test call sites

## 2. Test Helper

### Problem

60 calls to `add_quest` in tests, most using the same defaults (`QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127`). Adding a field means editing all 60.

### Approach

A builder-style helper in the test module:

```rust
fn test_quest(conn: &Connection, title: &str) -> Quest {
    add_quest(conn, NewQuest {
        title: title.to_string(),
        ..NewQuest::default()
    }).unwrap()
}
```

Default: Recurring, 1-day cycle, Easy, all times, all days.

Tests that need specific values override just what they care about:
```rust
let q = add_quest(&conn, NewQuest {
    title: "Hard task".into(),
    difficulty: Difficulty::Epic,
    ..NewQuest::default()
}).unwrap();
```

Similarly for saga steps:
```rust
fn test_saga_step(conn: &Connection, saga_id: &str, title: &str) -> Quest {
    add_saga_step(conn, NewSagaStep {
        saga_id: saga_id.to_string(),
        title: title.to_string(),
        ..NewSagaStep::default()
    }).unwrap()
}
```

## 3. Attribute and Skill Resequencing

### Problem

Attributes and skills on the Character tab have a fixed display order (by `sort_order`). There's no way to reorder them.

### Approach

Same pattern as saga step resequencing (proven working):

**Backend:**
- `reorder_attributes(conn, attr_ids: Vec<String>)` — takes full ordered list of attribute IDs, sets sort_order = position. NOT pair swaps — full list rebuild avoids the neighbor-oscillation bug.
- `reorder_skills(conn, skill_ids: Vec<String>)` — same pattern for skills

**Frontend (Character tab):**
- Attribute and skill meter rows get `tabindex="0"`, `onkeydown`, `onpointerdown`
- **Alt+Arrow** moves an attribute/skill one position, calls reorder, re-renders, re-focuses
- **Arrow** moves focus between rows
- **Drag-and-drop** with same pointer event pattern as saga steps
- Drop targets scoped to attribute rows or skill rows (don't mix the two)

**Scoping rules:**
- Attributes reorder among attributes only
- Skills reorder among skills only (across attribute groups — skills are a flat list sorted by sort_order)

## Implementation Order

1. **2F.5-1: Parameter structs** — Define `NewQuest`, `QuestUpdate`, `NewSagaStep`, `QuestSelectionParams`. Update db.rs function signatures and commands.rs wrappers. Mechanical, no behavior change.

2. **2F.5-2: Test helper** — Add `test_quest()` and `test_saga_step()` helpers. Update all 60+ test call sites to use them. Verify all tests pass.

3. **2F.5-3: Attribute/skill resequencing** — Backend reorder functions (full ordered ID list, not sort_order pair swaps) + command wrappers. Frontend keyboard (Alt+Arrow) and drag-and-drop on Character tab meter rows. Both methods must support full resequencing to any position — repeated Alt+Arrow presses must keep moving in the same direction, not oscillate between two neighbors (previous failed implementation used pair swaps which caused this). Drag-and-drop allows moving to any position in one gesture. Testing: add 5+ attributes, Alt+ArrowDown on first → becomes second, press again → becomes third. Drag last to first position. Same tests for skills.
