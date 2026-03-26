# Step Spec: Phase 3-4 — Skip exclusion fix + overlay sync ✅

## Goal

Overlay shows the same quest as Lane 1 (Castle Duties). After a skip in Lane 1, neither the quest giver nor the overlay shows the just-skipped quest. Replace `offered_quest` with `last_skipped_id` in the skip state.

---

## Substep 1: Backend — rename offered_quest to last_skipped

**`SkipStateInner`** in commands.rs: rename `offered_quest_id: Option<String>` to `last_skipped_id: Option<String>`.

**Rename commands:**
- `set_offered_quest` → `set_last_skipped` — accepts `quest_id: Option<String>`, stores it
- `get_offered_quest` → `get_last_skipped` — returns the stored ID

Update `invoke_handler` registration in main.rs.

---

## Substep 2: Frontend — quest giver stores last-skipped on skip

**`laneSomethingElse`**: After calling `skip_quest`, call `set_last_skipped` with the skipped quest ID. (Currently calls `set_offered_quest` with the offered quest — wrong direction.)

**`renderLane`**: Remove the `set_offered_quest` call. The offered quest is no longer tracked — only the skipped quest matters.

**Clear on completion**: After `laneDone` completes and calls `loadAll`, call `set_last_skipped` with null (no stale exclude). Similarly after `timerDone`.

---

## Substep 3: Frontend — overlay reads last-skipped

**Overlay `loadQuest`**: Replace `get_offered_quest` → `get_last_skipped`. Pass the result as `excludeQuestId` to `get_next_quest`. This means:
- No skip: `last_skipped` is null, no exclude, overlay gets same top quest as Lane 1
- After skip A: `last_skipped` is A, overlay excludes A, gets B (same as Lane 1)

**Overlay "Run" (somethingElse)**: Also calls `set_last_skipped` with the skipped quest ID so the main quest giver stays in sync if it re-renders.

---

## Testing checkpoint

Build app. Lane 1 shows quest A. Skip A → Lane 1 shows B. Wait for overlay → overlay also shows B (not A). Complete B → both move to next quest. Skip in Lane 2 → overlay unaffected (only Lane 1 skips matter).

---

## NOT in this step

Nothing — this completes Phase 3.

## Done When

Overlay and Lane 1 show the same quest. Skipping in Lane 1 excludes the skipped quest from both. `set_last_skipped`/`get_last_skipped` replace the old offered_quest commands. `cargo test` passes.
