# Step Spec: Migrate Completion Snapshots from Names to IDs

Parent design: [xp-from-history-design.md](design/xp-from-history-design.md) — Step 1

## Goal

Change the `skills`, `attributes`, and `tags` columns in `quest_completion` from JSON arrays of entity names to JSON arrays of entity IDs. This makes the completion data rename-proof, which is a prerequisite for deriving XP totals from history.

The history UI should continue to display entity names (and colors) exactly as it does now. The change is invisible to the user.

## What changes

### 1. Migration: Convert existing name arrays to ID arrays

In `migrate()`, after the existing snapshot migrations:

1. Detect whether migration is needed using the established pattern (check for a new marker column, e.g., `SELECT snapshot_version FROM quest_completion LIMIT 0`).
2. For each completion with non-NULL skills/attributes/tags JSON:
   - Parse the JSON array of names
   - Look up each name in the current skill/attribute/tag table
   - Replace with the entity's ID
   - Unresolvable names (deleted/renamed entities) are dropped from the array
3. Update the row with the new ID arrays

Tags cannot be renamed so they're always resolvable (unless deleted). Skills and attributes can be renamed, so some old names may not resolve — these are dropped.

### 2. Update snapshot code to store IDs

Four code paths create completion records with skill/attribute/tag data:

1. **`complete_quest`** (db.rs ~2717-2740) — Snapshots linked skill names, attribute names, tag names. Change to snapshot IDs instead.
2. **Saga completion bonus** (`check_saga_completion`, db.rs ~1495-1510) — Snapshots last step's linked skill/attribute names. Change to snapshot IDs.
3. **Skill level-up bonus** (`award_xp`, db.rs ~2660-2673) — Snapshots attribute name. Change to snapshot attribute ID.
4. **Campaign completion bonus** (`check_campaign_progress`, db.rs ~2083) — No skill/attribute snapshots (only awards to character). No change needed.

### 3. Update get_completions to resolve IDs to names

`get_completions` currently returns the JSON arrays as-is (parsed from name strings). After migration, the arrays contain IDs.

Resolve IDs to names before returning:
- Load all skills, attributes, and tags into maps (id -> name) once at the top of the function
- For each completion, map the ID arrays to name arrays
- Deleted entities resolve to "Unknown"
- The `Completion` struct stays unchanged — it still has `Option<Vec<String>>` for display names

The frontend and `renderCompletions` don't change at all.

## Out of scope

- Derived XP queries (step 2 of the design)
- Deleting or excluding pre-migration completions (step 3)
- Changing the Completion struct to expose IDs to the frontend (not needed yet)

## Test plan

1. `cargo test` — all existing tests pass
2. Build and launch app — history UI displays the same entity names and colors as before
3. Complete a quest with linked skills/attributes/tags — new completion shows correct names in history
4. Verify via CLI (`nq list history | head`) that the raw JSON now contains UUIDs, not names
5. Rename a skill or attribute, complete a quest — old completions still show the old entity's current name (not "Unknown")
