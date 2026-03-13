# Step Spec: Phase 0, Step 3 — "Completion Redesign + Edit/Delete"

## Goal

Completions become visible, independent records. Quests can be edited and
deleted. Schema migrated to support the new model.

## Scope

**Schema migration:**
- Add `quest_title` (TEXT NOT NULL) to `quest_completion` — snapshot of title at completion time
- Make `quest_id` nullable on `quest_completion` — quest may be deleted while completions remain
- Backfill `quest_title` on existing completion records from their linked quest
- Migration runs automatically on app launch (detect missing column, alter table)

**Backend:**
- Update `complete_quest` to snapshot `quest_title` into the completion record
- Implement `get_completions` → returns all completions, ordered by completed_at descending
- Implement `delete_completion` (completion_id) → deletes a single completion record
- Update `delete_quest` → deletes the quest row, sets `quest_id = NULL` on its completions (does NOT delete them)
- Fix `update_quest` to handle cycle_days = 0 as one-off (set to NULL)

**Frontend:**
- Split list into two sections:
  - **Active quests** (top): ordered by sort_order desc. Includes deactivated one-offs (greyed out).
  - **Completion history** (bottom): reverse chronological. Strikethrough. Each row shows quest title, timestamp, [Del].
- Active quest visual states:
  - Due/refreshed: emphasized
  - Recently completed (recurring): de-emphasized
  - Deactivated one-off: greyed out, no [Done] button
- Edit mode:
  - Click title OR press E while quest row is focused to enter edit mode
  - Tab to focus quest rows (tabindex on list items)
  - Enter saves, Escape cancels
- Delete quest: [Del] button, confirmation prompt. Available on all quests including deactivated one-offs.
- Delete completion: [Del] button on completion rows, no confirmation needed (low stakes).

## NOT in this step

- Reorder / drag-and-drop (Step 4)
- Full keyboard navigation between list items with arrow keys (Step 4)

## Done When

- Completions appear as their own rows in a history section below active quests
- Completing a quest creates a visible completion row with title and timestamp
- Deleting a quest leaves its completions in the history
- Individual completions can be deleted
- Editing a quest title, cycle works (including setting cycle to 0 for one-off)
- Edit mode can be entered with keyboard (E key on focused row)
- Existing data is migrated on app launch (no data loss)
- Tests cover: schema migration, completion with title snapshot, delete quest preserves completions, delete completion, update cycle to one-off

## Next Step Preview

Step 4: "Reorder Quests" — keyboard and drag-and-drop resequencing for active quests only.
