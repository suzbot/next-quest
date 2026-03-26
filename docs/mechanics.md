# Next Quest — Mechanics

Formulas and tuning values used in the game systems. Updated as we implement
and adjust.

## XP Formula

```
Base XP     = 5 × difficulty_multiplier × cycle_multiplier
Time mult   = f(time_since_last_done / cycle_days)     [recurring only]
Final XP    = round(Base XP × Time mult)
```

Three factors combine to determine quest XP:

1. **Difficulty** — how hard the quest is
2. **Cycle** — how often it recurs (or one-off)
3. **Time elapsed** — how long since you last did it

### Difficulty Multipliers

| Difficulty | Multiplier | Base XP (daily) | Example |
|---|---|---|---|
| Trivial | 1x | 5 | Take meds, drink water |
| Easy | 5x | 25 | Shower, do dishes |
| Fair | 10x | 50 | Exercise, meal prep |
| Hard | 20x | 100 | Deep clean, long study session |
| Epic | 40x | 200 | Major project milestone, file taxes |

### Cycle Multipliers

| Cycle | Multiplier | Formula |
|---|---|---|
| Recurring | sqrt(cycle_days) | Daily=1x, 3-day=1.7x, Weekly=2.6x, Monthly=5.5x |
| One-off | 3x | Fixed (equivalent of ~9-day cycle) |

### Base XP Table (difficulty × cycle, before time modifier)

| | Trivial | Easy | Moderate | Challenging | Epic |
|---|---|---|---|---|---|
| **Daily** | 5 | 25 | 50 | 100 | 200 |
| **Every 3 days** | 9 | 43 | 87 | 173 | 346 |
| **Weekly** | 13 | 66 | 132 | 265 | 529 |
| **Monthly** | 27 | 137 | 274 | 548 | 1,095 |
| **One-off** | 15 | 75 | 150 | 300 | 600 |

### Time-Elapsed Modifier

A multiplier applied to recurring quest base XP based on how long since the quest was last completed, relative to its cycle.

```
r = time_since_last_done / cycle_days

r < 1:  multiplier = 0.1 + 0.9 × √r
r >= 1: multiplier = 1.0 + 0.5 × ln(r)
Floor:  0.1 (you always get something)
```

**Special cases:**
- One-off quests: multiplier = 1.0 (no cycle to measure against)
- Never completed: multiplier = 1.0

**Reference values:**

| r | Multiplier | Daily example | Weekly example |
|---|---|---|---|
| 0.04 | 0.28x | 1 hour ago | 7 hours ago |
| 0.14 | 0.44x | 3 hours ago | 1 day ago |
| 0.50 | 0.74x | 12 hours ago | 3.5 days ago |
| 1.0 | 1.00x | On time | On time |
| 2.0 | 1.35x | 2 days late | 2 weeks late |
| 3.0 | 1.55x | 3 days late | 3 weeks late |
| 7.0 | 1.97x | 1 week late | 7 weeks late |
| 30.0 | 2.70x | 1 month late | — |

### Combined Examples (all three factors)

| Scenario | Base XP | Time mult | Final XP |
|---|---|---|---|
| Easy daily, on time | 25 | 1.0x | 25 |
| Easy daily, 12 hours early | 25 | 0.74x | 19 |
| Easy daily, 3 days late | 25 | 1.55x | 39 |
| Moderate weekly, on time | 132 | 1.0x | 132 |
| Moderate weekly, 2 weeks late | 132 | 1.35x | 178 |
| Trivial daily, repeated after 1 hour | 5 | 0.28x | 1 |
| Epic monthly, on time | 1,095 | 1.0x | 1,095 |
| Epic monthly, 2 months late | 1,095 | 1.35x | 1,478 |
| Challenging one-off | 300 | 1.0x | 300 |

**Design rationale:** Piecewise formula with square root ramp below cycle (rewards doing things even if early — at half-cycle you earn 74% XP) and logarithmic growth above cycle (motivates overdue quests without making procrastination a strategy). The turn of the curve is at r=1, the target cycle time.

### XP Distribution

On quest completion, final XP is awarded to:
1. **Character** (always) — full XP amount
2. **Linked attributes** — full XP amount per linked attribute
3. **Linked skills** — full XP amount per linked skill

Quests with no links earn only character XP.

### XP Is Permanent

XP is tallied at the time of quest completion and never subtracted. Deleting a
completion removes it from the visible history but does not affect XP totals.

## Level Curves

Fibonacci-style: each level costs the sum of the previous two levels' costs.
Different scales for character, attributes, and skills.

### Character Level Curve

Seeds: 300, 500.

| Level | XP for this level | Cumulative XP |
|---|---|---|
| 1 | — | 0 |
| 2 | 300 | 300 |
| 3 | 500 | 800 |
| 4 | 800 | 1,600 |
| 5 | 1,300 | 2,900 |
| 6 | 2,100 | 5,000 |
| 7 | 3,400 | 8,400 |
| 8 | 5,500 | 13,900 |
| 9 | 8,900 | 22,800 |
| 10 | 14,400 | 37,200 |

### Attribute Level Curve (1/2 of character)

Seeds: 150, 250.

| Level | XP for this level | Cumulative XP |
|---|---|---|
| 2 | 150 | 150 |
| 3 | 250 | 400 |
| 4 | 400 | 800 |
| 5 | 650 | 1,450 |
| 6 | 1,050 | 2,500 |
| 7 | 1,700 | 4,200 |
| 8 | 2,750 | 6,950 |
| 9 | 4,450 | 11,400 |
| 10 | 7,200 | 18,600 |

### Skill Level Curve (1/8 of character)

Seeds: 37, 62.

