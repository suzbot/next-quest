# Phase 1: "The Quest Giver" — Requirements

## Goal

Turn the app from a quest list into a quest giver. Instead of looking at a list
and deciding, the app picks a quest, interrupts you, and says "do this now."
The list and character views remain for management, but the primary interaction
becomes: get interrupted → see one quest → act on it or skip it.

## Core Concepts

- **Next Quest View** = the quest giver face. Shows one quest at a time with
  actions to take. This is what the app opens to, and what you land on when
  a Call to Adventure brings you back.
- **Quest Selection** = the logic that picks which quest to show. Simple and
  deterministic for now: due quests in list order, with fallbacks.
- **Overlay Call to Adventure** = an always-on-top visual notification that appears
  over whatever you're doing. A knock on the door — not a notification center
  entry you can ignore.
- **Break Timer** = a simple timer that runs while you're away doing a quest.
  No XP implications yet — just tracks how long you were gone.
- **Menu Bar Presence** = the app lives in the Mac menu bar. Dropdown shows
  the current quest and quick actions. Full window opens for list/character
  management.

---

## Quest Selection Logic

The app picks quests in this order:

1. **Due quests in list order.** A quest is "due" if its cycle has elapsed since
   last completion, measured in calendar days resetting at local midnight.
   One-off quests that haven't been completed are always due. Among due quests,
   selection follows `sort_order` (the player-defined list sequence).

2. **Fallback: longest-ago-completed.** If no quests are due, suggest the
   recurring quest that was completed longest ago, regardless of cycle.

3. **Wrap around.** When the user cycles through all options via "Something Else,"
   the list wraps back to the beginning.

### Local Midnight Reset

Quest cycles currently use UTC. This phase switches to local time so that a
daily quest completed at 11pm is due again after midnight, not after 24 elapsed
hours. "Due" means: the number of calendar days (in local time) since last
completion >= `cycle_days`.

---

## Next Quest View

The primary view of the app. Shows:

- The selected quest name
- **Done** — marks the quest complete, awards XP, stops break timer if running
- **Quest Now** — starts the break timer, user goes AFK to do the quest
- **Something Else** — skips to the next quest in the selection order

When all quests have been cycled through, it wraps back to the first.

When a break timer is active (Quest Now was pressed), the view shows:
- The quest in progress
- Running timer
- **Done** — stops timer, completes quest, awards XP
- **Cancel** — stops timer, no completion, returns to quest selection

---

## Quest Now from List View

Any quest in the list view can be manually triggered into the Quest Now flow.
This bypasses selection — the user is choosing a specific quest. The same
break timer and Done/Cancel flow applies.

---

## Menu Bar

The app lives in the Mac menu bar (system tray). The tray menu is minimal —
quest actions stay in the main window where they have proper UI.

**Tray menu shows:**
- Call to Adventure on/off toggle
- Open Next Quest (shows/focuses the main window)
- Quit

**Close-to-tray:** Closing the main window hides it instead of quitting.
The app continues running in the tray so the Call to Adventure overlay
can still reach the user.

**Full window** (opens from tray or overlay click):
- Quest Giver view (default)
- Quest list view
- Character view
- App settings (Call to Adventure interval)

---

## Overlay Call to Adventure

When the app determines a quest is available, it can interrupt the user with
an always-on-top overlay — a visual knock on the door that appears over
whatever the user is doing, similar to peon-ping's overlay system.

**Overlay behavior:**
- Appears over current work on all screens
- Shows a simple prompt (no quest details on the overlay itself)
- **Click** — brings the user to the Next Quest view
- **Maybe Later** — dismisses the overlay and restarts the polling interval

**Polling:**
- Configurable interval, default 20 minutes
- Timer resets on Maybe Later
- Toggle on/off from the menu bar dropdown
- Interval configurable within the app settings

---

## Break Timer

A simple elapsed-time timer.

- **Starts** when the user hits Quest Now (from Next Quest view or list view)
- **Stops + completes** when the user hits Done
- **Stops + abandons** when the user hits Cancel
- Visible in the Next Quest view and the menu bar dropdown
- No XP implications — just a clock for now

---

## What This Phase Does NOT Include

| Deferred Item | Planned For |
|---|---|
| XP for time away (break timer earning XP) | Phase 2+ |
| Smart quest selection (energy, time of day, difficulty weighting) | Phase 2 |
| Quest details on the overlay itself | Later if needed |
| Overlay sound/audio | Separate from peon-ping |
| Snooze duration configuration per-overlay | Later if needed |
| Multiple quest givers | Phase 2 |
| Training / Patrolling / Battling modes | Phase 3 |

---

## Implementation Grouping

These requirements cluster into three groups, roughly in dependency and
priority order. Exact step breakdown will happen in design/step specs.

### Group A: Quest Giver Core
The new interaction model. Can be built and used within the existing window
before any menu bar or Call to Adventure work.

- Quest selection logic (due detection, list ordering, fallback, wrap-around)
- Local midnight reset for cycle due dates
- Next Quest view (Done / Quest Now / Something Else)
- Break timer (start, stop, cancel)
- Quest Now from list view
- Consolidate duplicate link-loading code (tech debt, do early to keep db.rs clean)

### Group B: Menu Bar
Moving the app's presence to the Mac menu bar. Depends on Group A being
functional.

- System tray icon with native menu
- Call to Adventure toggle in tray menu
- Open Next Quest from tray menu
- Close-to-tray lifecycle (close hides, Quit exits)
- Timer state and quest selection moved to backend (shared architecture)

### Group C: Overlay Call to Adventure
The interrupt system that reaches out to the user. Depends on Group B for
the menu bar toggle, but could be built in parallel with a temporary toggle.

- Overlay rendering (always-on-top, all screens)
- Click-to-focus (brings user to Next Quest view)
- Maybe Later button (dismiss + restart interval)
- Configurable polling interval (default 20 min)
- Settings UI for interval configuration
