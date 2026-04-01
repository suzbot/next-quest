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
| time_of_day | Integer | Bitmask: Morning=1, Afternoon=2, Evening=4, Night=8. Default 15 (all). 0 or 15 = anytime. |
| days_of_week | Integer | Bitmask: Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64. Default 127 (every day). |
| saga_id | UUID? (text) | FK to Saga. NULL for regular quests, set for saga steps. |
| step_order | Integer? | Position within saga. NULL for regular quests. |
| last_completed | Timestamp? | Stored on the quest. Updated by `complete_quest`, editable via date picker. Not derived from completions — completion history is a read-only log. |
| importance | Integer | 0–5. Default 0. Dominant scoring signal (importance × 30.0). Displayed as "!" marks. |

**Derived values (computed, not stored):**
- `is_due` — whether the quest's cycle has elapsed since last completion
- `skill_ids` — IDs of linked skills (from quest_skill join table)
- `attribute_ids` — IDs of linked attributes (from quest_attribute join table)
- `tag_ids` — IDs of linked tags (from quest_tag join table)

**Rules:**
- Recurring quests stay active after completion and refresh after `cycle_days` elapse.
- One-off quests are deactivated (`active = 0`) on completion. Can still be deleted.
- `quest_type` is an explicit field — type is never inferred from `cycle_days`.
- Difficulty defaults to `easy`. Display labels: Trivial, Easy, Fair, Hard, Epic.
- Time-of-day defaults to 7 (all times). All or none selected = anytime.
- Days-of-week defaults to 127 (every day). All or none selected = every day.
- Saga steps (`saga_id` set) are not shown as individual rows on the Quest List tab. Instead, each saga appears as a single "saga slot" showing its current active step, interleaved with regular quests by sort_order. Saga steps also appear in the Saga tab and through the quest giver.
- Saga step XP uses the parent saga's cycle_days (not the one-off multiplier). One-off saga steps use 3x.

### Completion
A visible record that you did a quest at a specific time, or that a bonus was awarded.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| quest_id | UUID? (text) | FK to Quest. NULL for deleted quests or bonus entries. |
| quest_title | String | Snapshot of quest title, or bonus description (e.g., "Morning Routine complete!") |
| completed_at | Timestamp | ISO 8601 completion time |
| xp_earned | Integer | XP awarded for this completion |

**Rules:**
- Completions snapshot the quest title so they remain self-contained after quest rename or deletion.
- Quests and completions are independent: deleting a quest orphans (not deletes) its completions.
- Saga completion bonuses and campaign completion bonuses are also recorded as completions with `quest_id: NULL` and a descriptive title. These appear in the history list and count toward daily XP stats.
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

### Saga
A multi-step goal with ordered sub-quests. Can be one-off or recurring.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Saga title |
| cycle_days | Integer? | NULL = one-off, set = recurring (days after run completion before reset) |
| sort_order | Integer | Display order. Shares a unified namespace with quest sort_order — sagas and quests are interleaved on the quest list by this value. |
| active | Bool (int) | 1 = active |
| created_at | Timestamp | ISO 8601 creation time |
| last_run_completed_at | Timestamp? | Stamped when all steps complete a run. Drives recurring reset and cooldown styling. Not recomputed — permanent once stamped. |

**Rules:**
- Saga steps are quests with `saga_id` and `step_order` set.
- A saga's "current run" starts after `last_run_completed_at` (or `created_at` for new sagas). A step is complete in the current run if its `last_completed > last_run_completed_at`.
- A saga is due when: one-off and incomplete, or recurring and `cycle_days` have elapsed since `last_run_completed_at`, or user has started a new run early (any steps completed after stamp).
- The quest giver surfaces one active step per saga: the first step (by step_order) not yet completed in the current run.
- When all steps have a current-run completion, `last_run_completed_at` is stamped and a completion bonus is awarded.