| Level | XP for this level | Cumulative XP |
|---|---|---|
| 2 | 37 | 37 |
| 3 | 62 | 99 |
| 4 | 99 | 198 |
| 5 | 161 | 359 |
| 6 | 260 | 619 |
| 7 | 421 | 1,040 |
| 8 | 681 | 1,721 |
| 9 | 1,102 | 2,823 |
| 10 | 1,783 | 4,606 |

### Skill Level → Attribute Bump

When a skill levels up, its mapped attribute receives XP equal to the base XP
of a **Moderate one-off quest** (currently 150 XP). This is computed from the
formula, not hardcoded — if base XP or difficulty multipliers change, the bump
changes with them.

## Quest Selector Scoring

### Combined Score

```
score = overdue_ratio + importance_boost + list_order_bonus + membership_bonus + balance_bonus - skip_penalty
```

Importance is the dominant signal. Overdue ratio is secondary. List order, membership, and balance are tiebreakers. Skips diminish importance rather than subtracting from the score.

### Importance Boost (dominant signal)

```
importance_boost = importance × 30.0 / (1 + skips)
```

Importance (0–5) is the strongest scoring factor. Each level adds ~30 points — equivalent to a daily quest being 30 days overdue. Skipping divides the boost: 1 skip halves it, 2 skips cut it to a third, etc. After many skips, approaches a 0! quest's score. Never goes negative.

| Importance | 0 skips | 1 skip | 2 skips | 3 skips |
|---|---|---|---|---|
| 0 (—) | 0 | 0 | 0 | 0 |
| 1 (!) | 30 | 15 | 10 | 7.5 |
| 2 (!!) | 60 | 30 | 20 | 15 |
| 3 (!!!) | 90 | 45 | 30 | 22.5 |
| 5 (!!!!!) | 150 | 75 | 50 | 37.5 |

### Overdue Ratio (secondary signal)

Higher = more urgent.

| Quest state | Formula |
|---|---|
| Recurring, has completions | `days_since_completed / cycle_days` (min 1.0) |
| Recurring, never completed | `(days_since_created + cycle_days) / cycle_days` |
| One-off, never completed | `(days_since_created + 9) / 9` |
| Saga step (recurring saga) | `(days_since_activated + saga_cycle_days) / saga_cycle_days` |
| Saga step (one-off saga) | `(days_since_activated + 9) / 9` |

### List Order Bonus

`sort_order / global_max_sort_order` (max 1.0). Uses the full quest list's max, not the candidate pool. Saga steps get 1.0 (treated as top-of-list priority).

### Membership Bonus

+1.0 for regular quests referenced as a criterion in any active campaign. Boolean — does not stack. Saga steps do not get this (they already have 1.0 from list order).

### Balance Bonus

`0.5 × max(0, avg_level - linked_level)` per linked attribute/skill, take the max. Gently nudges quests feeding underleveled attributes/skills.

### Skip Penalty

`skip_count × 0.5`. Base penalty for 0! quests. For important quests, skips are handled by dividing importance (see above). Each "Something Else" or "Run" action adds one skip. Resets at local midnight.

### Reference Values

| Scenario | Score (0 skips) | Score (1 skip) | Score (3 skips) |
|---|---|---|---|
| 0!, just due, bottom of list | ~1.0 | ~0.5 | ~-0.5 |
| 0!, 30 days overdue daily | ~31.0 | ~30.5 | ~29.5 |
| 1!, just due | ~32.0 | ~16.5 | ~8.5 |
| 3!, just due, top of list | ~92.0 | ~46.5 | ~23.5 |
| 5!, just due | ~152.0 | ~76.5 | ~38.5 |

### Time-of-Day Windows

| Window | Hours (local) | Bitmask |
|---|---|---|
| Morning | 4:00 AM – 11:59 AM | 1 |
| Afternoon | 12:00 PM – 4:59 PM | 2 |
| Evening | 5:00 PM – 8:59 PM | 4 |
| Night | 9:00 PM – 3:59 AM | 8 |
| All times | — | 15 (or 0) |

### Days-of-Week Bitmask

Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64. Default 127 = every day.

## Saga Step XP

Saga steps use the parent saga's cycle for XP calculation, not the one-off multiplier.

### Step XP Formula

```
Base XP     = 5 × difficulty_multiplier × cycle_multiplier
cycle_mult  = sqrt(saga.cycle_days)    [recurring saga]
cycle_mult  = 3                         [one-off saga]
Time mult   = f(time_since_step_last_done / saga.cycle_days)
Final XP    = round(Base XP × Time mult)
```

### Step Base XP Examples (Easy difficulty, before time modifier)

| Saga cycle | Cycle mult | Base XP |
|---|---|---|
| One-off | 3.0 | 75 |
| Daily (1d) | 1.0 | 25 |
| Weekly (7d) | 2.6 | 66 |
| Monthly (30d) | 5.5 | 137 |

### Saga Completion Bonus

Awarded when all steps in a run are complete. Based on baseline XP (time mult = 1.0), not actual earned XP — no procrastination reward.

```
bonus = round(0.20 × sum of all steps' baseline XP)
baseline_per_step = 5 × difficulty_mult × cycle_mult
```

Distributed to: character + final step's linked skills/attributes.

## Default Skill-to-Attribute Mapping

| Skill | Attribute |
|---|---|
| Nature | Connection |
| Bureaucracy | Responsibility |
| Language | Knowledge |
| Animal Handling | Responsibility |
| Cooking | Health |
| Community | Connection |
| Cleaning | Pluck |
| Sociality | Connection |
| Logistics | Responsibility |
| Healing | Health |
| Crafting | Pluck |
| Acrobatics | Health |
