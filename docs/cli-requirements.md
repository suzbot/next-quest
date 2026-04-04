# Next Quest CLI — Requirements

**Status:** Approved. Functionally approved by Cowork, technically reviewed by Claude Code.

**Architecture:** The CLI is a separate lightweight binary (`nq`) that shares the data logic (`db.rs`) with the Tauri GUI app via a Rust library crate. Same business logic, validation, and sort-order management as the GUI — no Tauri/webview dependency. Both binaries read/write the same SQLite database.

**Context:** Cowork currently has read-only access to the Next Quest database via SQLite. That covers the daily insight report. What's missing is the ability to *create* quests and saga steps through the app's business logic, so that Cowork can turn brain dumps, email/calendar items, and saga breakdowns into real quests without risking database integrity.

---

## Use Cases

### 1. Brain dump → quest creation
User talks loosely at Cowork. Cowork structures the input into quests with metadata. Cowork calls the CLI to add them.

### 2. Saga step breakdown
User picks a saga that's stuck at "make a to-do list." Cowork asks questions, proposes steps. Cowork calls the CLI to add steps to the saga.

### 3. Email/calendar → quest creation (future)
Cowork reads Gmail/GCal, identifies items that imply tasks, cross-references against existing quests, and adds new ones through the CLI.

---

## Commands

### `add-quest`

Create a new quest. Goes through the same validation and business logic as the UI.

**Required fields:**
- `--title` — quest title (string)
- `--difficulty` — one of: trivial, easy, moderate, challenging, epic
- `--type` — one of: recurring, one_off

**Optional fields (with defaults matching the UI):**
- `--importance` — 0–5 (default: 0)
- `--cycle-days` — integer, required if type is recurring, ignored if one_off
- `--time-of-day` — named values like "morning,afternoon" (default: anytime). CLI converts to bitmask internally.
- `--days-of-week` — named values like "mon,wed,fri" (default: every day). CLI converts to bitmask internally.
- `--tags` — comma-separated tag names. Creates new tags if they don't exist.
- `--skills` — comma-separated skill names to link
- `--attributes` — comma-separated attribute names to link

**Output:** JSON with the created quest's ID and confirmation.

**Ordering:** New quests always append to the end of the quest list (highest sort_order + 1, unified across quests and sagas, same as the UI). There is no `--sort-order` flag — priority ordering is a user activity done in the GUI. Cowork creates quests; the user decides where they sit in the priority list.

**Example:**
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

### `add-saga-step`

Add a step to an existing saga. Enforces step ordering and saga business logic.

