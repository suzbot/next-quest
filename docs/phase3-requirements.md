# Phase 3: The Three Quest Givers — Requirements

**Status:** Draft

**Goal:** Split the quest giver into three lanes by difficulty, each with its own personality, images, and flavor text. Hard things stop competing with trivials. The user can see what's coming at every difficulty level and engage when ready.

---

## User story

> My app is great at keeping me on top of pills, cat food, and daily routines. But hard things — taxes, doctor appointments, deep cleaning — never surface because they're drowned out by important trivials. I avoid them because they ambush me; I need to see them coming so I can psych up.
>
> I want three quest givers on my Next Quest tab:
> - The Captain of the Guard assigns my daily duties. Always there, always reliable.
> - The Adventurer's Guild posts jobs for when I'm warmed up. Errands, cleaning, exercise.
> - The Royal Court has the one big thing the realm needs from me today. I can see it all day and engage when I'm ready.

---

## 1. Three lanes on the Next Quest tab

The Next Quest tab shows three stacked sections, each functioning as an independent quest giver:

| Lane | Name | Difficulty filter | Personality |
|---|---|---|---|
| 1 | Castle Duties | Trivial | Captain of the Guard |
| 2 | Adventures | Easy + Fair | The Adventurer's Guild |
| 3 | Royal Quests | Hard + Epic | The Royal Court |

Each lane displays:
- Quest giver image (from lane-specific image pool)
- Flavor text (from lane-specific text pool)
- Quest name (with saga name if applicable)
- Done / Quest Now / Something Else buttons

If a lane has no eligible quests, it shows a quest giver image with empty state text:
- Lane 1: "The walls are secure."
- Lane 2: "I haven't heard any new rumors."
- Lane 3: "The realm is at peace."

Lanes are stacked vertically, Castle Duties on top.

## 2. Lane-specific images and flavor text

Images and flavor text are organized by lane for easy reskinning:

```
ui/images/lane1/     — Captain of the Guard images
ui/images/lane2/     — Adventurer's Guild images
ui/images/lane3/     — Royal Court images
ui/text/lane1/       — Castle Duties flavor text
ui/text/lane2/       — Adventures flavor text
ui/text/lane3/       — Royal Quests flavor text
```

Each lane loads its own pool at startup. No-repeat random selection within each lane (same as today's single pool).

Existing quest giver images can be distributed across lanes or new images created. Monster/victory images remain shared (used during Quest Now timer from any lane).

## 3. Scoring within lanes

Each lane uses the same scoring algorithm — importance, overdue ratio, list order, membership, balance, skip penalty. The only difference is the candidate pool is filtered by difficulty.

"Something Else" in one lane only skips within that lane's pool. Skip counts are per-quest (as today), so skipping a Royal Quest doesn't affect Castle Duties.

## 4. Saga difficulty and lane assignment

A saga's lane is determined by its hardest step's difficulty:

- If any step is Hard or Epic → Lane 3 (Royal Quests)
- Else if any step is Easy or Fair → Lane 2 (Adventures)
- Else → Lane 1 (Castle Duties)

ALL steps of a saga appear in the saga's lane, regardless of individual step difficulty. A trivial tax step is still part of a Royal Quest because the saga as a whole is epic-level work.

This is inferred automatically — no new field needed (though an explicit override could be added later).

## 5. CTA overlay

The Encounters overlay only surfaces Lane 1 (trivial) quests. Hard things are never surprise ambushes — they're visible on the Next Quest tab for the user to engage when ready.

The overlay continues to exclude whatever Lane 1 quest the main quest giver is showing.

## 6. Quest Now / timer

Starting a timer from any lane locks all three lanes (same tab-locking behavior as today). The timer view replaces all three lanes while active. Completion feedback shows in context of whichever lane initiated it.

## 7. Debug scoring

When debug scoring is enabled, each lane shows its score breakdown (same as today's single debug display).

---

## What doesn't change

- Quest list tab (shows all quests regardless of difficulty)
- Saga tab (unchanged)
- Campaign tab (unchanged)
- Character tab (unchanged)
- Scoring algorithm (unchanged — just called three times with different filters)
- Overlay UI (same, just filtered to trivial)

---

## Future extensions (not in this phase)

- Energy field on quests (energizing vs draining)
- Explicit saga difficulty override
- Lane-specific quest giver NPC names (editable)
- Different overlay behavior per lane
