# Phase 2B: "Flavor Text and Images" — Design

## Overview

Two implementation steps. No backend changes — this is entirely frontend
HTML/CSS/JS and image assets.

## Image Strategy

Images live in `images/monsters/` and `images/quest-givers/` at the
project root. Since Tauri's `frontendDist` points to `ui/`, these need
to be accessible from the webview. Two options:

1. **Move images into `ui/images/`** — simplest, they get bundled with
   the frontend dist.
2. **Use Tauri's asset protocol** to serve files from outside `ui/`.

Recommend **option 1** — move the images folder into `ui/` so they're
served naturally. No Tauri config changes needed.

Images are loaded by scanning the directory at runtime. Since we can't
do filesystem reads from the webview, we'll hardcode the image filenames
in a JS array. When images are added/removed, the array is updated.

## Step 2B-1: Quest Giver View — Split Screen + Flavor Text

### Layout Change

The Quest Giver view switches from centered text to a split layout:

```
┌────────────────────────────────────────┐
│                 │                      │
│  [NPC Image]    │  Flavor text...      │
│                 │                      │
│                 │  QUEST NAME          │
│                 │                      │
│                 │  Does our stalwart   │
│                 │  hero accept this    │
│                 │  quest?              │
│                 │                      │
│                 │  Done  Quest Now     │
│                 │  Something Else      │
└────────────────────────────────────────┘
```

- Left panel: dark background (#222), random quest giver image, pixelated
  rendering
- Right panel: flavor text (italic), quest name (bold), prompt, action
  buttons
- Image changes when Something Else is pressed or a new quest loads
- Flavor text chosen randomly from the pool on each load

### Timer Mode Layout

```
┌────────────────────────────────────────┐
│                 │                      │
│  [Monster Img]  │  QUEST NAME          │
│                 │                      │
│                 │     03:42            │
│                 │                      │
│                 │  Done  Cancel        │
└────────────────────────────────────────┘
```

Monster image replaces the NPC when timer is active.

### Flavor Text Pool (Quest Giver)

```javascript
const questGiverLines = [
  "Word has reached the guild of a task requiring attention...",
  "An old friend calls upon your skills once more...",
  "The townsfolk speak of a deed that needs doing...",
];
```

### Image Lists

```javascript
const questGiverImages = [
  "images/quest-givers/Bard 1 01 Atari.gif",
  // ... all files
];

const monsterImages = [
  "images/monsters/Bard 1 14 Atari.gif",
  // ... all files
];
```

Random selection: `images[Math.floor(Math.random() * images.length)]`

### CSS

New styles for the split layout, reusing the mockup's approach:
- `.split` flex container
- `.split-image` with dark background, centered image, pixelated rendering
- `.split-text` with padding and flex-column layout

### What Changes

- `renderQuestGiver()` renders split layout instead of centered text
- `renderTimerMode()` renders split layout with monster image
- New random selection functions for images and flavor text

## Step 2B-2: Overlay — Monster Encounter + Battle Actions

### Layout Change

Overlay switches from simple centered text to split layout with monster
image and battle-themed actions.

### Encounter Flavor Text Pool

```javascript
const encounterLines = [
  "A trial lays before you...",
  "The obligations of this mystic place fall upon you without warning!",
  "You face death itself in the form of",
  "A shadow stirs in the darkness before you...",
  "Something is approaching...",
];
```

### Action Label Mapping

| Original | Battle Label |
|---|---|
| Quest Now | Fight |
| Something Else | Run |
| Done | Cast Completion |
| Maybe Later | Hide in the Shadows |

### Overlay Window Size

Needs to grow to accommodate the split layout. Roughly 480x220.

### What Changes

- `overlay.html` gets the split layout
- Button labels change to battle theme
- Random monster image and flavor text on each show
- Completion animation stays (pulse, XP flash, level-ups)

## Image Folder Move

Move `images/` into `ui/` so it's served by Tauri:

```
ui/
  images/
    monsters/
    quest-givers/
  fonts/
  index.html
  overlay.html
```

## Implementation Order

1. **2B-1**: Move images, split-screen quest giver view + timer mode
2. **2B-2**: Split-screen overlay with battle actions
