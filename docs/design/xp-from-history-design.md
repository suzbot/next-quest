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
WHERE completed_at >= ?1  -- snapshot cutoff date
```

### Attribute XP
```sql
SELECT COALESCE(SUM(c.xp_earned), 0)
FROM quest_completion c, json_each(c.attributes) j
WHERE j.value = ?1  -- attribute ID
AND c.completed_at >= ?2  -- snapshot cutoff date
```

### Skill XP
```sql
SELECT COALESCE(SUM(c.xp_earned), 0)
FROM quest_completion c, json_each(c.skills) j
WHERE j.value = ?1  -- skill ID
AND c.completed_at >= ?2  -- snapshot cutoff date
```

The cutoff date is derived dynamically: `SELECT MIN(completed_at) FROM quest_completion WHERE difficulty IS NOT NULL`. Everything before the cutoff is visible in history but excluded from XP totals — avoids the problem of bonus completions (saga/campaign/level-up) having NULL difficulty. Works correctly for any user regardless of when they started.

`json_each(NULL)` produces no rows, so post-cutoff completions with NULL skill/attribute arrays (like campaign bonuses that only award to character) naturally contribute 0 to attribute/skill sums.

## Implementation Steps

Each step is a vertical slice that can be tested independently.

### Step 1: Migrate snapshots to IDs

- Add migration to convert existing name arrays to ID arrays in quest_completion
- Update complete_quest to snapshot IDs
- Update saga/campaign/level-up bonus code to snapshot IDs
- Update get_completions to resolve IDs to names for display
- Update renderCompletions to work with the new data shape (if needed)
- **Test:** History UI still displays correctly with entity names, not UUIDs

### Step 2: Build derived XP audit (no UI changes)

- Add `audit_xp` function to db.rs that computes derived XP for every entity using the queries above
- Add CLI command: `nq audit xp` — outputs a comparison table:
  ```
  Entity          Current   Derived   Delta
  Character         4200      4200       0
  Health             850       720    -130
  Cooking            300       300       0
  ...
  ```
- **Test:** Run audit, examine deltas. Expected discrepancies:
  - Pre-migration completions inflate character XP (NULL skills/attributes)
  - Pre-migration skill level-up bonuses not recorded (attribute deltas)

### Step 3: Verify pre-migration exclusion

- Re-run audit to confirm derived totals only count post-migration completions
- Verify pre-migration completions still appear in history UI
- **Test:** Audit deltas are zero (or near-zero). History list still shows old entries.

### Step 4: Wire display to derived totals

Two systems, two roles:

**Derived totals (source of truth for display):**
- `get_character` computes character XP as `SUM(xp_earned)` from all completions
- `get_attributes` computes each attribute's XP as `SUM(xp_earned)` from completions containing that attribute ID
- `get_skills` computes each skill's XP as `SUM(xp_earned)` from completions containing that skill ID
- All levels, XP bars, and level-related display derive from these sums
- This is what the user sees everywhere: character tab, quest giver, balance bonus calculation

**Cached running totals (internal, for completion-time logic):**
- `award_xp` continues updating `character.xp`, `attribute.xp`, `skill.xp` directly
- These cached values are used **only** within the `complete_quest` transaction for:
  1. Skill level-up detection (compare before/after to decide whether to create a level-up bonus record)
  2. Balance bonus scoring in the quest selector (reads attribute/skill levels to nudge underleveled quests)
- The cache drifts from derived totals between recalculations — this is fine because it's only used for relative comparisons (did a level boundary get crossed? is this skill below average?), not absolute display

**Reconciliation:**
- Add a `recalculate_xp_cache` function that resets all entity XP to match derived totals
- Run once at migration time to align cache with history after pre-migration deletions
- Available as a repair tool (`nq recalculate`) if cache ever drifts meaningfully
- Could optionally run on app startup, but probably unnecessary — cache only drifts if completions are manually deleted, and even then the impact is limited to level-up detection timing

**What changes in the read path:**
- `get_character`: replaces `SELECT xp FROM character` with the SUM query
- `get_attributes`: replaces `SELECT xp FROM attribute` with per-attribute SUM queries
- `get_skills`: replaces `SELECT xp FROM skill` with per-skill SUM queries
- Level calculation (`level_from_xp`) is unchanged — it takes an XP value regardless of source

**What does NOT change:**
- `award_xp` — still updates cached totals and detects level-ups
- `complete_quest` — still calls award_xp, still creates completion records with snapshots
- Quest selector scoring — reads from cached totals (balance bonus), which is acceptable since exact precision isn't needed for relative scoring
- XP formula — `xp_earned` is stamped at completion time and never recalculated

**Test:**
- Levels and XP bars match the step 2 audit numbers
- Completing a quest still shows correct XP feedback and triggers level-up detection
- Deleting a completion now reduces displayed XP (intentional — history is truth)
- Balance bonus scoring still works (uses cache, doesn't need exact parity)

## Out of Scope

- Changing the XP formula itself (that's a separate tuning exercise)
- Retroactive formula changes (stamped xp_earned is preserved as-is)
- Removing the cached running totals entirely (keep as cache for now, evaluate later)
- Running recalculation automatically on startup (can add later if drift becomes a problem)
