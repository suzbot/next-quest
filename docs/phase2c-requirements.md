# Phase 2C: "Flavor Enhancements" — Requirements

## Goal

Polish the RPG experience with victory/defeat visuals, thematic naming,
and quality-of-life improvements. The quest giver gets more personality,
the battle flow gets consequences you can see, and the image/text systems
become maintainable without code changes.

## Victory and Defeat Images

When the Quest Now timer completes or is cancelled, the image shown should
reflect the outcome.

- **Victory images**: Shown when the player completes the timer ("Victorious!").
  Stored in a new `ui/images/victory/` folder.
- **Defeat images**: Shown when the player cancels the timer ("Defeated").
  Stored in a new `ui/images/defeat/` folder.
- A random image is selected from the appropriate folder each time.
- Only applies to the timer flow in the main window quest giver view.
  The overlay's "Cast Completion" and the quest giver's instant "Done"
  continue to show the current monster/quest giver image as they do today.

## Victory and Defeat Button Names

The timer view's Done and Cancel buttons get thematic labels:

| Current Label | New Label | Action |
|---|---|---|
| Done | Victorious! | Complete the timed quest |
| Cancel | Defeated | Abandon the timed quest |

## Rename "Call to Adventure" to "Encounters"

All user-facing references to "Call to Adventure" become "Encounters":

- Settings view heading and toggle label
- System tray menu item
- Any internal state names that surface to the user

The overlay behavior itself doesn't change — just the name.

## Toggle Switch for Encounters

The current ON/OFF button for Encounters is ambiguous — it's unclear
whether clicking "OFF" means "it's currently off" or "click to turn off."

Replace with a labeled toggle switch that clearly shows current state.
Standard toggle appearance: a sliding switch with "ON" and "OFF" positions,
visually distinct active vs inactive states.

## Lock Tabs During Quest Now Timer

When a Quest Now timer is running, the player is in battle. They should not
be able to wander off to the quest list, character view, or settings without
explicitly ending the battle.

- Other tab buttons are visible but greyed out (not hidden).
- Clicking a locked tab shows the message: "You are currently locked in battle!"
- The only way to leave is to complete the quest ("Victorious!") or cancel
  it ("Defeated") from the timer view.

## Dynamic Image Loading from Folders

Currently, image lists are hardcoded as JS arrays in both `index.html` and
`overlay.html`. Adding or removing images requires editing code.

The user should be able to add or remove images from these folders and
see them in the game without code changes:

1. `ui/images/quest-givers/` — friendly NPC images
2. `ui/images/monsters/` — monster encounter images
3. `ui/images/victory/` — victory outcome images (new)
4. `ui/images/defeat/` — defeat outcome images (new)

It's acceptable if the flow requires an app relaunch or even a rebuild —
the key constraint is no code edits.

**Note:** A previous attempt at this exists in the backend but did not work.
The working approach will be determined in the design phase.

## No Immediate Repeats in Random Selection

When randomly choosing from any pool — flavor text, images, or any other
randomized content — never repeat the same item twice in a row. If the
pool has more than one item, the next selection must differ from the
previous one.

## Flavor Text from External Files

Already implemented in Phase 2B. Both `encounter-lines.txt` and
`quest-giver-lines.txt` are loaded from `ui/text/` at startup. No further
work needed beyond the no-repeat rule above.

## Skill and Attribute Changes

Two data changes to the default skill/attribute mappings:

1. **New skill**: Technology, under Knowledge.
2. **Move Animal Handling**: From Responsibility to Connection.

These are seed-data changes. No production data exists yet — a data reset
on this build is acceptable.

## Overlay Button Reorder

The overlay action buttons are rearranged from a single column into two
rows of two:

| Position | Left | Right |
|---|---|---|
| Row 1 | Fight | Cast Completion |
| Row 2 | Run | Hide in the Shadows |

The more committal action (engage) is on the left in each row, with the
alternative on the right.

## Summary of Changes

| Feature | Where it shows up |
|---|---|
| Victory/defeat images | Timer view completion and cancellation |
| Victory/defeat button names | Timer view action buttons |
| Rename to "Encounters" | Settings, tray menu |
| Toggle switch | Settings view |
| Tab locking | All tab buttons during active timer |
| Dynamic image loading | All image categories |
| No-repeat random selection | All random pools (text and images) |
| Overlay button reorder | Overlay action buttons |
| Technology skill | Character view, quest linking |
| Animal Handling move | Character view, quest linking |
