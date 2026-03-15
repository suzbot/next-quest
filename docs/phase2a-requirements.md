# Phase 2A: "Look and Feel" — Requirements

## Goal

Give the app its visual identity. The aesthetic is 1985 Apple IIGS Bard's
Tale — pixel fonts, light gray palette, bordered panels, RPG UI framing.
The app should feel like an RPG interface, not a modern web app.

## Visual Reference

- **Primary**: The Bard's Tale (1985, Apple IIGS) guild/combat interface
- Light gray background, black text, single-pixel dark borders
- X-pattern crosshatch border frame around the app
- Clean pixel font (Silkscreen)

## Color Palette

### Base Colors

| Name | Hex | Usage |
|---|---|---|
| Background | #b4b4b4 | Panel and app background |
| Text | #111 | Primary text |
| Border | #444 | Panel borders, dividers |
| Meter BG | #fff | Unfilled portion of meter bars |

### Accent Colors (from Bard's Tale icons)

Each accent has two tones: a dark variant for text on the background
(WCAG AA 4.5:1+) and a bright variant for meter fills and icons
(on white backgrounds).

| Name | Text Hex | Fill Hex | Usage |
|---|---|---|---|
| Flame Red | #992200 | #ff6633 | Errors, destructive actions, Challenging difficulty |
| Compass Blue | #2244aa | #4466cc | Links, interactive highlights, Trivial difficulty |
| Carpet Green | #1a6e1a | #55cc55 | XP, positive values, Easy difficulty |
| Treasure Gold | #7a5500 | #eea800 | Moderate difficulty |
| Shield Purple | #5c2d80 | #8844aa | Special, rare, Epic difficulty |

### Accessibility

- Never rely on color alone to convey information — always pair with
  label, icon, or shape
- Avoid using Flame Red + Carpet Green to distinguish items (red-green
  colorblindness)
- All text accent colors pass WCAG AA 4.5:1 against #b4b4b4

### Difficulty Color Coding

| Difficulty | Color |
|---|---|
| Trivial | Compass Blue |
| Easy | Carpet Green |
| Moderate | Treasure Gold |
| Challenging | Flame Red |
| Epic | Shield Purple |

## Features

### Step 2A-1: Base Reskin (font + colors + borders)

Apply the visual identity across the entire app:

- **Silkscreen font** — Google Font, applied to all text
- **Background** #b4b4b4, text #111, borders #444
- **Button style** — outset border, background-matching, Silkscreen font
- **X-pattern border frame** around the app window
- **Apply to both** index.html and overlay.html
- **No layout changes** — purely color, font, and border swap
- Remove bracket notation from button text (e.g., `[Done]` → `Done`)

### Step 2A-2: Progress Meters on Character View

Add horizontal meter bars to the Character view:

- **In addition to** existing text display (Lv, XP fraction), not replacing
- White background meter, filled portion uses accent color
- **Single color per meter** — color aligns with the associated attribute:
  - Each attribute gets one of the five accent colors
  - Skills inherit their parent attribute's color
  - Character XP uses a neutral or distinct color
- Character XP, all 5 attributes, all 12 skills get meters

### Step 2A-3: Accent Colors Throughout

Apply accent colors to functional elements:

- **Difficulty labels** color-coded per the difficulty table above
- **XP flash** uses Carpet Green text variant
- **Level-up flash** — when gaining XP causes a level up, flash the
  level-up notification in the accent color of the leveled entity
  (attribute color for attributes, parent attribute color for skills)
- **Cooldown quests** use muted text color
- Dual-tone pattern: dark variants for text, bright variants for fills

### Step 2A-4: Quest Completion Feedback

Visual reaction when marking a quest done:

- Brief highlight or flash on the quest row or CTA section
- XP popup that's more prominent than current plain text
- Level-up notification in colored text (per attribute color)
- Visual only for now (no audio)

## Attribute → Color Mapping

| Attribute | Accent Color |
|---|---|
| Health | Flame Red |
| Pluck | Treasure Gold |
| Knowledge | Compass Blue |
| Connection | Carpet Green |
| Responsibility | Shield Purple |

Skills inherit the color of their parent attribute.

## What This Phase Does NOT Include

| Deferred Item | Planned For |
|---|---|
| Audio feedback (sounds) | Phase 2+ |
| Character image / avatar | Phase 3 |
| Pixel art icons in-app | Phase 2A+ (future step) |
| Animated meter fills | Phase 2+ |
| Custom app icon / tray icon | Phase 2+ |
