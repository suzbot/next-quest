# Step Spec: Phase 1, Step 4 — "System Tray & Close-to-Tray"

## Goal

The app lives in the Mac menu bar. A native tray menu lets you see your
current quest and act on it without opening the full window. Closing the
window hides it instead of quitting — the app stays running in the tray.

## Scope

### Cargo Feature

Add `tray-icon` to the tauri dependency:

```toml
tauri = { version = "2", features = ["tray-icon"] }
```

### Tray Icon

Created in `main.rs` via `TrayIconBuilder` inside `.setup()`. Uses the
app's default window icon. On macOS, set `icon_as_template(true)` so it
adapts to dark/light menu bar.

### Native Tray Menu — Selection Mode

```
Take a shower                    ← quest name (disabled label)
─────────────
Done
Quest Now
Something Else
─────────────
✓ Call to Adventure              ← checkbox, toggles on/off
─────────────
Open Next Quest                  ← show/focus main window
Quit                             ← exit the app
```

If no quests are due: the quest name item shows "All caught up" (disabled).
If no quests exist: shows "No quests yet" (disabled).

### Native Tray Menu — Timer Mode

```
Questing: Take a shower          ← quest name with prefix (disabled label)
─────────────
Done
Cancel
─────────────
✓ Call to Adventure
─────────────
Open Next Quest
Quit
```

Timer elapsed time is NOT shown in the native menu (native menus don't
update dynamically while open). The timer is visible in the main window.

### Menu Event Handling

All menu events handled in `.on_menu_event()`:

| Menu Item | Action |
|---|---|
| Done (selection) | Call `complete_quest`, rebuild menu, emit event |
| Quest Now | Call `start_timer`, rebuild menu, emit event |
| Something Else | Increment skip count, rebuild menu |
| Done (timer) | Call `complete_timer`, rebuild menu, emit event |
| Cancel | Call `cancel_timer`, rebuild menu, emit event |
| Call to Adventure | Toggle checkbox state (stored in app state) |
| Open Next Quest | Show + focus main window |
| Quit | `app.exit(0)` |

### Menu Rebuild

A helper function `rebuild_tray_menu` reads current state (next quest,
timer, Call to Adventure toggle) and replaces the tray menu. Called after
every state-changing action.

The tray handler needs access to `DbState` and `AppTimerState` — these
are already managed by Tauri and accessible via `app.state()`.

### Skip Count State

The tray menu's Something Else needs its own skip count (separate from
the frontend's `nextQuestIndex`). Add to the managed app state:

```rust
pub struct TrayState {
    pub skip_count: i32,
    pub call_to_adventure: bool,
}
```

Reset `skip_count` to 0 on Done or when quests change.

### Close-to-Tray

Intercept window close via `.on_window_event()`:

```rust
.on_window_event(|window, event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
        window.hide().unwrap();
        api.prevent_close();
    }
})
```

The app continues running in the tray. "Open Next Quest" shows the window
again. "Quit" exits the app.

### Tray → Window Sync

After the tray performs a state-changing action (Done, Quest Now, Cancel,
Something Else), emit a Tauri event so the frontend reloads:

```rust
app.emit("quest-state-changed", ()).ok();
```

Frontend listens on startup:
```javascript
window.__TAURI__.event.listen("quest-state-changed", () => loadAll());
```

### Window → Tray Sync

When the frontend changes state (Done, Quest Now, etc.), the tray menu
needs rebuilding. Emit an event from the frontend, or rebuild the menu
in the relevant Tauri commands.

Simplest approach: the tray menu is rebuilt at the start of every menu
open (Tauri doesn't have a "menu will open" hook, so instead rebuild
after every command that changes quest/timer state). Since `complete_quest`,
`start_timer`, `cancel_timer`, `complete_timer`, and `add_quest` are the
state-changing commands, we rebuild after each.

Alternative: rebuild on a short interval (every few seconds). Simpler
to implement but slightly wasteful.

Recommend: rebuild in the commands themselves, since we already have the
state access there.

## NOT in this step

- Call to Adventure overlay (Group C)
- Call to Adventure polling/interval logic (Group C)
- Custom tray icon (future)
- Dynamic timer display in tray menu (not possible with native menus)

## Done When

- App shows an icon in the Mac menu bar
- Clicking the icon shows a native menu with the current quest
- Done/Quest Now/Something Else/Cancel work from the tray menu
- Call to Adventure toggle appears (no overlay behavior yet — just the toggle)
- "Open Next Quest" shows/focuses the main window
- "Quit" exits the app
- Closing the main window hides it; app stays in tray
- Tray and window stay in sync (actions in one reflect in the other)
- All existing tests pass, app builds and runs
