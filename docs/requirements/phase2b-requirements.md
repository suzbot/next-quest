# Phase 2B: "Flavor Text and Images" — Requirements

## Goal

Bring the quest giver to life. The app gets personality through randomized
RPG flavor text and visual character through quest giver and monster images.
The Bard's Tale combat/guild screen layout — illustration on one side,
text/actions on the other — becomes the model.

## Visual Layout

Split-screen inspired by Bard's Tale IIGS:
- **Left ~60%**: Character or monster illustration (animated GIF)
- **Right ~40%**: Flavor text, quest name, and action buttons
- Label below the image (quest giver name or monster name)

## Overlay — Monster Encounter

The overlay presents quests as monster encounters. A random monster image
appears on the left, random encounter text on the right.

### Encounter Flavor Text Pool

One is chosen randomly each time the overlay appears:

1. "A trial lays before you..."
2. "The obligations of this mystic place fall upon you without warning!"
3. "You face death itself in the form of"
4. "A shadow stirs in the darkness before you..."
5. "Something is approaching..."

### Overlay Actions (RPG-themed labels)

| Action | Label | Maps To |
|---|---|---|
| Quest Now | Fight | Start timer, open main window |
| Something Else | Run | Skip to next quest |
| Done | Cast Completion | Complete quest immediately |
| Maybe Later | Hide in the Shadows | Dismiss overlay, restart interval |

### Overlay Layout

```
┌──────────────────────────────────────────┐
│                   │                      │
│  [Monster Image]  │  Encounter text...   │
│                   │                      │
│                   │  QUEST NAME          │
│                   │                      │
│   "Dust Golem"    │  Do you...           │
│                   │  Fight               │
│                   │  Run                 │
│                   │  Cast Completion     │
│                   │  Hide in the Shadows │
└──────────────────────────────────────────┘
```

## Quest Giver View — Friendly NPC

The main window quest giver view shows a friendly NPC illustration on the
left with quest-giving text on the right.

### Quest Giver Flavor Text Pool

One is chosen randomly each time the view loads a new quest:

1. "Word has reached the guild of a task requiring attention..."
2. "An old friend calls upon your skills once more..."
3. "The townsfolk speak of a deed that needs doing..."

### Quest Giver Actions

Same functional actions as today (Done, Quest Now, Something Else) but
presented in the split-screen layout alongside the NPC image.

### Quest Giver Layout

```
┌──────────────────────────────────────────┐
│                   │                      │
│  [NPC Image]      │  Flavor text...      │
│                   │                      │
│                   │  QUEST NAME          │
│                   │                      │
│  "Guild Master"   │  Done                │
│                   │  Quest Now           │
│                   │  Something Else      │
└──────────────────────────────────────────┘
```

## Timer Mode — Monster Battle

When Quest Now is active (timer running), the view shows a monster image
(the quest is the enemy you're battling) with the timer and Done/Cancel.

## Image Assets

- **Quest giver images**: Animated GIFs of friendly NPCs, provided by user
  into a folder (e.g., `ui/images/quest-givers/`)
- **Monster images**: Animated GIFs of monsters/enemies, provided by user
  into a folder (e.g., `ui/images/monsters/`)
- Images are selected randomly from the available pool
- Future: generate pixel art programmatically

## Multiple Quest Givers (Future Enhancement)

Different NPCs with different personalities. Deferred until the base
image/text system is working — the architecture supports it by having
a pool of quest giver images and text lines.

## Open Questions

- Overlay window size may need to increase to fit the split-screen layout
- Should monster/quest giver selection be truly random, or should certain
  images map to quest difficulty or attributes?
- Should the quest giver NPC persist across quests (same NPC for a session)
  or change with each quest?
