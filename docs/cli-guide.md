# Next Quest CLI Guide

The `nq` CLI creates quests and queries data through the same business logic as the GUI app. Both read and write the same SQLite database. The GUI can be running while the CLI writes.

**Binary location:** `/usr/local/bin/nq` after running `./relscripts/install.sh`, or `./target/debug/nq` for debug builds.

**All output is JSON.** Success goes to stdout. Errors go to stderr with a non-zero exit code.

---

## Commands

### list-quests

Returns the quest list — same view as the GUI. Standalone quests and saga slots (showing each saga's current active step) interleaved by priority order.

```
nq list-quests [--active] [--difficulty <level>] [--due] [--day <day>] [--time <time>]
```

**Filters (all optional, combinable):**
- `--active` — only active quests
- `--difficulty <level>` — one of: trivial, easy, moderate, challenging, epic
- `--due` — only quests whose cycle has elapsed (does not check day-of-week or time-of-day)
- `--day <day>` — filter by day-of-week bitmask. Accepts: `today`, a day name (`monday`, `sunday`, `mon`, `sun`), or comma-separated (`mon,wed,fri`). Quests set to "everyday" always pass.
- `--time <time>` — filter by time-of-day window. Accepts: `now`, `morning`, `afternoon`, `evening`, `night`. Quests set to "anytime" always pass.

**Combining `--due` with `--day` and `--time`:** `--due` checks cycle eligibility only. To get quests that are both due and eligible right now, combine all three: `nq list-quests --due --day today --time now`.

**Output fields per item:**

| Field | Type | Description |
|---|---|---|
| id | string | Quest UUID |
| title | string | Quest title |
| item_type | string | "quest" or "saga" |
| quest_type | string | "recurring" or "one_off" |
| difficulty | string | trivial, easy, moderate, challenging, epic |
| is_due | bool | Whether the quest/saga is due now |
| active | bool | Whether the quest is active |
| cycle_days | int or null | Days between refreshes (null for one-off) |
| importance | int | 0–5 |
| sort_order | int | Priority position (higher = more prominent) |
| created_at | string | ISO 8601 timestamp |
| time_of_day | string[] | e.g. ["morning", "evening"] or ["anytime"] |
| days_of_week | string[] | e.g. ["mon", "wed", "fri"] or ["everyday"] |
| last_completed | string or null | ISO 8601 timestamp of last completion |
| dismissed_today | bool | Whether dismissed via "Not Today" |
| skills | string[] | Linked skill names |
| attributes | string[] | Linked attribute names |
| tags | string[] | Linked tag names |
| saga_id | string or null | Saga UUID (for saga items) |
| saga_name | string or null | Saga name (for saga items) |

### list-sagas

Returns all sagas with progress info.

```
nq list-sagas
```

**Output fields:**

| Field | Type | Description |
|---|---|---|
| id | string | Saga UUID |
| name | string | Saga name |
| cycle_days | int or null | Days between runs (null = one-off) |
| active | bool | Whether the saga is active |
| is_due | bool | Whether the saga is due |
| total_steps | int | Total step count |
| completed_steps | int | Steps completed in current run |
| created_at | string | ISO 8601 timestamp |
| last_run_completed_at | string or null | When the saga last finished all steps |

### list-tags

```
nq list-tags
```

Returns `[{id, name}, ...]`

### list-skills

```
nq list-skills
```

Returns skills with their parent attribute name and XP/level progression.

**Output fields:** id, name, attribute (name or null), xp, level, xp_for_current_level, xp_into_current_level

### list-attributes

```
nq list-attributes
```

Returns attributes with XP/level progression.

**Output fields:** id, name, xp, level, xp_for_current_level, xp_into_current_level

### list-history

Returns completion history — every quest completion and bonus entry, newest first.

```
nq list-history
```

**Output fields per item:**

| Field | Type | Description |
|---|---|---|
| id | string | Completion UUID |
| quest_id | string or null | Quest UUID. Null for bonus entries or deleted quests. |
| quest_title | string | Snapshot of quest title, or bonus description |
| completed_at | string | ISO 8601 timestamp |
| xp_earned | int | XP awarded |
| difficulty | string or null | Quest difficulty at completion time. Null for bonus entries or pre-migration records. |
| cycle_days | int or null | Effective cycle_days at completion time. Null for one-off quests, bonus entries, or pre-migration records. |
| skills | string[] or null | Linked skill names at completion time. Null for bonus entries or pre-migration records. |
| attributes | string[] or null | Linked attribute names at completion time. Null for bonus entries or pre-migration records. |
| tags | string[] or null | Linked tag names at completion time. Null for bonus entries or pre-migration records. |
| level_ups | object[] | Level-ups triggered (empty for history queries; populated at completion time only) |
| xp_awards | object[] | XP distribution breakdown (empty for history queries; populated at completion time only) |

**Example output:**
```json
[
  {
    "id": "abc-123",
    "quest_id": "def-456",
    "quest_title": "Take a shower",
    "completed_at": "2026-04-12T14:30:00Z",
    "xp_earned": 20,
    "difficulty": "easy",
    "cycle_days": 7,
    "skills": ["Healing"],
    "attributes": ["Health"],
    "tags": ["Inside"],
    "level_ups": [],
    "xp_awards": []
  },
  {
    "id": "ghi-789",
    "quest_id": null,
    "quest_title": "Morning Routine complete!",
    "completed_at": "2026-04-12T14:30:00Z",
    "xp_earned": 15,
    "difficulty": null,
    "cycle_days": null,
    "skills": null,
    "attributes": null,
    "tags": null,
    "level_ups": [],
    "xp_awards": []
  }
]
```

---

### add-quest

Create a new quest with optional skill, attribute, and tag links.

```
nq add-quest \
  --title "Quest title" \
  --difficulty easy \
  --type recurring \
  --cycle-days 3 \
  --importance 2 \
  --time-of-day "morning,evening" \
  --days-of-week "mon,wed,fri" \
  --skills "Cleaning,Cooking" \
  --attributes "Initiative (Monk)" \
  --tags "Inside,Computer"
```

**Required flags:**
- `--title` — quest title
- `--difficulty` — trivial, easy, moderate, challenging, epic
- `--type` — recurring or one_off

**Optional flags:**
- `--importance` — 0–5 (default: 0). Displayed as "!" marks in the app. Dominant scoring signal for the quest giver.
- `--cycle-days` — required for recurring, ignored for one_off. How many days before the quest is offered again.
- `--time-of-day` — comma-separated: morning, afternoon, evening, night, anytime. Default: anytime. The quest giver only offers this quest during these windows.
- `--days-of-week` — comma-separated: mon, tue, wed, thu, fri, sat, sun, everyday. Default: everyday. The quest giver only offers this quest on these days.
- `--tags` — comma-separated tag names. Auto-created if they don't exist.
- `--skills` — comma-separated skill names. Must match an existing skill (case-insensitive). Error lists available skills if not found.
- `--attributes` — comma-separated attribute names. Must match an existing attribute (case-insensitive).

**Ordering:** New quests always append to the end of the quest list. The user reorders them in the GUI.

**Success output:**
```json
{
  "ok": true,
  "id": "uuid",
  "title": "Quest title",
  "skills": ["Cleaning", "Cooking"],
  "attributes": ["Initiative (Monk)"],
  "tags": ["Inside", "Computer"]
}
```

**Error output:**
```json
{"ok": false, "error": "Skill 'Baking' not found. Available: Self Care, Movement, Medical, Cleaning, ..."}
```

### add-saga

Create a new saga. Returns the saga ID needed for `add-saga-step`.

```
nq add-saga --name "Spring Cleaning" --cycle-days 30
nq add-saga --name "Move to New Apartment"
```

**Required flags:**
- `--name` — saga name

**Optional flags:**
- `--cycle-days` — days between runs. Omit for a one-off saga.

**Success output:**
```json
{
  "ok": true,
  "id": "saga-uuid",
  "name": "Spring Cleaning",
  "cycle_days": 30
}
```

### add-saga-step

Add a step to an existing saga.

```
nq add-saga-step \
  --saga "saga-uuid-here" \
  --title "Step title" \
  --difficulty trivial \
  --skills "Planning"
```

**Required flags:**
- `--saga` — saga UUID. Get this from `list-sagas`.
- `--title` — step title
- `--difficulty` — same values as add-quest

**Optional flags:** same as add-quest (importance, time-of-day, days-of-week, tags, skills, attributes)

**Ordering:** Steps append after the last step in the saga. To add multiple steps in order, send them sequentially.

**Success output:**
```json
{
  "ok": true,
  "id": "step-uuid",
  "title": "Step title",
  "step_order": 3,
  "saga_id": "saga-uuid",
  "skills": ["Planning"],
  "attributes": [],
  "tags": []
}
```

### add-batch

Create multiple quests from a JSON array on stdin. Primary command for brain dumps.

```
echo '[
  {"title": "Quest 1", "difficulty": "easy", "quest_type": "one_off", "tags": "Computer"},
  {"title": "Quest 2", "difficulty": "trivial", "quest_type": "recurring", "cycle_days": 1, "skills": "Cleaning"}
]' | nq add-batch
```

**JSON fields per item:** same as add-quest flags, using snake_case. Required: title, difficulty, quest_type. Optional: importance, cycle_days, time_of_day, days_of_week, tags, skills, attributes.

**`--dry-run`:** validates all items and shows what would be created, without writing.

```
cat quests.json | nq add-batch --dry-run
```

**Error handling:** all-or-nothing. If any item fails validation, nothing is created and all errors are returned with their array index. Fix and resend.

**Success output:**
```json
{
  "ok": true,
  "created": [
    {"id": "uuid-1", "title": "Quest 1"},
    {"id": "uuid-2", "title": "Quest 2"}
  ]
}
```

**Error output:**
```json
{
  "ok": false,
  "errors": [
    {"index": 1, "error": "Invalid difficulty 'hard'. Valid values: trivial, easy, moderate, challenging, epic"}
  ]
}
```

**Dry-run output:**
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

---

## Conventions

**JSON field naming:** all snake_case. `cycle_days`, `quest_type`, `time_of_day`, etc.

**Name matching:** skill and attribute names are case-insensitive. `"cleaning"` matches `"Cleaning"`. Tag names are also matched case-insensitively, but new tags are created with the casing you provide.

**Time-of-day windows:** Morning (4am–noon), Afternoon (noon–5pm), Evening (5pm–9pm), Night (9pm–4am). These are hard filters — the quest giver won't offer a morning-only quest in the evening.

**Days-of-week:** Hard filters — a mon,wed,fri quest won't be offered on Tuesday.

**Difficulty levels:** trivial, easy, moderate, challenging, epic. These determine which quest giver lane the quest appears in (trivial → Castle Duties, easy → Adventures, moderate+ → Royal Quests) and affect XP earned.

**Importance (0–5):** The dominant scoring signal. Each level adds roughly 30 days of urgency. A quest with importance 3 will be offered as aggressively as a quest that's 90 days overdue. Use 0 for things that can wait, 4–5 for things that need doing soon.

---

## Workflows

### Brain dump

1. User describes tasks loosely
2. Cowork structures them into quest definitions
3. Preview: `echo '<json>' | nq add-batch --dry-run`
4. Create: `echo '<json>' | nq add-batch`
5. User reorders in the GUI if needed

### Saga step breakdown

1. Create the saga: `nq add-saga --name "Project Name"` (returns saga ID)
2. Add steps in order (they append sequentially):
   ```
   nq add-saga-step --saga <id> --title "Step 1" --difficulty trivial
   nq add-saga-step --saga <id> --title "Step 2" --difficulty easy
   nq add-saga-step --saga <id> --title "Step 3" --difficulty easy
   ```
3. User verifies and reorders in the GUI if needed

To add steps to an existing saga, get its ID with `nq list-sagas` first.

### Checking state before creating

Before adding quests, query existing data to avoid duplicates and use correct names:
```
nq list-quests              # what quests exist
nq list-sagas               # what sagas exist (get IDs for add-saga-step)
nq list-skills              # valid skill names for --skills
nq list-attributes          # valid attribute names for --attributes
nq list-tags                # existing tag names (new tags are auto-created)
nq list-history             # completion history with snapshot metadata
```

---

## Diagnostics

### audit-xp

Compare cached XP totals (stored on entities) against values derived by summing completion history. Reports per-entity: cached XP/level, derived XP/level, and delta.

```
nq audit-xp
```

Pre-migration completions (before snapshot data was recorded) are excluded from derived totals. The cutoff date is computed dynamically as the earliest completion with snapshot data.

### backfill-levelups

Replay post-cutoff completion history to detect skill level-ups and insert missing attribute bonus records. Idempotent — safe to run multiple times.

```
nq backfill-levelups
```

When a skill levels up, its parent attribute receives a bonus (equivalent to a Moderate one-off quest). This bonus was not recorded in completion history until recently. This command retroactively creates those records for all post-cutoff level-ups that are missing.

### recalculate

Sync cached XP on entity tables (character, attributes, skills) to match derived totals from completion history. Use as a repair tool if cached values drift.

```
nq recalculate
```

---

## What the CLI Does NOT Do

- **Complete quests** — that's the user's action in the app
- **Delete or deactivate quests** — too destructive for automation
- **Modify existing quests** — user does this in the app
- **Run the quest selector** — Cowork reads the database directly for analysis
