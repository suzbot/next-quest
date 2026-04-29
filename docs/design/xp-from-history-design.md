# Tech Design: Derive XP from Completion History

## Goal

Switch XP display to derive entity totals from completion history instead of relying on running counters. This aligns the app's display with the analysis nq-companion already does, and makes completion history the single source of truth for XP.

## Approach

Keep running totals on entities as a performance cache (for level-up detection during completion), but derive the displayed XP from history. Build and validate the derived queries before wiring them in.

## Prerequisites

### 1. Migrate completion snapshots from names to IDs

Currently `skills`, `attributes`, and `tags` columns in `quest_completion` store JSON arrays of **names** (e.g., `["Cooking", "Health"]`). This breaks if entities are renamed — old completions reference the old name and wouldn't match the current entity.

**Change:** Store JSON arrays of entity **IDs** instead. Look up display names at query time.

**Migration:**
1. For each completion with non-NULL skills/attributes/tags JSON, parse the name array, look up each name in the current entity tables, and replace with the corresponding ID.
2. Names that can't be resolved (deleted entities) are dropped from the array.
3. Update `complete_quest`, saga bonus, campaign bonus, and level-up bonus code to snapshot IDs instead of names.
4. Update `get_completions` to join through IDs to get display names, falling back to "Unknown" for deleted entities.

**Tradeoff:** Completions lose the "self-contained after deletion" property for linked entities. The title, XP, difficulty, and cycle are still self-contained. Deleted skill/attribute names show as "Unknown" in history. This is rare in practice.

### 2. Record all XP-affecting events as completions (done)

Already completed in prior commit:
- Skill level-up bonuses now create completion records with attribute ID snapshot
- Saga completion bonuses now snapshot last step's linked skills/attributes

Campaign completion bonuses only award to character (no skill/attribute distribution), so no change needed.

### 3. Pre-migration completions

Completions from before the skill/attribute snapshot migration have NULL for skills, attributes, tags, and difficulty. These can't participate in derived XP calculation — they'd inflate character XP relative to attribute/skill XP.

**Approach:** Keep them in history (visible in the UI, available for analysis) but exclude from XP totals. All derived SUM queries include `WHERE difficulty IS NOT NULL` — difficulty was added in the same migration as the other snapshot columns, so it's a reliable marker for "this completion has full data."

## Core Queries

### Character XP
```sql
SELECT COALESCE(SUM(xp_earned), 0) FROM quest_completion
WHERE character_id IS NOT NULL AND completed_at >= ?1  -- cutoff date
```

### Attribute XP
```sql
SELECT COALESCE(SUM(c.xp_earned), 0)
FROM quest_completion c, json_each(c.attributes) j
WHERE j.value = ?1  -- attribute ID
AND c.completed_at >= ?2  -- cutoff date
```

### Skill XP
```sql
SELECT COALESCE(SUM(c.xp_earned), 0)
FROM quest_completion c, json_each(c.skills) j
WHERE j.value = ?1  -- skill ID
AND c.completed_at >= ?2  -- cutoff date
```

**Cutoff date:** Derived dynamically as `SELECT MIN(completed_at) FROM quest_completion WHERE difficulty IS NOT NULL`. Falls back to `MIN(completed_at)` for new users with no pre-migration data. Everything before the cutoff is visible in history but excluded from XP totals.

**`character_id`:** Controls which completions count toward character XP and daily stats. Quest completions, saga bonuses, and campaign bonuses set it. Skill level-up bonuses leave it NULL (they only award attribute XP). This models the same behavior the user sees on screen: "+25 Character, +25 Cooking" — the completion lists its XP recipients.

`json_each(NULL)` produces no rows, so completions with NULL skill/attribute arrays naturally contribute 0 to those entity sums.

## Implementation Steps

Each step is a vertical slice that can be tested independently.

### Step 1: Migrate snapshots to IDs (done)

Completion records now store entity UUIDs instead of names. Migration converts existing name arrays on app launch. `get_completions` resolves IDs back to display names transparently.

