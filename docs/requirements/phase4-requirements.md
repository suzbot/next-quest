# Phase 4: Stats and Feedback — Requirements

**Status:** Draft

**Goal:** Give the user motivating, at-a-glance insight into their progress — how much they've done today, how today compares to their history, and what their all-time XP total is. Also enrich completion feedback to show which skills and attributes benefited.

---

## 1. XP Stats on Character tab

### User story

> I want to see how I'm doing today compared to my average. Did I have a productive day? Am I close to my high score? I want this to motivate me, not pressure me — it's information, not a target.

### Stats section

New section on the Character tab (below the existing meters, or between character meter and attributes). Displays:

| Stat | Description |
|---|---|
| Today's Score | Running total of XP earned today (resets at local midnight) |
| Last Score | Most recent non-zero daily XP total (yesterday or last active day) |
| Avg XP / Day | Average daily XP across all days with at least one completion. Excludes zero-XP days — this represents "what a normal active day looks like," not "how often do I use the app." |
| High Score | Highest single-day XP total ever |

### All-time XP

Show total all-time earned XP under the character name, alongside the level display. This is the character's stored XP value (not derived from completion history — consistent with shown levels).

### Data source

All stats are computed from the `quest_completion` table's `completed_at` and `xp_earned` columns, grouped by local date. Also includes bonus XP from saga completions and campaign completions (these are already reflected in character XP but may not all be in quest_completion — need to account for accomplishment bonus_xp too, or just use character.xp for all-time).

"Today" is determined by local date (same logic as quest due dates).

### What it doesn't do

- No targets or goals ("earn 500 XP today") — that creates pressure
- No streaks ("3 days in a row") — per the hard rule
- No comparison to other people
- No negative framing ("you're below average today")

---

## 2. XP type on celebration text

### User story

> When I complete a quest, I want to see which skills and attributes got XP, not just the total. "Cooking +25 XP, Health +25 XP" tells me more than "+25 XP" and makes the RPG progression feel real.

### Behavior

On quest completion, the XP feedback shows the distribution:
- Character XP always shown
- Each linked attribute and skill shown with its XP amount
- Format: "+25 XP (Cooking, Health)" or similar — concise, not a wall of text

Applies to all five completion paths (quest list, quest giver lanes, timer, saga tab, overlay).

### What it doesn't do

- Doesn't change XP calculation — just displays what's already happening
- Doesn't show skill level-up bonus (the 150 XP attribute bump) — that's already shown via the level-up notification

---

## Implementation notes

- Stats are read-only computations — no new tables needed
- "Today" boundary should use the same `local_today` logic as quest due dates for consistency
- Stats can be computed on demand when the Character tab loads (no background tracking needed)
- XP type display needs the quest's linked skills/attributes, which are already available in the completion flow

---

## Future extensions (not in this phase)

- Daily XP tracking table (for faster queries if history grows large)
- System-suggested daily todo list based on typical XP capacity
- Weekly/monthly trend visualization
