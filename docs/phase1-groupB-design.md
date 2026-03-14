# Phase 1 Group B: "Menu Bar" — Design

Covers: system tray icon, close-to-tray lifecycle, and Call to Adventure
toggle. Quest actions stay in the main window for now.

## System Tray (Native Menu)

Tauri 2 has built-in tray support via `TrayIconBuilder`. The tray menu is
minimal — just the Call to Adventure toggle, a way to open the window, and
quit. Quest actions (Done, Quest Now, Something Else) stay in the main
window where they have proper UI.

### Tray Menu Layout

```
Call to Adventure: ON/OFF       ← text toggle
───────────────
Open Next Quest                 ← shows/focuses main window
Quit                            ← exit the app
```

Menu rebuilds when Call to Adventure is toggled.

### Tray Icon

Uses the app's default window icon with `icon_as_template(true)` for
macOS dark/light mode adaptation. A custom tray-specific icon can come
later (Phase 2 — graphics).

## Timer State in Backend

Timer and quest selection state moved to the Rust backend in Step 3
so both the tray and the main window share one source of truth. This
was done in anticipation of tray quest actions, but remains useful
architecture for the Call to Adventure overlay (Group C) which will
need to read quest state from the backend.

### Commands (added in Step 3)

| Command | What it does |
|---|---|
| `start_timer(quest_id)` | Looks up quest title, sets timer state |
| `cancel_timer()` | Clears timer state |
| `complete_timer()` | Calls `complete_quest`, clears timer, returns completion |
| `get_timer_state()` | Returns current timer state for display |
| `get_next_quest(skip_count)` | Returns the next due quest using selection logic |

## App Lifecycle

### Close-to-Tray

Closing the main window **hides** it instead of quitting the app.
The app continues running in the tray so the Call to Adventure overlay
(Group C) can still reach the user.

### Quit

"Quit" in the tray menu calls `app.exit(0)`. Cmd+Q also quits.

### Open Next Quest

"Open Next Quest" in the tray menu shows the main window and brings it
to focus. If already visible, just focuses it.

## Cargo Changes

Added `tray-icon` feature to the tauri dependency:

```toml
tauri = { version = "2", features = ["tray-icon"] }
```

## What Was Deferred

Quest actions in the tray menu (Done, Quest Now, Something Else) were
originally planned but removed after evaluation. Native macOS menus are
too limited for a good quest interaction experience — the main window
is the right surface for that. If this is revisited, a custom dropdown
window (not native menu) would be the approach.
