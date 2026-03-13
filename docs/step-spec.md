# Step Spec: Phase 1, Step 2 — "Quest Now & Break Timer"

## Goal

Add the ability to commit to a quest and track time away. Pressing Quest Now
starts a timer and puts the app in "go do it" mode. Done completes the quest,
Cancel abandons it. Navigating away from the Quest Giver tab cancels the timer —
the app is pushing you away from the screen, not inviting you to stay.

## Scope

### Quest Giver View: Quest Now Button

Add a `[Quest Now]` button to the selection mode, between Done and Something Else:

```
      Take a shower

  [Done]  [Quest Now]  [Something Else]
```

### Quest Giver View: Timer Mode

When Quest Now is pressed, the view switches to timer mode:

```
         QUEST GIVER

    Take a shower

        03:42

    [Done]  [Cancel]
```

- Timer displays elapsed time as MM:SS (advances to H:MM:SS after 59:59)
- **Done** — stops timer, completes quest, awards XP, shows XP flash,
  returns to selection mode
- **Cancel** — stops timer, no completion, returns to selection mode

### Navigating Away Cancels Timer

Clicking [Quests] or [Character] while a timer is running silently cancels
the timer and returns the Quest Giver to selection mode. No confirmation
prompt — the cost of an accidental cancel is trivially recoverable (just
press Quest Now again).

### Quest Now from List View

Add a `[Quest Now]` button to each active quest row in the list view.

Clicking it:
1. Sets the timer quest to that quest
2. Starts the timer
3. Switches to the Quest Giver tab in timer mode

### Timer Implementation

Pure frontend. No backend changes, no persistence.

```javascript
let timerQuestId = null;      // quest being worked on (null = selection mode)
let timerQuestTitle = null;   // cached title for display
let timerStart = null;        // Date.now() when Quest Now pressed
let timerInterval = null;     // setInterval handle
```

- `setInterval` at 1-second resolution
- Closing the app loses the timer (acceptable for now)

### Timer Display Format

```javascript
function formatTimer(ms) {
  const totalSecs = Math.floor(ms / 1000);
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = totalSecs % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}
```

## NOT in this step

- Timer persistence across app restarts
- XP for time away
- Menu bar / system tray (Group B)
- Overlay interruption (Group C)

## Done When

- Quest Now button appears in Quest Giver selection mode
- Pressing Quest Now switches to timer mode with running clock
- Done stops timer, completes quest, shows XP, returns to selection
- Cancel stops timer, returns to selection without completing
- Switching tabs while timer is running cancels the timer
- Quest Now button appears on each active quest in the list view
- Pressing Quest Now in the list switches to Quest Giver tab in timer mode
- All existing tests pass, app builds and runs
