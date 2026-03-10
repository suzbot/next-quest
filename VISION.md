# Next Quest — Vision

## The Core Insight

The app is a **quest giver**, not a quest list.

The #1 ADHD trap with task apps is that *managing the system becomes the task*.
Next Quest flips this: you tell it your goals and values once, and it tells you
**one thing to do right now**. It's the NPC you walk up to who says
"Your next quest is..." — not the 47-page journal you never open again.

### RPG Theme
The entire UX is framed as an RPG. You're not "checking off tasks" — you're
completing quests, earning XP, leveling up your character, and progressing
through skill trees. The language, visuals, and feedback loops all reinforce
this framing. Tasks are quests. Categories are guilds or skill lines. Big goals
are quest chains. Daily stuff is grinding. Hard stuff is boss fights.

### Hard Rule: No Streaks
Streaks are punishment systems disguised as motivation. One missed day breaks
the streak, and now the system that was supposed to help you feels like another
thing you failed at. Next Quest never tracks consecutive days. Every day is a
fresh start. Your XP and levels only go up — they never decay, reset, or
punish absence. You pick up exactly where you left off.

---

## Problems We're Solving

| Problem | Design Response |
|---|---|
| Hyperfocus on computer — forget to do life | **Interrupt system**: timed prompts that pull you away |
| No routine sticks | **Quest rotation**: the app remembers recurring tasks so you don't have to |
| Can't initiate | **One quest at a time**: no list paralysis, just "do this" |
| Big goals feel overwhelming | **Quest chains**: big goals broken into tiny steps, revealed one at a time |
| To-do lists grow forever | **The app manages the list, not you**: you seed it, it curates |
| Don't know where to start (home repair, etc.) | **Template quest chains**: prebuilt breakdowns for common life tasks |
| Limited energy budget | **Energy modes**: "Daily Maintenance" vs "Boss Fight" mode |
| Need to clear small stuff before tackling big stuff | **Gearing Up**: daily mode clears the deck, then unlocks boss mode |

## What Motivates (The Reward System)

- **Character progression**: RPG-style avatar that visibly levels up
- **Skill/Attribute/Badge system** (from Dominate Life model):
  - **Attributes** = personal values (Discipline, Health, Creativity, etc.)
  - **Skills** = directional goals (Reading, Fitness, Cooking, etc.)
  - **Badges** = discrete achievements (2026 Spring Cleaning, Read 12 Books, etc.)
- **XP for stepping away**: lock timer gives XP (Focus Hero model)
- **Visual meters**: progress bars, daily completion arcs (NO streaks — ever)
- **Bounded rewards**: "Do this quest, earn 30 min free computer time"

## Modes

### Daily Maintenance Mode ("Grinding")
- Recurring quests: shower, meds, dishes, laundry, etc.
- Low-energy, rote tasks
- Quick XP, keeps the basics running
- Clears the mental deck

### Boss Fight Mode ("Raiding")
- Bigger goal quest chains (revealed one step at a time)
- Requires more focus/energy
- Unlocked after daily basics are handled (or manually overridden)
- Deeper XP rewards, badge progress

### Focus Mode ("AFK Quest")
- Computer locks/dims, timer runs
- XP accrues while you're away on your quest
- Return triggers quest completion + reward feedback
- Optional: background audio continues (ambient/podcast)

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
|   Quest Giver UI          |  <- What you see: one quest, your character, meters
+---------------------------+
|   Quest Selector Engine   |  <- Picks the next quest based on mode, energy, time
+---------------------------+
|   Quest Library           |  <- All quests: recurring, chains, templates, custom
+---------------------------+
|   Character & Progression |  <- XP, levels, skills, attributes, badges
+---------------------------+
|   Interrupt System        |  <- Timers, notifications, screen prompts
+---------------------------+
```

---

## MVP (Phase 1): "The Quest Giver"

The smallest thing that's already useful:

1. **Menu bar app** (lives in your Mac's top bar, always accessible)
2. **One screen**: shows your next quest + a "Done" button
3. **Quest pool**: you seed ~10-20 simple tasks (recurring dailies + a few one-offs)
4. **Quest selection**: app picks one for you based on simple rules (time of day, not-recently-done)
5. **XP + Level**: completing quests gives XP. You have a level. It goes up. Dopamine.
6. **Break timer**: "Step away for 15 min" prompt on a configurable interval. XP for honoring it.
7. **Daily reset**: recurring quests refresh each day (no streak tracking — just fresh quests)

**What MVP deliberately leaves out:**
- Character avatar/visuals (just a level number + XP bar for now)
- Quest chains / big goals (just flat tasks)
- Energy modes (just one mode)
- Templates
- Audio integration
- Novelty engine

This gets you a working quest giver in days, not months. You start using it
immediately, and the feedback from real use shapes what Phase 2 looks like.

## Phase 2: "Level Up"

- Character with visual progression
- Attributes, Skills, Badges
- Two modes (Daily Maintenance / Boss Fight)
- Quest chains for bigger goals

## Phase 3: "The Full Party"

- Focus/AFK timer with XP
- Template quest chains
- Novelty engine
- Bounded reward system (computer time earning)

---

## Tech Decision (TBD)

Options for a Mac desktop app:
- **SwiftUI** (native macOS, best for menu bar apps, notifications, system integration)
- **Tauri** (Rust + web frontend, cross-platform potential, lighter than Electron)
- **Electron** (web tech, heaviest, but most flexible UI)

Recommendation: **SwiftUI** if we're committed to Mac-only. Gives us native menu bar,
notifications, screen dimming, and feels right at home. But we should decide based on
your comfort level and whether you might want this on other devices later.
