# Phase 2A: "Look and Feel" — Design

## Overview

Four implementation steps, each independently testable. No backend
changes — this is entirely CSS, HTML, and frontend JS.

## Font Strategy

**Silkscreen** via Google Fonts. Load in both index.html and overlay.html:

```html
<link href="https://fonts.googleapis.com/css2?family=Silkscreen:wght@400;700&display=swap" rel="stylesheet">
```

This is a runtime dependency on Google Fonts. For offline use, we could
bundle the font file later — acceptable for now since the app requires
internet for nothing else.

Alternative: download the font files into `ui/fonts/` and use `@font-face`.
Avoids the external dependency. Recommend this approach to align with the
minimal-dependencies principle.

## Step 2A-1: Base Reskin

### CSS Changes (index.html)

Replace the entire `<style>` block. Key changes:

- `body` background: `#1a1a2e` → `#b4b4b4`
- `body` color: `#c8c8c8` → `#111`
- `body` font-family: `"Courier New"` → `'Silkscreen', monospace`
- All border colors: `#222`/`#333`/`#444` → `#444` consistently
- Button style: dark background → `#b4b4b4` with `border: 2px outset #ccc`
- Input style: dark background → `#ccc` with `border: 1px inset #888`
- `.hidden` stays `display: none !important`
- Quest due/cooldown colors adjusted for light background
- Remove bracket notation from all button text in HTML

### X-Pattern Border

Wrap the body content in an X-pattern border frame using CSS
repeating-linear-gradient (same technique as swatch page):

```css
.app-border {
  background:
    repeating-linear-gradient(45deg, transparent, transparent 4px, #9a9a9a 4px, #9a9a9a 5px),
    repeating-linear-gradient(-45deg, transparent, transparent 4px, #9a9a9a 4px, #9a9a9a 5px);
  background-color: #b4b4b4;
  padding: 12px;
  border: 2px solid #444;
}
```

### overlay.html

Same reskin: light background, Silkscreen font, matching button style.
Remove bracket notation from buttons.

### What NOT to change

- Layout, spacing, element structure
- Backend commands
- Functional behavior

## Step 2A-2: Progress Meters

### Character View Changes

Each stat row (character XP, attributes, skills) adds a horizontal meter
bar between the label and the text values:

```html
<div class="meter-row">
  <span class="meter-label">Health</span>
  <div class="meter-bar-bg">
    <div class="meter-bar-fill" style="width: 40%; background: #ff6633;"></div>
  </div>
  <span class="meter-text">Lv 2 (40/100 XP)</span>
</div>
```

### Color Assignment

The fill color is determined by the attribute. The `renderCharacterView`
function maps attribute names to fill colors:

```javascript
const attrFillColors = {
  "Health": "#ff6633",       // Flame Red
  "Pluck": "#eea800",        // Treasure Gold
  "Knowledge": "#4466cc",    // Compass Blue
  "Connection": "#55cc55",   // Carpet Green
  "Responsibility": "#8844aa" // Shield Purple
};
```

Skills look up their parent attribute's color via `attribute_id`.

Character XP meter uses a neutral color (e.g., `#888` or one of the
accent fills — TBD based on visual feel).

### CSS

```css
.meter-bar-bg {
  flex: 1;
  height: 10px;
  background: #fff;
  border: 1px solid #444;
}

.meter-bar-fill {
  height: 100%;
}
```

Fill width computed as: `(xp_into_current_level / xp_for_current_level) * 100%`.

## Step 2A-3: Accent Colors

### Difficulty Labels

In `renderQuestRow` and quest display, apply the difficulty color to
the difficulty label text:

```javascript
const difficultyColors = {
  "trivial": "#2244aa",      // Compass Blue text
  "easy": "#1a6e1a",         // Carpet Green text
  "moderate": "#7a5500",     // Treasure Gold text
  "challenging": "#992200",  // Flame Red text
  "epic": "#5c2d80"          // Shield Purple text
};
```

### XP Flash

Change XP flash color from current `#5a8a5a` to `#1a6e1a` (Carpet Green
text variant).

### Level-Up Flash

When `complete_quest` returns, the frontend needs to detect if a level-up
occurred. Currently the completion response only includes `xp_earned`.
Two approaches:

1. **Compare before/after**: fetch character/attribute/skill data before
   and after completion, compare levels. Extra API calls but no backend
   change.
2. **Return level-ups in completion response**: add a `level_ups` field
   to the `Completion` struct listing what leveled up. Backend change but
   cleaner.

Recommend **option 2** for Step 2A-4 (completion feedback). For Step 2A-3,
just apply colors to existing elements — level-up detection is Step 2A-4.

### Cooldown Quests

Change cooldown text color from current `#555` to `#777` (muted on light
background).

## Step 2A-4: Quest Completion Feedback

### Visual Flash

When a quest is completed (from quest list or CTA overlay):
- Brief highlight on the completed row (background flash)
- XP text appears in Carpet Green, slightly larger, then fades

### Level-Up Notification

Requires backend change: `complete_quest` returns level-up info.

Add to `Completion` struct:

```rust
pub struct Completion {
    // ... existing fields ...
    pub level_ups: Vec<LevelUp>,
}

pub struct LevelUp {
    pub name: String,       // "Health", "Cooking", "Character"
    pub new_level: i32,
    pub entity_type: String, // "character", "attribute", "skill"
}
```

The `complete_quest` function checks levels before and after XP award.
If any entity leveled up, it's included in the response.

Frontend displays: `"Health reached Level 3!"` in Flame Red text,
shown briefly after the XP flash.

## Implementation Order

1. **2A-1**: Base reskin — font, colors, borders, buttons. Test: app looks
   like the swatch page.
2. **2A-2**: Meters — character view gets colored bars. Test: all stats
   show meters with correct attribute colors.
3. **2A-3**: Accent colors — difficulty labels, XP flash, cooldown colors.
   Test: difficulty labels are color-coded, XP flash is green.
4. **2A-4**: Completion feedback — flash effects, level-up detection and
   display. Test: completing a quest shows visual feedback, level-ups
   flash in attribute color.
