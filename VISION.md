# Next Quest — Vision

## The Core Insight

The app is not just quest list, it is a **quest giver**.

The #1 ADHD trap with task apps is that _managing the system becomes the task_.
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
| ------- | --------------- |

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
   i. Task Name
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

## Phase 2A: "Look and Feel" ✓

Initial visual identity for the app:

1. **Art direction**: Silkscreen pixel font, light gray Bard's Tale palette, X-pattern border, outset buttons
2. **Progress meters**: colored horizontal bars for character, attributes, and skills (color per attribute)
3. **Accent colors**: difficulty color-coded (Flame Red, Compass Blue, Carpet Green, Treasure Gold, Shield Purple)
4. **Quest completion feedback**: row flash + XP in difficulty color, quest giver pulse + fade, level-up notifications in attribute color, timer completion with elapsed time flash

## Phase 2B: "Flavor Text and Images" ✓

Bring the quest giver to life with personality and visuals:

1. **Randomized flavor text** on the overlay and quest giver
2. **Quest giver images** — side-by-side layout inspired by Bard's Tale guild screen
3. **Battle-themed overlay** — monster encounters with Fight/Run/Cast Completion/Hide in the Shadows
4. **Keyboard shortcuts** on the overlay (F/R/C/H)
5. **Monster image + encounter text carry through** to timer view

## Phase 2C: "Flavor Enhancements" ✓

Polish and quality-of-life improvements:

1. **Victory/defeat images** — timer completion shows victory image, cancellation shows defeat image with "Sorry, bud."
2. **Victory/defeat button names** — "Victorious!" and "Defeated" in timer view
3. **Rename to "Encounters"** — settings, tray menu (internal names unchanged)
4. **Toggle switch** — labeled Off/On toggle replaces ambiguous button
5. **Tab locking** — other tabs greyed out during Quest Now timer, "You are currently locked in battle!"
6. **Dynamic image loading** — build-time manifest from image folders, no hardcoded arrays
7. **No-repeat random selection** — images and text never repeat the same item twice in a row
8. **Overlay button reorder** — 2x2 grid: Fight | Cast Completion, Run | Hide in the Shadows
9. **Responsive overlay sizing** — window height adapts to content length
10. **Skill/attribute changes** — Technology (Knowledge) added, Animal Handling moved to Connection
11. **Skill-attribute map from DB** — level-up colors built dynamically, not hardcoded
12. **Timer rendering fix** — timer display updates text only, no DOM rebuild per tick

## Phase 2D: "Levelling" ✓

1. **Tab renames** — "Quests" → "Quest List", "Quest Giver" → "Next Quest"
2. **Quest list reorganization** — expandable detail rows, fixed-width meta columns, icon buttons (⚔/✓), edit-only delete, History moved to Character tab
3. **XP time-elapsed modifier** — piecewise multiplier (sqrt ramp below cycle, log growth above), 0.1x floor, 1.0x baseline at cycle
4. **Level curve retuning** — attributes from 1/5 to 1/2 character, skills from 1/10 to 1/8

## Phase 2E: "Editable Attributes and Skills" ✓

1. **Backend CRUD** — add, rename, delete for attributes and skills, skill attribute remapping, nullable attribute_id with migration
2. **Index-based color palette** — 10-color cycling palette (5 fill + 5 text), replaces hardcoded name-keyed maps
3. **Attribute UI** — inline rename, delete with confirm, add via + on header
4. **Skill UI** — inline rename with attribute dropdown, delete with confirm, add via + on header
5. **Keyboard navigation** — Arrow up/down between rows, E/Enter to edit, Tab to + buttons
6. **Collapsible History** — caret toggle on Character tab
7. **Orphaned quest links** — silently removed on skill/attribute deletion

## Phase 2F: "Logic Enhancement" ✓

Smarter quest selection based on new quest attributes and offer tracking:

