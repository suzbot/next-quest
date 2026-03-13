# Next Quest — Mechanics

Formulas and tuning values used in the game systems. Updated as we implement
and adjust.

## XP Formula

```
XP = base * difficulty_multiplier * cycle_multiplier
```

**Base XP:** 10

### Difficulty Multipliers

| Difficulty | Multiplier | XP (daily) | Example |
|---|---|---|---|
| Trivial | 1x | 10 | Take meds, drink water |
| Easy | 2x | 20 | Shower, do dishes |
| Moderate | 4x | 40 | Exercise, meal prep |
| Challenging | 7x | 70 | Deep clean, long study session |
| Epic | 12x | 120 | Major project milestone, file taxes |

### Cycle Multipliers

| Cycle | Multiplier | Formula |
|---|---|---|
| Recurring | sqrt(cycle_days) | Daily=1x, 3-day=1.7x, Weekly=2.6x, Monthly=5.5x, Yearly=19.1x |
| One-off | 3x | Fixed (equivalent of ~9-day cycle) |

### XP Distribution

On quest completion, XP is awarded to:
1. **Character** (always) — full XP amount
2. **Linked attributes** — full XP amount per linked attribute
3. **Linked skills** — full XP amount per linked skill

Quests with no links earn only character XP.

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

### Attribute Level Curve (1/5th of character)

Seeds: 60, 100.

| Level | XP for this level | Cumulative XP |
|---|---|---|
| 2 | 60 | 60 |
| 3 | 100 | 160 |
| 4 | 160 | 320 |
| 5 | 260 | 580 |
| 6 | 420 | 1,000 |
| 7 | 680 | 1,680 |
| 8 | 1,100 | 2,780 |

### Skill Level Curve (1/10th of character)

Seeds: 30, 50.

| Level | XP for this level | Cumulative XP |
|---|---|---|
| 2 | 30 | 30 |
| 3 | 50 | 80 |
| 4 | 80 | 160 |
| 5 | 130 | 290 |
| 6 | 210 | 500 |
| 7 | 340 | 840 |
| 8 | 550 | 1,390 |

### Skill Level → Attribute Bump

When a skill levels up, its mapped attribute receives **70 XP** (equivalent of
a challenging daily quest). This makes skill progression a meaningful contributor
to attribute growth, especially at early levels.

### XP Is Permanent

XP is tallied at the time of quest completion and never subtracted. Deleting a
completion removes it from the visible history but does not affect XP totals.

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

## Future Mechanics (not yet implemented)

- **Time-elapsed modifier** (Phase 2): Log-curve modifier based on time since
  last completion. Diminishing returns for rapid repeats, increasing reward as
  time passes, leveling off to avoid rewarding procrastination.
