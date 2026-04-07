# Next Quest — Mechanics

How the app works under the hood. If you want to understand why the quest giver picked a particular quest, how XP is calculated, or what all the numbers mean, then this is the reference for you.

## CLI

The `nq` command-line tool creates quests and queries data through the same business logic as the GUI. Both read and write the same database. See the [CLI Guide](cli-guide.md) for all commands and options.

## Quests

A quest is something you do. It has a **difficulty** and is either **recurring** (with a cycle in days) or **one-off**.

### Quest List

The quest list shows all quests and saga slots in priority order (highest priority at top). You can:

- **Add quests** with title, difficulty, cycle, importance, time-of-day, day-of-week, and linked skills/attributes/tags
- **Edit** any quest inline
- **Reorder** via drag-and-drop or shift+arrow keys
- **Complete** directly from the list with inline XP feedback
- **Filter** by search text, difficulty, importance, time of day, day of week, or due status
- **Search** across quest titles, saga names, skill names, attribute names, tags, difficulty labels, and importance marks

### Difficulty

Difficulty affects how much XP you earn and which quest giver lane the quest appears in.

| Difficulty  | Display Label | Examples                            |
| ----------- | ------------- | ----------------------------------- |
| Trivial     | Trivial       | Take meds, drink water              |
| Easy        | Easy          | Shower, do dishes                   |
| Moderate    | Fair          | Exercise, meal prep                 |
| Challenging | Hard          | Deep clean, long study session      |
| Epic        | Epic          | Major project milestone, file taxes |

### Cycles

Recurring quests refresh after a set number of days. One-off quests are done once and deactivated. Cycle length affects both XP earned and how urgently the quest giver surfaces the quest.

### Importance

A 0–5 rating (displayed as "!" marks) that is the strongest factor in quest selection. A quest marked !!!!! will surface well ahead of an unmarked quest, even if the unmarked quest is more overdue.

### Time of Day and Day of Week

Quests can be restricted to specific time windows and days. The quest giver only shows quests that match the current time and day.

| Window    | Hours (local)      |
| --------- | ------------------ |
| Morning   | 4:00 AM – 11:59 AM |
| Afternoon | 12:00 PM – 4:59 PM |
| Evening   | 5:00 PM – 8:59 PM  |
| Night     | 9:00 PM – 3:59 AM  |

Quests set to "all times" or "every day" are always eligible. These settings are also available as filters on the quest list.

### Category Tags

User-defined labels (e.g., "Computer", "Outside", "Phone") for organizing quests. Created inline when first applied. Searchable via the quest list's fuzzy search. No effect on scoring.

## Quest Giver

### Lanes

The Next Quest tab shows three quest givers, each offering one quest from a different difficulty tier:

| Lane | Name          | Difficulties     |
| ---- | ------------- | ---------------- |
| 1    | Castle Duties | Trivial          |
| 2    | Adventures    | Easy             |
| 3    | Royal Quests  | Fair, Hard, Epic |

Each lane scores and selects independently. "Something Else" skips within a single lane. Starting a timer from any lane locks all three until you finish or cancel.

### Actions

