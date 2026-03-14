# Step Spec: Phase 1, Step 5 — "Call to Adventure"

## Goal

The app reaches out and interrupts you. When Call to Adventure is on and a
quest is due, an overlay pops up over your work after a configurable interval.
Click it to go to the quest, or hit Maybe Later to snooze. A new Settings
tab holds the interval config and the reset buttons (moved from Character).

Split into two sub-steps with a testing checkpoint after each.

---

## Step 5a: Settings Tab + Polling Infrastructure

### Settings Tab

New nav tab: `[Quest Giver] [Quests] [Character] [Settings]`

**Settings view contains:**
- **Call to Adventure interval**: number input, in minutes, default 20.
  Saved via a new backend command. Label: "Call to Adventure interval (minutes)"
- **Reset buttons** (moved from Character view):
  - [Reset Char] — zeroes all XP
  - [Reset Quests] — deletes all quests
  - [Reset History] — deletes all completions
  - Same two-click "Sure?" confirmation behavior

### Backend Changes

**TrayStateInner gains interval:**
```rust
pub struct TrayStateInner {
    pub call_to_adventure: bool,
    pub cta_interval_secs: u64,     // default 1200 (20 min)
}
```

**New commands:**
| Command | What it does |
|---|---|
| `get_settings()` | Returns Call to Adventure state and interval |
| `set_cta_interval(minutes)` | Updates the polling interval |

### Polling Thread

A background thread spawned in `.setup()` that runs the Call to Adventure
check loop.

**Rules:**
- Sleeps for the configured interval, then checks conditions
- Only proceeds when Call to Adventure is ON
- Does NOT proceed when a break timer is running
- Does NOT proceed when the main window is focused
- Resets the interval when:
  - Maybe Later is pressed (Step 5b)
  - A break timer stops (Done or Cancel)
  - Call to Adventure is toggled ON
- When toggled OFF, the thread skips on its next wake

**For Step 5a:** When conditions are met, the thread emits a
`"call-to-adventure"` Tauri event (logged in the console). The overlay
window is wired up in Step 5b.

### 5a Testing Checkpoint

- Settings tab appears with interval input (default 20)
- Changing the interval saves the value
- Reset buttons appear on Settings tab, removed from Character tab
- Reset buttons still work with "Sure?" confirmation
- Call to Adventure toggle in tray menu still works
- With Call to Adventure ON and a quest due: after the interval, a
  console event fires (visible in dev tools if running `cargo tauri dev`,
  otherwise just verify the thread runs without crashing)
- Event does NOT fire when timer is running or window is focused
- All existing tests pass, app builds and runs

---

## Step 5b: Overlay Window

### Overlay Window

A small Tauri window — borderless, always-on-top, centered on the main
screen. Appears over whatever the user is doing.

**Content:**
```
   A quest awaits...

      [Maybe Later]
```

- Clicking anywhere on the overlay (except Maybe Later) shows and focuses
  the main window on the Quest Giver tab, then closes the overlay.
- **Maybe Later** closes the overlay and restarts the polling interval.

**Window properties:**
- Borderless (no title bar)
- Always on top
- Dark background matching the app aesthetic
- Small and centered — roughly 300x120
- Not resizable, not minimizable

### overlay.html

A separate small HTML file for the overlay window content:

- `openApp()` — invokes `dismiss_overlay` with action `"open"`
- `maybeLater()` — invokes `dismiss_overlay` with action `"later"`,
  stops event propagation so it doesn't also trigger openApp

The backend handles the response: "open" shows the main window and
closes the overlay, "later" restarts the polling interval and closes
the overlay.

### Backend Changes

**New commands:**
| Command | What it does |
|---|---|
| `dismiss_overlay(action)` | Closes overlay. "open" shows main window, "later" restarts interval |

**Polling thread change:** Instead of emitting an event, the thread
creates/shows the overlay window when conditions are met. Does not
show a second overlay if one is already visible.

### 5b Testing Checkpoint

- With Call to Adventure ON and a quest due: after the interval, overlay
  appears over current work
- Clicking the overlay opens the main window on Quest Giver tab
- Clicking Maybe Later dismisses the overlay, interval restarts
- Overlay does NOT appear when timer is running
- Overlay does NOT appear when main window is focused
- Completing or cancelling a timer resets the interval
- Toggling Call to Adventure ON resets the interval
- All existing tests pass, app builds and runs

---

## NOT in this step

| Deferred Item | Planned For |
|---|---|
| Persistent settings (survive app restart) | Phase 1 Step 6 — small follow-up to store settings in SQLite |
| Overlay on all screens | Later if needed |
| Sound/audio on overlay | Separate from peon-ping |
| Quest details on the overlay | Later if needed |