1. **Time-of-day windows** — bitmask multiselect (Morning=1, Afternoon=2, Evening=4); hard-filtered in quest selector
2. **Day-of-week affinity** — bitmask multiselect (Mon=1..Sun=64); hard-filtered in quest selector
3. **Overdue escalation** — scoring system replaces list-order selection; overdue ratio is primary signal
4. **Last-offered freshness** — in-memory skip tracking with daily reset; 0.5 penalty per skip
5. **Quest selector scoring** — `score = overdue_ratio - skip_penalty + list_order_bonus`; due pool preferred, not-due fallback
6. **Manual filtering** — quest list filter bar (attribute, skill, time, day, due-only); frontend-only, AND-combined
7. **Debug scoring** — settings toggle shows score breakdown in quest giver
8. **UI polish** — difficulty labels renamed (Fair/Hard), cycle abbreviated (↻ #d), meter layout (level left, XP right)

## Phase 2F.5: "Cleanup" ✓

1. **Parameter structs** — `NewQuest`, `QuestUpdate`, `NewSagaStep` with `Default` impls and serde. Adding a field is now a one-place change.
2. **Test helpers** — `test_quest()` and `test_quest_with()` replace 60+ verbose test calls
3. **Attribute/skill resequencing** — Alt+Arrow keyboard and drag-and-drop on Character tab. Full ordered list rebuild, not pair swaps.

## Phase 2G: "Advanced Logic"

2G.1 - Sagas: ✓

1. **Saga tab** — create, edit, delete sagas (one-off or recurring with cycle days)
2. **Saga steps are quests** — full quest UI (title, difficulty, TOD, DOW, skill/attribute tags) within expandable saga view
3. **Quest selector integration** — one active step per saga in candidate pool, scored like one-offs, saga name shown in quest giver
4. **Current run logic** — `last_run_completed_at` stamp, due/cooldown styling, progress bar, early new-run support
5. **Step XP uses saga cycle** — recurring saga steps earn XP based on saga's cycle_days, not one-off multiplier
6. **Saga completion bonus** — 20% of baseline step XP, gold celebration notification, distributes to character + final step's linked skills/attributes
7. **Full resequencing** — drag-and-drop + Alt+Arrow keyboard reordering for steps, any position
8. **Quest Now + Done on steps** — timer flow and completion with XP flash from saga tab
9. **Saga completion from quest giver/overlay** — completing final step via quest giver or overlay stamps the saga

2G.2 - Campaigns and Honors:

Story:
I want to accomplish 'Spring Cleaning 2026'.
This is a way to track and reward accomplishing a series of related, already existing, quests (or sagas).
For example the 'Spring Cleaning' campaign may consist of "4 completions of Laundry saga, 4 instances of vaccuming, and 2 instances of mopping floors"
Completing the 'Spring Cleaning' campagin gives me an xp bonus.
On the character tab, in a second column i see a section called 'Accomplishments'
I see 'Completed Spring Cleaning 2026' in the Accomplishments
This can later be built on to do things like more complex and dynamic rewards, honors, titles, etc.

## Phase 2H: "Polish"

2H.1 Second logic pass after living with 2F:

1. Check and refine how 'saga steps' are integrated into quest-giver list.
2. **Edit last-done date** — correct mistakes in completion history
3. Split evening into Evening (EV) and Night (NT)
4. Add ! -> !!!!! Importance field, incorporate into quest giver, reweight against urgency
5. More weight to sequence (same as skip weight)
6. More weight if its in a saga or campaign? Or just active/started ones?
7. **Attribute/skill balancing** — favor quests linked to underleveled attributes for variety

2H.3 Potential UI reorg

1. Evaluate: adding quest timer to overlay instead of going into full app?
2. Evaluate: having quest giver view always at the top and tabs expand bottom detail?
3. Show kind of XP gained on celebration text?
4. Show on next step whether its part of a saga, feat, or both

## Phase 3: "The Three Quest Givers"

The quest giver becomes three quest givers, each with their own personality, images, and flavor text. Quests are separated into lanes by difficulty, so hard things don't compete with trivials. Each lane works identically to the current quest giver (same scoring, same UI pattern).

**Lane 1 — Castle Duties** (Captain of the Guard): Trivial quests. The daily routine — pills, meals, cat care. Always showing, always reliable.

**Lane 2 — Adventures** (The Adventurer's Guild): Easy and Fair quests. The stretch goals — errands, cleaning projects, exercise. Tackle these when you're warmed up.

**Lane 3 — Royal Quests** (The Royal Court): Hard and Epic quests. The big challenges — taxes, doctor appointments, major projects. Visible all day so you can psych up. Do at most one per day.

Key design decisions:
- All three lanes stacked vertically on the Next Quest tab, Castle Duties on top
- Each lane has its own quest giver images (`ui/images/lane1/`, `lane2/`, `lane3/`) and flavor text (`ui/text/lane1/`, `lane2/`, `lane3/`) for easy reskinning
- Same scoring algorithm within each lane — importance, overdue, list order, etc.
- Saga steps appear in whichever lane matches the saga's difficulty (inferred from hardest step)
- CTA overlay only surfaces Lane 1 (trivial) quests — no surprise ambush with hard things
- Quest Now / timer works from any lane (locks all three)

## Phase 4: "Everything Else"

Potential UI reorg:
- Evaluate: adding quest timer to overlay instead of going into full app?
- Evaluate: having quest giver view always at the top and tabs expand bottom detail?
- Show kind of XP gained on celebration text?
- Show on next step whether its part of a saga, feat, or both

Rewards:
- Receive coin and items from questing
- Buy stuff with gold
- Rotating fighting text at different skill/attribute levels

Polish and systems:
- **Soft preference / fallback relaxation** — time-of-day and day-of-week filters relax when nothing else is available, instead of hard-excluding
- Ability to 'Undo' a previously completed task, resetting the last done date and xp gains (only the most recent completed, or any?)
- **Attribute color customization** — store color on the attribute row instead of deriving from position index. User-picks or assignment UI so reordering doesn't change colors.
- Reset behavior rethink (what does "Reset Char" mean with custom skills/attributes?)
- Seed data guard rework (seed_data() skips if character exists, needs rethinking)
- SQLite WAL cleanup (WAL/SHM files must be deleted alongside DB file on reset)
- Elemental Alignment Matrix: Fire (Fight, Shout, Hot and Bright) <-> Water (Follow, Attune, Adapt, Flow, Listen, Reflect), Earth (Grounded, Solidity, Present,) <-> Sky (Abstract, Future/Past, Thinking, Planning, Remembering) Centered: Green, Growth
- Character image with visual progression
- Novelty engine
- Template Sagas
- Bounded reward system (computer time earning)
