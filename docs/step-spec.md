# Step Spec: Phase 0.5, Step 4 — "Character View"

## Goal

Give the player a place to see what all that quest-completing has earned them.
A second page showing character level, attributes, and skills — plus the
ability to rename the character and reset all XP for testing.

## Scope

**Navigation:**
- Two buttons at the top of the page: `[Quests]` and `[Character]`
- Toggles between two container divs. No routing, no URL changes.
- Quests view is the default. Character view loads its data when shown.

**Character view layout (text-based, monospace):**
```
[Quests] [Character]

ADVENTURER                Level 1 (0/300 XP)
                          [Rename] [Reset]

ATTRIBUTES
  Health              Lv 1   (0/60 XP)
  Pluck               Lv 1   (0/60 XP)
  Knowledge           Lv 1   (0/60 XP)
  Connection          Lv 1   (0/60 XP)
  Responsibility      Lv 1   (0/60 XP)

SKILLS                                    ATTRIBUTE
  Cooking             Lv 1   (0/30 XP)    Health
  Healing             Lv 1   (0/30 XP)    Health
  Acrobatics          Lv 1   (0/30 XP)    Health
  Cleaning            Lv 1   (0/30 XP)    Pluck
  ...
```

**Character name editing:**
- Clicking `[Rename]` replaces the name with a text input + `[Save]` button
- Enter saves, Escape cancels
- Calls `update_character`

**Reset buttons (temporary dev/testing tools):**
- Placed below the character view content
- Three buttons: `[Reset Char]` `[Reset Quests]` `[Reset History]`
- Each button has a two-click "Sure?" confirmation: first click changes label
  to `[Sure?]`, second click executes, timeout reverts after ~2 seconds
- `Reset Char` — zeroes XP on character, all attributes, and all skills
- `Reset Quests` — deletes all quests (and their link rows)
- `Reset History` — deletes all completion records

**Backend:**
- New command: `reset_character` → zeroes all XP
- New command: `reset_quests` → deletes all quests, quest_skill, quest_attribute rows
- New command: `reset_completions` → deletes all quest_completion rows
- All other commands already exist from Steps 1-3

**Frontend data flow:**
- Switching to character view calls `get_character`, `get_attributes`,
  `get_skills` and renders the display
- Attribute names are cached and used to label skills' mapped attributes

## NOT in this step

- XP tuning (post-Phase 0.5)
- Visual polish, pixel fonts, progress bars (Phase 2)
- Add/rename/remove attributes or skills (Phase 2)

## Done When

- Navigation buttons switch between quests and character views
- Character view displays name, level, XP progress
- All 5 attributes shown with level and XP progress
- All 12 skills shown with level, XP progress, and mapped attribute name
- Character name is editable via rename flow
- Reset Char zeroes all XP
- Reset Quests deletes all quests and links
- Reset History deletes all completions
- All three reset buttons have "Sure?" confirmation
- This completes Phase 0.5

## Phase 0.5 Complete When This Step Ships

After this step, all four Phase 0.5 features are in place:
schema/seeds/difficulty, quest links, XP engine, and character view.
