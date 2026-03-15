# Phase 1.5: "Enhanced Overlay" — Design

## Goal

The Call to Adventure becomes a mode of the main window rather than a
separate overlay window. When the CTA fires, the main window appears as
a small interruption card showing the quest with action buttons. The user
can act directly or expand to see the full app. This eliminates the
two-window architecture and all macOS window-management issues.

## Current State

The CTA overlay is a separate Tauri window (overlay.html) that shows
"A quest awaits..." with a Maybe Later button. This window has persistent
issues with macOS focus management — closing/destroying it triggers
unwanted focus shifts to the main window.

## New Approach: Single Window with CTA Mode

The main window serves double duty. It has two sections:

1. **CTA section** (top) — always visible when CTA fires. Shows the
   flavor header, quest name, and action buttons.
2. **Tabs section** (bottom) — collapsible. Contains the existing
   Quest Giver, Quests, Character, and Settings tabs.

### Collapsed Layout (interruption card)

When the CTA fires, the window appears small and always-on-top:

```
        A quest awaits...

        Take a shower

  [Done]  [Quest Now]  [Something Else]

              [Maybe Later]

            ▼ Show full app
```

### Expanded Layout (full app)

User clicks "Show full app" to expand:

```
        A quest awaits...

        Take a shower

  [Done]  [Quest Now]  [Something Else]

              [Maybe Later]

            ▲ Hide full app
  ─────────────────────────────────
  [Quest Giver] [Quests] [Character] [Settings]

  (tab content here)
```

### Window Behavior

**When CTA fires:**
- Show the main window (if hidden)
- Set always-on-top
- Collapse the tabs section
- Resize window to card size (~360 wide, ~200 tall)
- Center on screen

**When user expands:**
- Remove always-on-top
- Expand tabs section
- Resize window to full size (~400 wide, ~600 tall)

**When user clicks Maybe Later:**
- Hide the window (back to tray)
- Restart the CTA interval

**When user opens via tray menu "Open Next Quest":**
- Show the window expanded (full app mode)
- No CTA section visible? Or CTA section always visible?

Decision: **CTA section is only visible when the CTA has fired and the
user hasn't dismissed it.** Opening the app manually via tray shows the
normal tabbed interface without the CTA header. This keeps the CTA
feeling like an interruption, not a permanent UI element.

### CTA Section Actions

- **Done** — completes the quest, shows XP flash, then advances to the
  next quest. CTA section stays visible so the user can keep going.
- **Quest Now** — starts the timer, hides the CTA section, shows the
  Quest Giver tab in timer mode. Removes always-on-top.
- **Something Else** — swaps to the next quest (skip count, wrapping).
- **Maybe Later** — hides the window back to tray. Restarts interval.

### What Gets Removed

- `ui/overlay.html` — deleted, no longer needed
- `show_overlay()` in main.rs — replaced with showing the main window
  in CTA mode
- `dismiss_overlay` command — replaced with new CTA-specific commands
- Overlay window creation/destruction logic

### Backend Changes

**New commands:**
| Command | What it does |
|---|---|
| `activate_cta()` | Sets CTA mode active, returns the next quest |
| `dismiss_cta(action, quest_id?)` | Handles Maybe Later, Quest Now |

**Polling thread changes:**
- Instead of creating an overlay window, calls the main window to enter
  CTA mode: set always-on-top, show window, emit event to frontend
- Frontend listens for a `cta-activated` event and renders the CTA section

**Window management:**
- `always_on_top` toggled via Tauri window API based on CTA mode
- Window resize via Tauri window API

### Frontend Changes

**index.html restructured:**
- CTA section div at the top (hidden by default)
- Expand/collapse toggle between CTA and tabs
- Tabs section wraps existing nav + tab content
- CTA mode state: `ctaActive` boolean

**New frontend functions:**
- `showCTAMode()` — show CTA section, hide tabs, resize window
- `expandFullApp()` / `collapseToCard()` — toggle tabs
- `dismissCTA("later")` — hide window
- `dismissCTA("quest_now", questId)` — start timer, show full app

## Fallback: Option A

If the single-window approach has unforeseen issues, fall back to the
overlay window with a show/hide pattern instead of create/destroy.
The polling loop would check `overlay.is_visible()` instead of
`overlay.is_some()`. This avoids the macOS focus issues from
window destruction.

## Implementation Order

1. **Restructure index.html** — add CTA section, expand/collapse toggle,
   wrap tabs in collapsible container. No backend changes. Test that
   existing app works with the new layout, CTA section hidden.
2. **Wire up CTA mode** — polling thread shows main window in CTA mode
   instead of creating overlay. CTA actions work. Remove overlay.html
   and overlay window code. Test full CTA flow.
