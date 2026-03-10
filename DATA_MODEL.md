# Next Quest — MVP Data Model

## Entities

### Quest
A task in the quest pool that the app can assign to the player.

| Field | Type | Description |
|---|---|---|
| id | UUID | Unique identifier |
| title | String | Short quest name ("Take a shower", "Do the dishes") |
| xp_reward | Int | XP earned on completion |
| quest_type | Enum | `daily_recurring` or `one_off` |
| last_completed | Timestamp? | When this quest was last completed (null if never) |
| active | Bool | Whether this quest is in the available pool |

**Rules:**
- `daily_recurring` quests become available again the next day after completion
- `one_off` quests are marked `active = false` when completed
- Quests are seeded by the player, managed by the app

### Character
The player's RPG avatar. One per player (single-player app).

| Field | Type | Description |
|---|---|---|
| name | String | Character name |
| xp_total | Int | Cumulative XP earned (only goes up, never decays) |
| level | Int (derived) | Calculated from xp_total via level curve |

**Rules:**
- XP only increases — no decay, no streaks, no punishment for absence
- Level is derived, never stored separately (or cached + recalculated)

### QuestLog
Historical record of every completed quest. The source of truth for progression.

| Field | Type | Description |
|---|---|---|
| id | UUID | Unique identifier |
| quest_id | UUID | FK to Quest |
| completed_at | Timestamp | When the quest was completed |
| xp_earned | Int | XP awarded (snapshot — won't change if quest XP is later edited) |

**Rules:**
- Append-only — entries are never deleted or modified
- `xp_earned` is snapshotted at completion time (decoupled from quest's current `xp_reward`)

## Relationships

```
Character (1) ──── has many ──── QuestLog
Quest (1)     ──── has many ──── QuestLog
```

## Quest Selector (MVP Logic)

The engine that picks the next quest. MVP rules (simple, will evolve):

1. Filter to `active = true` quests
2. Exclude `daily_recurring` quests completed today
3. From remaining, pick one (weighted random — prefer least-recently-completed)

## Level Curve (TBD)

How XP maps to level. Placeholder: simple thresholds.
Exact curve to be designed when we build the progression system.

## What's Deferred to Phase 2+

- Skills, Attributes, Badges (quest → skill/attribute mappings)
- Quest Chains (quest → parent quest relationships)
- Energy Modes (mode field on quest or separate mode entity)
- Timer/AFK quests (duration-based quest type)
