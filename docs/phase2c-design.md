# Phase 2C: "Flavor Enhancements" — Design

## Overview

Nine changes spanning frontend, backend, and build tooling. Most are
small UI changes in `index.html` and `overlay.html`. The two bigger
pieces are dynamic image loading (build-time manifest) and tab locking.

## 1. Dynamic Image Loading — Build-Time Manifest

### Problem

Image lists are hardcoded as JS arrays in both HTML files. Adding or
removing images requires editing code.

### Approach

A Rust build script generates a JSON manifest listing all images by
category. The frontend fetches this manifest at startup.

**Why build-time, not runtime:** Tauri's `frontendDist` embeds the
`ui/` directory into the binary for production builds. The files aren't
on the filesystem at runtime — they're served from memory. There's no
directory to scan. A build-time manifest becomes just another embedded
file.

### Implementation

1. Extend `src-tauri/build.rs` to scan `ui/images/` subdirectories
   for `.gif` files and write `ui/images/manifest.json`:

```json
{
  "quest-givers": ["images/quest-givers/Bard 1 01 Atari.gif", ...],
  "monsters": ["images/monsters/Bard 1 14 Atari.gif", ...],
  "victory": ["images/victory/..."],
  "defeat": ["images/defeat/..."]
}
```

2. Both `index.html` and `overlay.html` fetch `images/manifest.json`
   at startup and populate their image arrays from it.

3. Remove the hardcoded JS image arrays from both HTML files.

4. Remove the dead `get_image_lists` Tauri command from `commands.rs`.

### Fallback

If the manifest is missing or a category is empty, fall back to an
empty array. The UI already handles "no images" gracefully (the split
layout still renders, just without an image).

## 2. No-Repeat Random Selection

### Problem

`randomFrom(arr)` can return the same item twice in a row.

### Implementation

Replace `randomFrom()` with a `randomFromExcluding(arr, previous)`
function in both HTML files:

```javascript
function randomFromExcluding(arr, previous) {
  if (arr.length <= 1) return arr[0] || null;
  let pick;
  do { pick = arr[Math.floor(Math.random() * arr.length)]; }
  while (pick === previous);
  return pick;
}
```

Each call site tracks its own "previous" value. This applies to:
- Quest giver images
- Monster images
- Victory/defeat images
- Quest giver flavor text lines
- Encounter flavor text lines