### Step 2: Build derived XP audit + backfill (done)

- `nq audit-xp` — compares cached vs derived XP/levels for all entities
- `nq backfill-levelups` — replays post-cutoff completions per skill, detects level boundaries, inserts missing attribute bonus records. Idempotent.
- Remaining deltas after backfill are expected — they represent pre-cutoff XP that can't be attributed.

### Step 3: Verify pre-migration exclusion (done)

- Audit confirms derived totals only count post-cutoff completions
- Pre-migration completions remain visible in history UI
- Deltas are stable and explainable

### Step 4: Wire display to derived totals (done)

**Derived totals (source of truth for all display):**
- `get_character` computes character XP as `SUM(xp_earned) WHERE character_id IS NOT NULL`
- `get_attributes` computes each attribute's XP as `SUM(xp_earned)` from completions containing that attribute ID
- `get_skills` computes each skill's XP as `SUM(xp_earned)` from completions containing that skill ID
- `get_xp_stats` (best day, avg, today) uses the same `character_id IS NOT NULL` filter
- All levels, XP bars, and level-related display derive from these sums

**`character_id` column:** Each completion lists which entities receive its XP. Quest completions, saga bonuses, and campaign bonuses set `character_id` (the character's UUID). Skill level-up bonuses leave it NULL — they only award attribute XP. This keeps level-up bonuses from inflating character totals and daily stats.

**Completion flow (insert first, detect after):**

All XP-producing paths follow the same pattern:
1. Snapshot derived levels **before** (via `get_character`/`get_attributes`/`get_skills`)
2. Insert the completion record into history
3. Read derived levels **after** (history now includes the new record)
4. Compare before/after → detect level-ups
5. For each skill level-up, insert a level-up bonus completion record
6. Sync the cached XP on entities to match derived totals

This replaces the old `award_xp` approach which updated cached XP, detected level-ups from the cache, then inserted the completion record. The new flow is simpler: history is always written first, and everything reads from it.

Three functions follow this pattern:
- **`complete_quest`** — inserts quest completion, detects character/skill/attribute level-ups
- **`check_saga_completion`** — inserts saga bonus completion, detects level-ups
- **`check_campaign_progress`** — inserts campaign bonus completion, detects level-ups

**`award_xp` is removed.** Its responsibilities are split:
- Cache updates → replaced by `sync_xp_cache` (called at the end of each completion path)
- Level-up detection → moved to after completion record insertion, using derived totals
- Level-up bonus insertion → moved to the same post-insertion detection step

**Cache sync (`sync_xp_cache`):**
- After all completion records are inserted, update `character.xp`, `attribute.xp`, `skill.xp` to match derived totals
- This keeps the cache in sync for any code that reads entity tables directly
- Available as a standalone CLI tool (`nq recalculate`) for repair

**What changes in the read path:**
- `get_character`: derives XP from history SUM
- `get_attributes`: derives each attribute's XP from history SUM
- `get_skills`: derives each skill's XP from history SUM
- Level calculation (`level_from_xp`) is unchanged — it takes an XP value regardless of source
- Quest selector scoring uses derived totals (via `get_attributes`/`get_skills`)

**What does NOT change:**
- XP formula — `xp_earned` is stamped at completion time and never recalculated
- Completion record structure — same snapshots (skill/attribute/tag IDs, difficulty, cycle)
- `reset_character` — deletes all completions and zeroes the cache (single operation, single button)

**Test:**
- Levels and XP bars match the step 2 audit numbers
- Completing a quest still shows correct XP feedback and triggers level-up detection
- Deleting a completion reduces displayed XP (history is truth)
- Level-up bonuses add to attribute XP but not character XP or daily stats
- Balance bonus scoring works from derived totals

## Out of Scope

- Changing the XP formula itself (that's a separate tuning exercise)
- Retroactive formula changes (stamped xp_earned is preserved as-is)
- Running cache sync automatically on startup (can add later if needed)
