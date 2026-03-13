# Step Spec: Phase 0, Step 1 — "Add and See Quests"

## Goal

Add quests and see them in a list. The most basic proof that the app
stores and displays data.

## Scope

**Backend:**
- Initialize SQLite database on app launch (create file + tables if they don't exist)
- Implement `add_quest` command (title, cycle_days) → creates quest, returns it
- Implement `get_quests` command → returns all active quests ordered by sort_order desc

**Frontend:**
- Quest list: renders quests returned by `get_quests`
- Each quest shows: title, cycle ("Every N days" or "One-off")
- Add quest form: text input for title, input for cycle_days, submit button
- Cycle defaults to recurring / 1 day. User can set to 0 or blank for one-off.
- On add: calls `add_quest`, re-fetches and re-renders list
- Keyboard: Tab between form fields and submit. Enter to submit.
- All text rendered in a monospace font.

## NOT in this step

- Mark done / completion tracking
- Edit quest
- Delete quest
- Reorder / drag-and-drop
- Visual states (due/de-emphasized/strikethrough)
- Last completed display

## Done When

- App launches and shows an empty list
- You can type a quest name, set a cycle, and hit submit
- The quest appears in the list
- Quests survive closing and reopening the app (persisted to SQLite)

## Next Step Preview

Step 2: "Complete Quests" — mark done button, completion records, last-done
display, visual states.
