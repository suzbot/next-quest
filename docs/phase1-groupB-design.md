# Phase 1 Group B: "Menu Bar" — Design

Covers: system tray icon, native dropdown menu, timer state in backend,
close-to-tray lifecycle, and shared state between tray and window.

## System Tray (Native Menu)

Tauri 2 has built-in tray support via `TrayIconBuilder`. We use a **native OS
menu** — text items, separators, checkboxes. No custom dropdown window for now.

### Tray Menu Layout

```
Take a shower                   ← current quest name (disabled, just a label)
───────────────
Done
Quest Now
Something Else
───────────────
✓ Call to Adventure              ← checkbox toggle
───────────────
Open Next Quest                  ← shows/focuses main window
Quit
```

When a timer is running, the menu changes:

```
Questing: Take a shower         ← quest name with prefix
Timer: 03:42                    ← elapsed time (disabled label)
───────────────
Done
Cancel
───────────────
✓ Call to Adventure
───────────────
Open Next Quest
Quit
```

### Tray Icon

Use the app's default window icon initially. On macOS, set `icon_as_template`
so it adapts to dark/light mode. A custom tray-specific icon can come later.

### Menu Rebuild

The tray menu is **rebuilt** whenever state changes: quest completed, timer
started/stopped, Something Else pressed, Call to Adventure toggled. Tauri's
`TrayIcon::set_menu()` replaces the menu. This is cheap for native menus.

## Timer State Moves to Backend

Currently the break timer is pure frontend JS. With both the tray menu and
the main window needing timer state, it moves to Rust.

### New Struct: `TimerState`

```rust
pub struct TimerState {
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,  // Unix millis
}
```

Stored in a `Mutex<TimerState>` managed by Tauri alongside `DbState`.

### New Commands

| Command | What it does |
|---|---|
| `start_timer(quest_id)` | Looks up quest title, sets timer state |
| `cancel_timer()` | Clears timer state |
| `complete_timer()` | Calls `complete_quest`, clears timer, returns completion |
| `get_timer_state()` | Returns current timer state for display |

`complete_timer` is a convenience that combines "stop timer + complete quest"
in one call, avoiding race conditions between tray and window.

### Frontend Changes

The frontend timer logic simplifies. Instead of owning the timer state,
it calls `get_timer_state()` and uses `setInterval` only for display updates.

- `startTimer()` → calls `invoke("start_timer", { questId })`
- `cancelTimer()` → calls `invoke("cancel_timer")`
- `timerDone()` → calls `invoke("complete_timer")`
- Display tick: `setInterval` calls `get_timer_state()` or computes
  elapsed locally from the known `started_at`

The simpler approach: frontend stores `started_at` locally after calling
`start_timer`, computes elapsed from `Date.now() - startedAt`. Only calls
backend to start/stop/complete. This avoids polling the backend every second.

## App Lifecycle

### Close-to-Tray

Closing the main window **hides** it instead of quitting the app.
Implemented via `on_window_event` intercepting `CloseRequested`:

```rust
.on_window_event(|window, event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
        window.hide().unwrap();
        api.prevent_close();
    }
})
```

### Quit

"Quit" in the tray menu calls `app.exit(0)`. Cmd+Q also quits.

### Open Full App

"Open Next Quest" in the tray menu shows the main window and brings it
to focus. If already visible, just focuses it.

```rust
if let Some(window) = app.get_webview_window("main") {
    let _ = window.show();
    let _ = window.set_focus();
}
```

## Tray ↔ Window Sync

When the tray menu performs an action (Done, Quest Now, Something Else),
the main window needs to know. Two mechanisms:

1. **Shared backend state** — timer state lives in Rust. Both surfaces
   read from the same source of truth.
2. **Tauri events** — after a tray action, emit an event that the frontend
   listens for to trigger a `loadAll()` refresh.

```rust
// In tray menu handler, after completing a quest:
app.emit("quest-state-changed", ()).ok();
```

```javascript
// In frontend:
window.__TAURI__.event.listen("quest-state-changed", () => loadAll());
```

## Quest Selection in the Tray

The tray menu needs to know which quest to show. Options:

1. **Backend selection** — add a `get_next_quest()` command that returns
   the selected quest. The tray handler calls this to build the menu.
2. **In-memory cache** — tray handler reads from a shared selection state.

Recommend **option 1**: add `get_next_quest(skip_count)` that applies the
same selection logic (due quests in sort order, fallback to longest-ago).
The `skip_count` parameter handles Something Else (0 = first due quest,
1 = second, etc., wrapping around). This keeps selection logic in one place
rather than duplicating it in JS and Rust.

This means the frontend Quest Giver view also switches to calling
`get_next_quest` instead of doing selection in JS. One source of truth.

## Cargo Changes

Add the `tray-icon` feature to the tauri dependency:

```toml
tauri = { version = "2", features = ["tray-icon"] }
```

No new crates needed.

## What's NOT Changing

- **Data model** — no schema changes
- **XP engine** — no changes
- **Quest list / character views** — unchanged

## Implementation Order

1. **Timer state to backend + quest selection command** — new Rust state
   and commands, frontend migrates to use them. No tray yet. Tests.
2. **System tray + close-to-tray** — tray icon, native menu, menu actions,
   close-to-tray lifecycle. Event sync between tray and window.