- **Done** — Complete the quest immediately. Earns XP.
- **Quest Now** — Start a timed session. "Victorious!" completes it; "Defeated" cancels.
- **Something Else** — Skip this quest. Counts as a skip (reduces the quest's score temporarily). A new quest appears.
- **⏾ Not Today** — Remove this quest from the candidate pool for the rest of the day. No skip counted, no scoring impact. Persists across app restart. Resets at midnight.

### How Quests Are Selected

The quest giver scores all eligible quests in a lane and picks the highest. Eligible means: active, matches the lane's difficulty, matches the current time of day and day of week, and not dismissed for today.

Quests are split into **due** (cycle elapsed or one-off never completed) and **not-due** pools. Due quests are always preferred. Not-due quests only appear if there are no due quests or all due quests have been skipped to zero.

See [Quest Scoring](#quest-scoring) for details on the math.

## Encounters Overlay

A small always-on-top window that fires on a configurable interval. Shows a single trivial quest (from the Castle Duties pool) with a quick-complete button. Designed to break hyperfocus without demanding much effort. Syncs with Lane 1 — both show the same quest.

Whether it is turned on, and the interval for how frequently it is displayed can be adjusted on the 'Settings' tab.

### Actions

- **Fight** — Start a timed session on the quest giver (same as Quest Now). The overlay closes.
- **Cast Completion** — Complete the quest immediately. Earns XP. The overlay closes.
- **Run** — Skip this quest (counts as a skip). A new quest appears in the overlay.
- **Hide in the Shadows** — Dismiss the overlay without acting. Does not count as a skip.

## Sagas

A saga is a multi-step goal with ordered sub-quests. Can be one-off or recurring.

### Lifecycle

1. **Active run** — A saga is due when it's a new one-off, or a recurring saga whose cycle has elapsed. The quest giver surfaces the first incomplete step.
2. **Step completion** — Complete steps in order. Each step earns XP individually.
3. **Run completion** — When all steps are done, a completion bonus is awarded. Recurring sagas reset and become due again after their cycle elapses.
4. **Between runs** — Recurring sagas cool down after a run completes. No steps appear in the quest giver until the next cycle.

You can start a new run early by completing a step before the saga is technically due.

### On the Quest List

Each saga appears as a single slot on the quest list, interleaved with regular quests in priority order. The slot shows the current active step (or step 1 if between runs).

- **Due sagas** show the active step with action buttons — complete it or start a timer.
- **Between-runs sagas** appear dimmed, like not-due recurring quests.
- **Completed one-off sagas** appear dimmed at the bottom.
- Sagas are reorderable alongside quests via drag-and-drop or shift+arrow.
- Clicking a saga slot's title navigates to the Sagas tab for editing.
- Fuzzy search matches step title, saga name, linked skills/attributes/tags, difficulty, and importance.

### Lane Assignment

A saga's lane is determined by its hardest step. A saga with an Epic step puts all its steps in the Royal Quests lane, even the trivial ones.

## Campaigns

Campaigns track progress toward a goal you define. Each campaign has one or more criteria — a quest or saga that must be completed a certain number of times. When all criteria are met, the campaign completes and is recorded as a permanent accomplishment on your character.

Quests that are part of an active campaign receive a small scoring boost (see [Membership Bonus](#membership-bonus)).

## Character, Attributes, and Skills

Your **character** is your overall RPG avatar. It levels up from all quest XP.

**Attributes** represent personal values — broad categories of what matters to you. The defaults are Health, Pluck, Knowledge, Connection, and Responsibility.

**Skills** are directional goals mapped to attributes — specific areas you want to grow. Completing quests linked to a skill directs XP to both the skill and its parent attribute.

**Accomplishments** are permanent records of completed [campaigns](#campaigns). They appear on the Character tab as a history of milestones you've achieved. Accomplishments survive even if the campaign is later deleted.

## Quest Scoring

Quests are presented by the quest giver based on an internal scoring system, with highest scoring eligible quests being presented first.

```
score = overdue_ratio + importance_boost + list_order_bonus + membership_bonus + balance_bonus - skip_penalty
```

**Importance** is the dominant signal. **Overdue ratio** is secondary. The rest are tiebreakers.

#### Importance Boost

```
importance × 30.0 / (1 + skips)
```

Each importance level adds ~30 points — equivalent to a daily quest being 30 days overdue. Skipping divides the boost rather than subtracting: 1 skip halves it, 2 skips cut it to a third. After many skips, an important quest's score approaches that of an unimportant quest. Never goes negative.

| Importance | 0 skips | 1 skip | 2 skips | 3 skips |
| ---------- | ------- | ------ | ------- | ------- |
| 0 (—)      | 0       | 0      | 0       | 0       |
| 1 (!)      | 30      | 15     | 10      | 7.5     |
| 2 (!!)     | 60      | 30     | 20      | 15      |
| 3 (!!!)    | 90      | 45     | 30      | 22.5    |
| 5 (!!!!!)  | 150     | 75     | 50      | 37.5    |

#### Overdue Ratio

How urgently the quest needs doing. Higher = more overdue.

- **Recurring quests:** days since last completed, divided by cycle length (minimum 1.0 when due)
- **One-off quests:** days since created, scaled to a 9-day equivalent cycle
- **Saga steps:** days since the step became active, using the saga's cycle

#### List Order Bonus

Your position on the quest list is your priority. Quests higher on the list get a larger bonus (up to 1.0). Sagas use their position on the quest list, same as regular quests.

#### Membership Bonus

+1.0 for quests or saga steps that are part of an active campaign. Doesn't stack across multiple campaigns.

#### Balance Bonus

Gently nudges quests that feed underleveled attributes or skills. If a quest is linked to a skill that's below the average level, it gets a small boost.

#### Skip Penalty

Each "Something Else" or "Run" adds 0.5 to the penalty. Resets at local midnight, or manually via Settings → Reset Skips.

Note: skips also divide the [Importance Boost](#importance-boost) (1 skip halves it, 2 skips cut it to a third). This is proportional to importance on purpose — a flat 0.5 penalty wouldn't be enough to move a !!!!! quest, making high-importance quests functionally unskippable. Dividing instead lets the user skip anything when they need to, while still weighting important things heavily on the first offer.

## XP and Progression

### How XP Is Earned

```
Base XP     = 5 × difficulty_multiplier × cycle_multiplier
Time mult   = f(time_since_last_done / cycle_days)
Final XP    = round(Base XP × Time mult)
```

Three factors combine:

1. **Difficulty** — harder quests earn more
2. **Cycle** — less frequent quests earn more per completion
3. **Time elapsed** — doing things late earns a bit extra, but doing things early still earns most of the XP

### Difficulty Multipliers

| Difficulty | Multiplier | Base XP (daily) |
| ---------- | ---------- | --------------- |
| Trivial    | 1x         | 5               |
| Easy       | 5x         | 25              |
| Fair       | 10x        | 50              |
| Hard       | 20x        | 100             |
| Epic       | 40x        | 200             |

### Cycle Multipliers

| Cycle     | Multiplier       | Examples                            |
| --------- | ---------------- | ----------------------------------- |
| Recurring | sqrt(cycle_days) | Daily=1x, Weekly=2.6x, Monthly=5.5x |
| One-off   | 3x               | Fixed (equivalent of ~9-day cycle)  |

### Base XP Table (difficulty × cycle, before time modifier)

|                  | Trivial | Easy | Fair | Hard | Epic  |
| ---------------- | ------- | ---- | ---- | ---- | ----- |
| **Daily**        | 5       | 25   | 50   | 100  | 200   |
| **Every 3 days** | 9       | 43   | 87   | 173  | 346   |
| **Weekly**       | 13      | 66   | 132  | 265  | 529   |
| **Monthly**      | 27      | 137  | 274  | 548  | 1,095 |
| **One-off**      | 15      | 75   | 150  | 300  | 600   |

### Time-Elapsed Modifier

Applied to recurring quests based on how long since last completed, relative to the cycle.

```
r = time_since_last_done / cycle_days

r < 1:  multiplier = 0.1 + 0.9 × √r     (early: still rewarded)
r >= 1: multiplier = 1.0 + 0.5 × ln(r)   (late: bonus, but diminishing)
Floor:  0.1 (you always get something)
```

One-off and never-completed quests use a multiplier of 1.0.

| r    | Multiplier | Daily example  | Weekly example |
| ---- | ---------- | -------------- | -------------- |
| 0.50 | 0.74x      | 12 hours early | 3.5 days early |
| 1.0  | 1.00x      | On time        | On time        |
| 2.0  | 1.35x      | 1 day late     | 1 week late    |
| 7.0  | 1.97x      | 1 week late    | 7 weeks late   |

**Design rationale:** Square root ramp below cycle rewards doing things early (at half-cycle you earn 74% XP). Logarithmic growth above cycle motivates overdue quests without making procrastination a strategy.

### Combined Examples

| Scenario                   | Base XP | Time mult | Final XP |
| -------------------------- | ------- | --------- | -------- |
| Easy daily, on time        | 25      | 1.0x      | 25       |
| Easy daily, 12 hours early | 25      | 0.74x     | 19       |
| Easy daily, 3 days late    | 25      | 1.55x     | 39       |
| Fair weekly, on time       | 132     | 1.0x      | 132      |
| Epic monthly, on time      | 1,095   | 1.0x      | 1,095    |
| Hard one-off               | 300     | 1.0x      | 300      |

### XP Distribution

On completion, final XP is awarded to:

1. **Character** — always receives the full amount
2. **Linked attributes** — each receives the full amount
3. **Linked skills** — each receives the full amount

Quests with no links earn only character XP.

### XP Is Permanent

XP is tallied at completion and never subtracted. Deleting a completion removes it from the visible history but does not reduce XP totals.

### Saga Step XP

Saga steps use the parent saga's cycle for XP calculation. A step in a weekly saga earns weekly-cycle XP, regardless of the step's individual difficulty.

### Saga Completion Bonus

When all steps in a run are complete, a bonus of 20% of the total baseline XP across all steps is awarded. Based on baseline XP (time modifier = 1.0), not actual earned XP — no procrastination reward. Distributed to the character and the final step's linked skills/attributes.

### Attribute Levels

Same curve, same seeds as character (300, 500). Level 2 at 300 XP, level 5 at 2,900 XP, level 10 at 37,200 XP.

### Skill Levels

Seeds: 150, 300. Level 2 at 150 XP, level 5 at 1,650 XP, level 10 at 21,300 XP.

### Skill Level-Up Bonus

When a skill levels up, its parent attribute receives an XP bump equal to the base XP of a Fair one-off quest (currently 150 XP).

## Levels

Fibonacci-style progression: each level costs the sum of the previous two levels.

### Character Levels

| Level | XP Required | Cumulative |
| ----- | ----------- | ---------- |
| 2     | 300         | 300        |
| 3     | 500         | 800        |
| 4     | 800         | 1,600      |
| 5     | 1,300       | 2,900      |
| 6     | 2,100       | 5,000      |
| 7     | 3,400       | 8,400      |
| 8     | 5,500       | 13,900     |
| 9     | 8,900       | 22,800     |
| 10    | 14,400      | 37,200     |
