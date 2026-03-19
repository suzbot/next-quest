# Next Quest — Data Model

## Current Entities

### Quest
A template for something you do. Can be recurring or one-off.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| title | String | Short quest name ("Take a shower", "File taxes") |
| quest_type | Enum | `recurring` or `one_off` |
| cycle_days | Integer? | Days between refreshes. Always set for recurring, always NULL for one-off. |
| sort_order | Integer | Player-defined position. Higher = more prominent. |
| active | Bool (int) | 1 = active, 0 = deactivated (completed one-off) |
| created_at | Timestamp | ISO 8601 creation time |
| difficulty | Enum | `trivial`, `easy`, `moderate`, `challenging`, `epic` |
| time_of_day | Integer | Bitmask: Morning=1, Afternoon=2, Evening=4. Default 7 (all). 0 or 7 = anytime. |
| days_of_week | Integer | Bitmask: Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64. Default 127 (every day). |

**Derived values (computed, not stored):**
- `last_completed` — most recent completion timestamp for this quest
- `is_due` — whether the quest's cycle has elapsed since last completion
- `skill_ids` — IDs of linked skills (from quest_skill join table)
- `attribute_ids` — IDs of linked attributes (from quest_attribute join table)

**Rules:**
- Recurring quests stay active after completion and refresh after `cycle_days` elapse.
- One-off quests are deactivated (`active = 0`) on completion. Can still be deleted.
- `quest_type` is an explicit field — type is never inferred from `cycle_days`.
- Difficulty defaults to `easy`. Display labels: Trivial, Easy, Fair, Hard, Epic.
- Time-of-day defaults to 7 (all times). All or none selected = anytime.
- Days-of-week defaults to 127 (every day). All or none selected = every day.

### Completion
A visible record that you did a quest at a specific time.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| quest_id | UUID? (text) | FK to Quest. NULL if the quest has been deleted. |
| quest_title | String | Snapshot of quest title at completion time |
| completed_at | Timestamp | ISO 8601 completion time |
| xp_earned | Integer | XP awarded for this completion |

**Rules:**
- Completions snapshot the quest title so they remain self-contained after quest rename or deletion.
- Quests and completions are independent: deleting a quest orphans (not deletes) its completions.
- Individually deletable. Deleting a completion does NOT reduce XP — XP only goes up.

### Character
The player's RPG avatar. Exactly one row, seeded on first launch.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Character name (default: "Adventurer") |
| xp | Integer | Total accumulated XP |

**Derived values (computed via level curve):**
- `level` — current level
- `xp_for_current_level` — XP required to complete current level
- `xp_into_current_level` — progress into current level

### Attribute
A personal value category. Seeded on first launch: Health, Pluck, Knowledge, Connection, Responsibility.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Attribute name |
| sort_order | Integer | Display order |
| xp | Integer | Total accumulated XP |

**Derived values:** Same level fields as Character, using the Attribute level scale.

### Skill
A directional goal, mapped to one attribute. Seeded on first launch with 12 skills.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Skill name |
| attribute_id | UUID (text) | FK to parent Attribute |
| sort_order | Integer | Display order |
| xp | Integer | Total accumulated XP |

**Derived values:** Same level fields as Character, using the Skill level scale.

**Default skills and their attributes:**
- Health: Cooking, Healing, Acrobatics
- Pluck: Cleaning, Crafting
- Knowledge: Language, Technology
- Connection: Nature, Community, Sociality, Animal Handling
- Responsibility: Bureaucracy, Logistics

### Quest-Skill Link (quest_skill)
Many-to-many join table between quests and skills.

| Field | Type | Description |
|---|---|---|
| quest_id | UUID (text) | FK to Quest |
| skill_id | UUID (text) | FK to Skill |

### Quest-Attribute Link (quest_attribute)
Many-to-many join table between quests and attributes.

| Field | Type | Description |
|---|---|---|
| quest_id | UUID (text) | FK to Quest |
| attribute_id | UUID (text) | FK to Attribute |

## Relationships

```
Quest (1) ──── has many ──── Completion
Quest (M) ──── many-to-many ──── Skill       (via quest_skill)
Quest (M) ──── many-to-many ──── Attribute   (via quest_attribute)
Attribute (1) ──── has many ──── Skill
Character (singleton)
```

Completions reference quests via `quest_id`, but the FK is nullable — completions
survive quest deletion. Deleting a quest also cleans up its link rows in quest_skill
and quest_attribute.

## XP Engine

### XP Calculation
XP earned per completion: `base * difficulty_mult * cycle_mult`, rounded.

- **Base:** 10
- **Difficulty multiplier:** Trivial=1, Easy=2, Fair=4, Hard=7, Epic=12
- **Cycle multiplier:** One-off=3, Recurring=sqrt(cycle_days)

### XP Distribution
On quest completion, the calculated XP is awarded to:
1. **Character** — always
2. **Linked attributes** — each gets the full XP amount
3. **Linked skills** — each gets the full XP amount

### Skill Level-Up Bonus
When a skill levels up from an XP award, its parent attribute receives a 70 XP bump.

### Level Curve
Fibonacci-style progression with different seeds per scale:

| Scale | Seed (level 1 cost, level 2 cost) |
|---|---|
| Character | 300, 500 |
| Attribute | 60, 100 |
| Skill | 30, 50 |

Each subsequent level costs the sum of the two prior levels (e.g., Character: 300, 500, 800, 1300, 2100...).

### Settings
App configuration. Single row, seeded on first launch.

| Field | Type | Default | Description |
|---|---|---|---|
| id | Integer | 1 | Always 1 (single row) |
| cta_enabled | Integer | 0 | Call to Adventure on/off (0/1) |
| cta_interval_minutes | Integer | 20 | Polling interval in minutes |
| debug_scoring | Integer | 0 | Show score breakdown in quest giver (0/1) |

## Quest Selector

The quest giver picks quests using a scoring system with hard filters and soft ranking.

### Candidate Pool
1. All active quests
2. Hard-filter: time-of-day bitmask matches current local hour (Morning 4am–noon, Afternoon noon–5pm, Evening 5pm–4am)
3. Hard-filter: days-of-week bitmask includes today
4. Split into **due** and **not-due** pools; due always preferred

### Scoring (due quests)
```
score = overdue_ratio - skip_penalty + list_order_bonus
```
- **Overdue ratio**: `days_since_completed / cycle_days` (min 1.0). Never-completed recurring: `(days_since_created + cycle) / cycle`. One-off: `(days_since_created + 9) / 9`.
- **Skip penalty**: `skip_count × 0.5`. Resets daily. Recorded on "Something Else" (main) and "Run" (overlay). "Hide in the Shadows" does not count.
- **List order bonus**: `0.01 × sort_order / max_sort_order`. Top-of-list quests score slightly higher.

### Scoring (not-due fallback)
Only reached when due pool is empty or all due scores ≤ 0.
```
score = days_since_completed / max_days - skip_penalty + list_order_bonus
```

### Behavior
- "Something Else" / "Run": records a skip, then re-scores. The just-skipped quest is excluded from the next pick (unless it's the only quest).
- Exhaustion fallback: if all scores ≤ 0, returns the least-negative.
- Skip counts are in-memory, reset at local midnight or app restart.

## Planned Entities (Phase 2+)

| Entity | Phase | Purpose |
|---|---|---|
| Badge | 2 | Discrete achievements |
