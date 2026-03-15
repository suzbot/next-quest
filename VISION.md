# Next Quest — Vision

## The Core Insight

The app is not just quest list, it is a **quest giver**.

The #1 ADHD trap with task apps is that *managing the system becomes the task*.
Next Quest flips this: you tell it your goals and values, and it suggests
**one thing to do right now**. It's the NPC you walk up to who says
"Your next quest is..."

### RPG Theme
The entire UX is framed as an RPG. You're not "checking off tasks" — 
you're completing quests, earning XP, leveling up your character, and progressing skills. 
The language, visuals, and feedback loops all reinforce this framing. 
Tasks are quests. Multi-step endeavors are Epics or Sagas.

### Hard Rule: No Streaks
Streaks are punishment systems disguised as motivation. One missed day breaks
the streak, and now the system that was supposed to help you feels like another
thing you failed at. Next Quest never tracks consecutive days. Every day is a
fresh start.

Instead of tasks being on a strict schedule, there's a refresh rate of when the quest giver will start reminding you of them.
This doesn't keep you from completing them and earning XP as early and as often as you want.

---

## Problems We're Solving

| Problem | Design Response |
|---|---|

| No routine sticks | **Quest rotation**: the app remembers recurring tasks so you don't have to |
| Can't initiate | **One quest at a time**: no list paralysis, just "do this" |
| Big goals feel overwhelming | **Epics**: big goals broken into tiny steps, revealed one at a time |
| Hyperfocus on computer — forget to do life | **Interrupt system**: timed prompts that pull you away |
| Limited energy budget | **Energy modes**: "Daily Maintenance" vs "Boss Fight" mode |
| Need to clear small stuff before tackling big stuff | **Gearing Up**: daily mode clears the deck, then unlocks boss mode |
| Don't know where to start (home repair, etc.) | **Template epics**: prebuilt breakdowns for common life tasks |

## What Motivates (The Reward System)

- **Skill/Attribute/Badge system** (from Dominate Life model):
  - **Attributes** = personal values (Discipline, Health, Creativity, etc.)
  - **Skills** = directional goals (Reading, Fitness, Cooking, etc.)
  - **Badges** = discrete achievements (2026 Spring Cleaning, Read 12 Books, etc.)
- **XP for stepping away**: lock timer gives XP (Focus Hero model)
- **Visual meters**: progress bars, daily completion arcs (NO streaks — ever)
- **Character progression**: RPG-style avatar that visibly levels up
- **Bounded rewards**: "Do this quest, earn 30 min free computer time"



## Novelty Engine (Future)

The system that keeps the system fresh:
- Rotating quest presentation styles
- Seasonal themes for the character
- Random bonus quests ("side quests")
- Periodic "respec" prompts to revisit values/goals
- Surprise rewards and milestones

---

## Architecture Layers

```
+---------------------------+
|   Quest Giver UI          |  <- What you see: one quest, your list, your character, meters
+---------------------------+
|   Quest Selector Engine   |  <- Picks the next quest based on mode, energy, last completed, time of day, difficulty, etc
+---------------------------+
|   Quest Library           |  <- All quests: recurring, sagas, templates, custom
+---------------------------+
|   Character & Progression |  <- XP, levels, skills, attributes, badges
+---------------------------+
|   Interrupt System        |  <- Timers, notifications, screen prompts
+---------------------------+
```

---

## (Phase 0): "The List" ✓

Basic Function to start using the app

1. **Quest list view**: where you seed and review a basic tasks structure
  a. Ability to view, add, and edit recurring and one off tasks
    i.  Task Name
    ii. Cycle (how frequestly would like to be offered quest)
  b. Ability to mark as 'Done'
  c. See 'Last Done' Date/Time
  d. Ability to re-sequence tasks


## Phase 0.5 - "Table Stakes" ✓

Getting in line with what's already out there (Dominate Life), so I use this instead

 1. **Character View**: Text to start
    a. See Level and XP
    b. Attributes (Default: Health, Pluck, Knowledge, Connection, Responsibility)
    d. CRUD SKills (Defaults: Nature, Buerecracy, Language, Animal Handling, Cooking, Community, Cleaning, Sociality, Logistics, Healing, Crafting, Acrobatics)
 
2. Incorporate Difficulty Level to Task (Trivial, Easy, Moderate, Challenging, Epic)

 3. Link tasks to skills/attributes, so that on task completion
    a. General XP / Level goes up based on cycle and difficulty of completed task
    b. Attribute XP/level progresses based on cycle and difficulty, if associated to completed task
    c. Skill XP/level progresses based on cycle and difficulty, if associated to completed task


## (Phase 1): "The Quest Giver" ✓

The app stops being a list and starts being a quest giver:

1. **Next Quest View**: Quest Giver tab shows one quest at a time with Done, Quest Now, and Something Else
2. **Quest selection**: app picks the next due quest in list order, falls back to longest-ago-completed
3. **Break timer**: Quest Now starts a timer while you go do the thing. Done completes, Cancel abandons
4. **Quest Now from list**: any quest can be manually triggered into Quest Now flow
5. **Menu bar app**: lives in Mac's top bar with Call to Adventure toggle, Open Next Quest, and Quit. Close-to-tray keeps the app running
6. **Call to Adventure**: overlay interruption that pops up over your work when a quest is available (peon-ping style). Click to go to the quest, Maybe Later to snooze
7. **Local time for quest due dates**: cycles reset at local midnight, not UTC
8. **Code consolidation**: duplicate link-loading code unified


## Phase 1.5: "Enhanced Overlay" ✓

The Call to Adventure overlay becomes a mini quest giver:

1. **Quest name in overlay**: shows "A quest awaits..." with the actual quest name
2. **Done from overlay**: complete quests without opening the full app, XP flash, advances to next quest
3. **Quest Now from overlay**: starts timer and opens main window in timer mode
4. **Something Else in overlay**: cycle through quests within the overlay
5. **Maybe Later**: dismisses overlay, restarts interval

## Phase 2: "Level Up"

- Graphics
- Quest completion feedback — visual/animated reaction when marking a quest done (flash, sound, XP popup, etc.)
- More quest attributes and logic
- XP time-elapsed modifier: diminishing returns for repeating a quest quickly (same-day laundry), increasing reward as time passes to encourage doing it, but log-curve so procrastination isn't linearly rewarded
- Smarter quest giving
- Sagas for bigger goals
- Multiple Quest Givers
- Receive coin and items from questing
- Badges, Rewards
- Buy stuff with gold
- Editable Attributes and Skills
- Character Class selection
- Elemental Alignment Matrix: Fire (Fight, Shout, Hot and Bright) <-> Water (Follow, Attune, Adapt, Flow, Listen, Reflect), Earth (Grounded, Solidity, Present,) <-> Sky (Abstract, Future/Past, Thinking, Planning, Remembering) Centered: Green, Growth


## Phase 3: "The Full Party"

- Character image with visual progression
- Novelty engine
- Template Sagas
- Training / Patrolling / Battling Modes
- Bounded reward system (computer time earning)



