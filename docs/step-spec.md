# Step Spec: Phase 0, Step 2 — "Complete Quests"

## Goal

Mark quests as done, see when they were last completed, and see visual
distinction between quests that are due vs. recently completed vs. finished.

## Scope

**Backend:**
- Implement `complete_quest` command (quest_id) → creates a quest_completion record, returns updated quest
- If the quest is one-off (cycle_days is null), set `active = 0` on completion
- Update `get_quests` to return completed one-offs (active = 0) alongside active quests
- Update `get_quests` to include `last_completed` (most recent completion timestamp) and `is_due` (computed from cycle) in the returned data

**Frontend:**
- Add a "Done" button (or key) on each quest
- Display last completed date/time per quest (if ever completed)
- Three visual states via text styling:
  - **Due/refreshed**: normal/emphasized text — cycle elapsed or never completed
  - **Recently completed**: muted/de-emphasized text — recurring quest, cycle not yet elapsed
  - **Completed one-off**: strikethrough text — stays visible as accomplishment
- On completing a recurring quest: it moves to the bottom of the list
- Keyboard: Enter or a key on a focused quest marks it done

## NOT in this step

- Edit quest
- Delete quest
- Reorder / drag-and-drop
- Keyboard navigation between quests (Tab through list items — Step 4)

## Done When

- You can mark a quest as done and see the "Last Done" timestamp update
- Completing a recurring quest de-emphasizes it and moves it to the bottom
- Completing a one-off quest shows it with strikethrough
- A recurring quest returns to emphasized style after its cycle elapses
- Completing a quest multiple times in a day works and updates the timestamp each time
- Tests cover: completion recording, one-off deactivation, is_due calculation, last_completed derivation

## Next Step Preview

Step 3: "Edit and Delete" — inline editing of quest properties, delete with confirmation.