### Campaign
A user-defined collection of criteria tracking progress toward a larger accomplishment.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Campaign title ("Spring Cleaning 2026") |
| created_at | Timestamp | ISO 8601. Only completions after this count (no retroactive credit). |
| completed_at | Timestamp? | NULL until all criteria met, then stamped. |

**Rules:**
- Campaigns are active until all criteria are met.
- Criteria are locked after creation — edits are made by duplicating and creating a new version (duplication is Phase 2G.2-4).
- Campaign name can be renamed at any time.
- Deleting a campaign deletes its criteria and orphans any accomplishment.
- One quest/saga completion can count toward multiple campaigns simultaneously.

### Campaign Criterion
A single requirement within a campaign. Tracks completions of a specific quest or saga.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| campaign_id | UUID (text) | FK to Campaign |
| target_type | String | `"quest_completions"` or `"saga_completions"` (extensible string enum) |
| target_id | UUID (text) | FK to Quest or Saga |
| target_count | Integer | How many completions needed |
| current_count | Integer | Starts at 0, increments on qualifying completions |
| sort_order | Integer | Display order within campaign |

**Derived values (computed, not stored):**
- `target_name` — looked up from quest title or saga name. "Deleted quest" / "Deleted saga" if target removed.

**Constraints:** Unique on `(campaign_id, target_type, target_id)` — no duplicate criteria for same target within a campaign.

**Rules:**
- Current count only goes up — deleting a completion does not decrement.
- An orphaned criterion (target deleted) can never be satisfied — campaign becomes stuck. User's recourse is to duplicate the campaign without that criterion.
- Progress is tracked by `check_campaign_progress`, called from the frontend after every quest or saga completion across all five completion paths (quest list, quest giver, timer, saga tab, overlay).

### Accomplishment
A permanent record of completing a campaign. Created when a campaign completes.

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| campaign_id | UUID? (text) | FK to Campaign. NULL if campaign deleted. |
| campaign_name | String | Snapshot of campaign name at completion time |
| completed_at | Timestamp | ISO 8601 |
| bonus_xp | Integer | XP awarded (0 until Phase 2G.2-3 wires bonus calculation) |

**Rules:**
- Parallels how completions relate to quests — the accomplishment survives campaign deletion.
- Individually deletable. Deleting an accomplishment does NOT reduce XP.
- Table exists but accomplishment records are not yet created (Phase 2G.2-3).

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

### Tag (Category)
A user-defined label for organizing quests by context (e.g., "Computer", "Outside", "Phone").

| Field | Type | Description |
|---|---|---|
| id | UUID (text) | Unique identifier |
| name | String | Tag display name (unique) |
| sort_order | Integer | Display/creation order |

**Rules:**
- Tags are created inline when first applied to a quest.
- Deleting a tag removes all quest-tag links.
- Tags are searchable via the fuzzy search field on the quest list.

### Quest-Tag Link (quest_tag)
Many-to-many join table between quests and tags.

| Field | Type | Description |
|---|---|---|
| quest_id | UUID (text) | FK to Quest |
| tag_id | UUID (text) | FK to Tag |

## Relationships

```
Quest (1) ──── has many ──── Completion
Quest (M) ──── many-to-many ──── Skill       (via quest_skill)
Quest (M) ──── many-to-many ──── Attribute   (via quest_attribute)
Quest (M) ──── many-to-many ──── Tag          (via quest_tag)
Saga (1) ──── has many ──── Quest            (via quest.saga_id)
Campaign (1) ──── has many ──── Campaign Criterion
Campaign (1) ──── has many ──── Accomplishment
Attribute (1) ──── has many ──── Skill
Character (singleton)
```

Completions reference quests via `quest_id`, but the FK is nullable — completions
survive quest deletion. Deleting a quest also cleans up its link rows in quest_skill
and quest_attribute. Deleting a saga deletes its steps and orphans their completions.
Deleting a campaign deletes its criteria and orphans its accomplishments.

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

