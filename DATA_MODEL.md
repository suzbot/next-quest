# Next Quest — Data Model

## Current Entities (Phase 0)

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

**Derived values (computed, not stored):**
- `last_completed` — most recent completion timestamp for this quest
- `is_due` — whether the quest's cycle has elapsed since last completion

**Rules:**
- Recurring quests stay active after completion and refresh after `cycle_days` elapse.
- One-off quests are deactivated (`active = 0`) on completion. Can still be deleted.
- `quest_type` is an explicit field — type is never inferred from `cycle_days`.

### Completion
A visible record that you did a quest at a specific time.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| quest_id | UUID? (text) | FK to Quest. NULL if the quest has been deleted. |
| quest_title | String | Snapshot of quest title at completion time |
| completed_at | Timestamp | ISO 8601 completion time |

**Rules:**
- Completions snapshot the quest title so they remain self-contained after quest rename or deletion.
- Quests and completions are independent: deleting a quest orphans (not deletes) its completions.
- Individually deletable.

## Relationships

```
Quest (1) ──── has many ──── Completion
```

Completions reference quests via `quest_id`, but the FK is nullable — completions
survive quest deletion.

## Planned Entities (Phase 0.5+)

| Entity | Phase | Purpose |
|---|---|---|
| Character | 0.5 | Player's RPG avatar — name, XP, level |
| Attribute | 0.5 | Personal values (Health, Discipline, etc.) with XP/levels |
| Skill | 0.5 | Directional goals (Cooking, Fitness, etc.) with XP/levels |
| Quest Difficulty | 0.5 | Trivial/Easy/Moderate/Challenging/Epic — affects XP |
| Quest-Skill Link | 0.5 | Many-to-many: quests contribute XP to linked skills |
| Badge | 2 | Discrete achievements |