**Required fields:**
- `--saga` — saga ID (not name, since saga names aren't enforced unique. Cowork resolves IDs from the database before calling.)
- `--title` — step title
- `--difficulty` — same as add-quest

**Optional fields:**
- `--step-order` — integer position in the saga (default: append after the last step)
- Same optional fields as add-quest (importance, time-of-day, etc.)

**Output:** JSON with the created step's ID, its position in the saga, and confirmation.

**Example:**
```
nq add-saga-step \
  --saga "abc123-uuid-here" \
  --title "Measure bedroom for new furniture layout" \
  --difficulty trivial
```

### `add-batch`

Accept a JSON array of quest definitions from stdin. Each element uses the same fields as `add-quest`. Supports `--dry-run`.

This is the primary command for the brain dump workflow, which typically produces 5–10 quests at once.

**Error handling:** All-or-nothing. The entire batch is validated before any writes. If any item fails validation, nothing is created and the response includes all errors so they can be fixed in one pass. This is simpler than partial success (no need to track which items were created and which weren't) and fine for batches of 5–10 items — fix the errors and resend.

**Output:** JSON with an array of created quest IDs on success, or an array of per-item validation errors on failure.

**Example:**
```
cat quests.json | nq add-batch
```

Or inline:
```
echo '[{"title":"Quest 1","difficulty":"easy","type":"one_off"},{"title":"Quest 2","difficulty":"trivial","type":"recurring","cycle_days":1}]' | nq add-batch
```

### `list-quests`

Return quests as JSON with all computed fields included (is_due, linked skill/attribute names, saga progress). Uses the same business logic as the GUI rather than raw database rows.

**Filters:**
- `--active` — only active quests
- `--saga` — filter by saga ID
- `--difficulty` — filter by difficulty
- `--due` — only quests currently due

**Output:** JSON array of quest objects.

### `list-sagas`

JSON output with saga metadata and current step status. Same rationale as list-quests — exposes computed fields through business logic.

### `list-tags`, `list-skills`, `list-attributes`

Return the current set of tags, skills, and attributes as JSON arrays. Cowork needs these to correctly populate fields on add-quest and validate input before calling.

---

## Design Considerations

### 1. JSON output
All commands default to JSON output for programmatic parsing. The primary consumer is Cowork (an AI agent). A human-readable mode can be added later if needed.

### 2. Validation and error handling
The CLI validates input the same way the UI does — rejects invalid difficulty values, enforces cycle_days on recurring quests, etc. Errors are clear and JSON-parseable.

### 3. Named values for bitmasks
Time-of-day and days-of-week are bitmasks internally, but the CLI accepts human-readable names and converts them.

Example: `--time-of-day "morning,evening"` instead of `--time-of-day 5`

### 4. Tag and skill matching
Tags and skills are matched by name (case-insensitive). If a tag doesn't exist, create it. If a skill name doesn't match, return an error rather than silently ignoring.

### 5. Dry run mode
`add-batch` supports a `--dry-run` flag that validates all items and shows what would be created without writing to the database. Useful for Cowork to confirm structured output before committing. Not needed on `add-quest` or `add-saga-step` — single-item commands either succeed or return a clear validation error, so there's nothing to preview.

### 6. Database concurrency
The GUI app may be running while the CLI writes. SQLite WAL mode handles this — it allows concurrent reads and queues concurrent writes briefly instead of failing immediately. WAL mode is not currently enabled; the CLI work will add `PRAGMA journal_mode=WAL` to database initialization (benefits both GUI and CLI). The CLI also handles SQLITE_BUSY gracefully with brief retries rather than erroring immediately.

WAL mode creates two sidecar files (`-wal` and `-shm`) next to the database. These must stay with the `.db` file — not a concern here since the database lives in a fixed app data directory.

### 7. JSON field naming
All JSON input and output uses snake_case (`cycle_days`, `time_of_day`, `quest_type`). This matches Rust's default serde serialization, the CLI flag style (`--cycle-days`), and avoids configuration overhead. Cowork should use snake_case when constructing JSON for `add-batch`.

### 8. Saga step ordering guidance
There is no `--sort-order` flag on any command. For saga steps, `--step-order` controls position within the saga (defaults to appending after the last step). To add multiple steps in a specific order, send them sequentially in the desired order — each will append after the previous. To insert a step at a specific position, use `--step-order` with the target position number.

---

## What Cowork Does NOT Need

- **Completing quests** — that's the user's action in the app. Cowork should never mark things done.
- **Deleting or deactivating quests** — too destructive for automation. User does this in the app.
- **Modifying existing quests** — risky for automation to change things the user set up. Could revisit later, but not in v1.
- **Quest selection / scoring** — Cowork reads the database for analysis but doesn't need to run the selector.

---

## Current Reference Data

For context, here's what exists in the database today that the CLI would need to validate against:

**Tags:** OUtside, Computer, Phone, Away

**Skills (grouped by attribute):**
- Health: Self Care, Movement, Medical
- Initiative: Cleaning, Cooking, Handiness
- Responsibility: Bureaucracy, Planning, Money, Career
- Connection: Community, Naturalism, Friendship, Animal Care
- Creativity: Reading, Music, ART
- Intellect: Information, Technology, Problem Solving

**Difficulty levels:** trivial, easy, moderate, challenging, epic

**Time-of-day values:** Morning (4am–noon), Afternoon (noon–5pm), Evening (5pm–9pm), Night (9pm–4am)

**Days-of-week values:** Mon, Tue, Wed, Thu, Fri, Sat, Sun