### Not Today (not_today)
Tracks quests dismissed from the quest giver for the current day.

| Field | Type | Description |
|---|---|---|
| quest_id | TEXT PRIMARY KEY | FK to Quest (the dismissed quest) |
| dismissed_date | TEXT | Local date (YYYY-MM-DD) when dismissed |

**Rules:**
- One row per dismissed quest. Stale rows (date before today) are cleaned up on app startup.
- Dismissed quests are excluded from the quest giver candidate pool but remain visible on the quest list with ⏾ icon and cooldown styling.
- Dismissing a saga step effectively dismisses the saga for the day (next step can't activate until current step completes).
- Action buttons (⚔ ✓) remain available on the quest list — completing a dismissed quest works normally.
- Does not count as a skip and does not affect scoring.

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

### Lanes

The quest giver has three lanes, each filtering by difficulty:

| Lane | Name | Difficulties |
|---|---|---|
| 1 | Castle Duties | Trivial |
| 2 | Adventures | Easy |
| 3 | Royal Quests | Moderate, Challenging, Epic |

Saga steps appear in the lane matching their saga's hardest step difficulty. Each lane has its own quest giver images and flavor text.

### Candidate Pool (per lane)
1. All active quests matching the lane's difficulty filter + active saga steps whose saga matches the lane
2. Hard-filter: time-of-day bitmask matches current local hour (Morning 4am–noon, Afternoon noon–5pm, Evening 5pm–9pm, Night 9pm–4am)
3. Hard-filter: days-of-week bitmask includes today
4. Split into **due** and **not-due** pools; due always preferred. Saga steps go in the due pool when present (they only appear when their saga has an active run — either the saga is due, or the user started a new run early).

### Scoring
```
score = overdue_ratio + importance_boost + list_order_bonus + membership_bonus + balance_bonus - skip_penalty
```

All factors apply to both due and not-due pools (not-due uses normalized days_since instead of overdue_ratio).

- **Overdue ratio**: `(days_overdue + cycle) / cycle` for recurring, `(days + 9) / 9` for one-off. Saga steps use their saga's cycle_days (one-off sagas fall back to 9).
- **Importance boost**: `importance × 30.0 / (1 + skips)`. Importance (0–5) is the dominant scoring signal. Each level ≈ 30 days of daily overdue. Skips diminish importance gracefully rather than subtracting — after many skips, approaches a 0! quest.
- **List order bonus**: `sort_order / global_max_sort_order` (max 1.0). `global_max_sort_order` is the max across both quest and saga sort_orders. Saga steps use their parent saga's sort_order — same formula as regular quests. Position on the quest list is the priority.
- **Membership bonus**: +1.0 for quests or saga steps referenced in any active campaign. Regular quests match on `quest_completions` criteria; saga steps match on `saga_completions` criteria targeting their parent saga. Boolean — does not stack across multiple campaigns.
- **Balance bonus**: `0.5 × max(0, avg_level - linked_level)` per linked attribute/skill, take the max. Gently nudges quests feeding underleveled attributes/skills.
- **Skip penalty**: `skip_count × 0.5`. Base penalty for non-important quests. For important quests, skips are handled by dividing importance (see above). Resets daily. Recorded on "Something Else" (main) and "Run" (overlay). "Hide in the Shadows" does not count.

### Behavior
- "Something Else" / "Run": records a skip, then re-scores. The just-skipped quest is excluded from the next pick (unless it's the only quest).
- Exhaustion fallback: if all scores ≤ 0, returns the least-negative.
- Skip counts are in-memory, reset at local midnight or app restart.
- The Encounters overlay shows only Lane 1 (trivial) quests and syncs with the quest giver. Both exclude the last-skipped quest so they show the same quest. The last-skipped ID is stored in skip state, set on "Something Else" / "Run", cleared on completion.

## Planned Entities (Phase 2+)

| Entity | Phase | Purpose |
|---|---|---|
| Badge | 2 | Discrete achievements |