The overlay tracks its own previous values independently from the
main window (they're separate webview contexts).

## 3. Victory and Defeat Images

### New Folders

- `ui/images/victory/` — shown on timer completion
- `ui/images/defeat/` — shown on timer cancellation

User provides the GIF assets. These are included in the manifest
alongside the existing categories.

### Timer Done Flow (`timerDone()`)

Currently shows the monster image during the completion feedback.
Change to: swap to a random victory image, then show the XP/level-up
feedback as today.

### Timer Cancel Flow (`cancelTimer()`)

Currently snaps straight back to the quest giver view with no feedback.
Change to: show a brief defeat screen (random defeat image + "Defeated"
text) for ~1.5 seconds, then fade back to the quest giver.

### No Victory/Defeat Images Available

If the victory or defeat folder is empty, fall back to the current
behavior (show the monster image for victory, snap back for defeat).

## 4. Victory and Defeat Button Names

In `renderTimerMode()`, change the button labels:

| Current | New |
|---|---|
| `Done` | `Victorious!` |
| `Cancel` | `Defeated` |

The button `onclick` handlers stay the same — only the labels change.

## 5. Rename "Call to Adventure" to "Encounters"

User-facing string changes only. Internal variable names stay as-is.

### Frontend (`index.html`)

- Settings heading: `<h2>Call to Adventure</h2>` → `<h2>Encounters</h2>`

### Backend (`tray.rs`)

- Tray menu label: `"Call to Adventure: ON"` / `"Call to Adventure: OFF"`
  → `"Encounters: ON"` / `"Encounters: OFF"`

The menu item ID (`"call_to_adventure"`), Rust struct fields, and DB
column (`cta_enabled`) all stay unchanged.

## 6. Toggle Switch for Encounters

Replace `<button id="cta-toggle-btn">` with a CSS-styled checkbox toggle.

### HTML Structure

```html
<label class="toggle">
  <input type="checkbox" id="cta-toggle" onchange="toggleCTA()">
  <span class="toggle-slider"></span>
  Encounters
</label>
```

### CSS

A track + thumb toggle styled to match the 8-bit aesthetic:
- Track: dark when off (`#666`), accent colored when on
- Thumb: light square that slides left/right
- Outset border to match existing button style

### Behavior

`loadSettings()` sets the checkbox `checked` state from the backend.
`toggleCTA()` reads `checked` and sends the new state.

## 7. Lock Tabs During Quest Now Timer

### Behavior

When `timerInterval` is active:

1. Tab buttons for Quests, Character, and Settings get a `locked` CSS
   class (greyed out, reduced opacity, `cursor: not-allowed`).
2. Clicking a locked tab shows "You are currently locked in battle!"
   as a temporary message (toast-style or inline) instead of switching
   views.
3. The Quest Giver tab remains active and clickable.
4. When the timer ends (either outcome), the lock is cleared.

### Implementation

Modify `showView()`:

```javascript
function showView(view) {
  if (timerInterval && view !== "quest-giver") {
    showBattleLockMessage();
    return;
  }
  // ... existing logic (minus the auto-cancel on navigate)
}
```

The existing behavior of auto-cancelling the timer when navigating away
is replaced by the lock — the user can no longer navigate away at all.

Add/remove the `locked` class on tab buttons when timer starts/ends.

### Battle Lock Message

A brief text flash near the tab bar: "You are currently locked in
battle!" — same animation style as XP flashes, fades after ~2 seconds.

## 8. Overlay Button Reorder

### Current Layout

Single column, top to bottom: Fight, Run, Cast Completion, Hide in the
Shadows.

### New Layout

Two rows of two buttons using CSS grid:

```
[ Fight          ] [ Cast Completion     ]
[ Run            ] [ Hide in the Shadows ]
```

### CSS

Change `.actions` from `flex-direction: column` to:

```css
.actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 3px;
}
```

Button order in HTML: Fight, Cast Completion, Run, Hide in the Shadows
(grid fills left-to-right, top-to-bottom).

Keyboard shortcuts (F/R/C/H) are unchanged.

## 9. Skill and Attribute Seed Data

### Changes to `db.rs` `seed_defaults()`

1. Add `("Technology", "Knowledge", 13)` to the skills array.
2. Change `("Animal Handling", "Responsibility", 11)` to
   `("Animal Handling", "Connection", 11)`.

### Frontend Skill-Attribute Maps

Both `index.html` and `overlay.html` have `skillAttrMap` objects used
for level-up colors. Update both:

- Add `"Technology": "Knowledge"`
- Change `"Animal Handling": "Responsibility"` to
  `"Animal Handling": "Connection"`

### Data Reset Required

These are seed-data changes. Requires a data reset on this build.
Note this in the step spec.

## Implementation Order

Suggest three steps, roughly ordered by complexity:

1. **2C-1: Cosmetic changes** — Button renames (victory/defeat labels),
   "Encounters" rename, overlay button reorder, toggle switch,
   seed data changes. All small, independent, low risk.

2. **2C-2: Build-time manifest + no-repeat random** — The manifest
   generation in `build.rs`, frontend manifest loading, removal of
   hardcoded arrays and dead backend code, and the `randomFromExcluding`
   function.

3. **2C-3: Victory/defeat images + tab locking** — New image folders
   and their integration into the timer flow, defeat screen for cancel,
   and tab locking during battle. Depends on the manifest from step 2.
