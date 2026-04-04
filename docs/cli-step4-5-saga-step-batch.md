# CLI Steps 4–5: add-saga-step and add-batch

**Goal:** Complete the CLI's write commands. Add saga steps from the command line, and create quests in bulk from JSON input.

**Design:** [cli-design.md](cli-design.md)

---

## Step 4: add-saga-step

### What We're Building

```
nq add-saga-step \
  --saga "abc123-uuid-here" \
  --title "Measure bedroom for new furniture layout" \
  --difficulty trivial \
  --skills "Planning" \
  --tags "Inside"
```

Output:
```json
{
  "ok": true,
  "id": "uuid-here",
  "title": "Measure bedroom for new furniture layout",
  "step_order": 3,
  "saga_id": "abc123-uuid-here",
  "skills": ["Planning"],
  "attributes": [],
  "tags": ["Inside"]
}
```

### Changes

**nq-core: `add_saga_step_full()`**

```rust
pub fn add_saga_step_full(
    conn: &Connection,
    step: NewSagaStep,
    tag_names: Vec<String>,
    skill_names: Vec<String>,
    attribute_names: Vec<String>,
) -> Result<Quest, String>
```

Same pattern as `add_quest_full()`:
- Resolves skill/attribute names (error if not found), auto-creates tags
- Calls `add_saga_step()` (which validates saga exists and assigns step_order)
- Calls `set_quest_tags()` and `set_quest_links()` to wire up links
- Returns the created quest (saga step) with link IDs populated

**CLI command:**

```
nq add-saga-step [options]
```

**Required flags:**
- `--saga` — saga ID (UUID). Cowork resolves IDs from `list-sagas` before calling.
- `--title` — step title
- `--difficulty` — validated via `Difficulty::try_from_str()`

**Optional flags:**
- `--step-order` — integer position (default: append after last step)
- `--importance` — 0–5 (default: 0)
- `--time-of-day` — comma-separated names (default: anytime)
- `--days-of-week` — comma-separated names (default: everyday)
- `--tags` — comma-separated tag names
- `--skills` — comma-separated skill names
- `--attributes` — comma-separated attribute names

**Note on `--step-order`:** The existing `add_saga_step()` always appends after the last step. To support explicit positioning, we'll add an optional `step_order` field to `NewSagaStep` — if provided, use it; if not, auto-assign. This is a small change to the existing function.

**Success output includes `step_order`** so Cowork can confirm position.

### Tests

- `add_saga_step_full` with links — step created in saga with correct links
- `add_saga_step_full` bad saga ID — error
- `add_saga_step_full` bad skill name — error, no step created

---

## Step 5: add-batch

### What We're Building

```
echo '[
  {"title":"Quest 1","difficulty":"easy","quest_type":"one_off","tags":"Computer"},
  {"title":"Quest 2","difficulty":"trivial","quest_type":"recurring","cycle_days":1}
]' | nq add-batch
```

Success output:
```json
{
  "ok": true,
  "created": [
    {"id": "uuid-1", "title": "Quest 1"},
    {"id": "uuid-2", "title": "Quest 2"}
  ]
}
```

Error output (nothing created):
```json
{
  "ok": false,
  "errors": [
    {"index": 1, "error": "Skill 'Baking' not found. Available: ..."}
  ]
}
```

Dry-run output:
```json
{
  "ok": true,
  "dry_run": true,
  "would_create": [
    {"index": 0, "title": "Quest 1", "difficulty": "easy", "quest_type": "one_off"},
    {"index": 1, "title": "Quest 2", "difficulty": "trivial", "quest_type": "recurring"}
  ]
}
```

### Changes

**Batch input struct (CLI):**

```rust
#[derive(Deserialize)]
struct BatchQuestInput {
    title: String,
    difficulty: String,
    quest_type: String,
    #[serde(default)]
    importance: i32,
    cycle_days: Option<i32>,
    time_of_day: Option<String>,
    days_of_week: Option<String>,
    tags: Option<String>,       // comma-separated
    skills: Option<String>,     // comma-separated
    attributes: Option<String>, // comma-separated
}
```

Uses `String` for difficulty/quest_type (not the enums) so we can validate each item and collect all errors before failing.

**CLI command:**

```
nq add-batch [--dry-run]
```

Reads JSON array from stdin. No other flags — all quest metadata is in the JSON.

**Flow:**

1. Read stdin to string, deserialize as `Vec<BatchQuestInput>`
2. **Validation pass** — for each item:
   - Validate difficulty, quest_type via `try_from_str`
   - Validate importance 0–5
   - Validate cycle_days present for recurring
   - Parse time-of-day and days-of-week names
   - Resolve skill and attribute names (must exist)
   - Collect all errors with their array index
3. If any errors → print all to stderr, exit 1, nothing created
4. If `--dry-run` → print what would be created, exit 0
5. **Creation pass** — call `add_quest_full()` for each item sequentially (sort_order auto-increments correctly since each appends after the previous)
6. Print created IDs and titles to stdout

**All-or-nothing:** Validation catches all errors before any writes. If validation passes but a write fails mid-batch (unlikely — schema errors would be caught in validation), the already-created quests remain. This is acceptable for the 5–10 item batches we're targeting.

### Tests

No new nq-core tests needed — `add_quest_full` is already tested. The batch logic is CLI-layer orchestration (validation loop + creation loop).

---

## Verification (combined)

1. **Tests pass:** `cargo test`
2. **add-saga-step works:**
   ```
   # Get a saga ID
   ./target/debug/nq list-sagas

   # Add a step
   ./target/debug/nq add-saga-step \
     --saga <id> \
     --title "Test saga step" \
     --difficulty trivial

   # Verify in app — step appears in correct saga
   ```
3. **add-saga-step error cases:** bad saga ID, bad skill name
4. **add-batch works:**
   ```
   echo '[{"title":"Batch 1","difficulty":"easy","quest_type":"one_off"},{"title":"Batch 2","difficulty":"trivial","quest_type":"recurring","cycle_days":1}]' | ./target/debug/nq add-batch

   # Verify in app — both quests appear
   ```
5. **add-batch dry-run:**
   ```
   echo '[{"title":"Dry Run","difficulty":"easy","quest_type":"one_off"}]' | ./target/debug/nq add-batch --dry-run

   # Verify nothing created in app
   ```
6. **add-batch error collection:**
   ```
   echo '[{"title":"Good","difficulty":"easy","quest_type":"one_off"},{"title":"Bad","difficulty":"hard","quest_type":"one_off"}]' | ./target/debug/nq add-batch

   # Should fail with error on index 1, nothing created
   ```
7. **GUI still works:** rebuild and launch
