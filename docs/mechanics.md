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

## Time-Elapsed Modifier (Phase 2D — not yet implemented)

A multiplier applied to quest XP based on how long since the quest was last completed, relative to its cycle.

```
r = time_since_last_done / cycle_days

r < 1:  multiplier = 0.1 + 0.9 × √r
r >= 1: multiplier = 1.0 + 0.5 × ln(r)
Floor:  0.1 (you always get something)

Final XP = base_xp × multiplier
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

**Design rationale:** Piecewise formula with square root ramp below cycle (rewards doing things even if early — at half-cycle you earn 74% XP) and logarithmic growth above cycle (motivates overdue quests without making procrastination a strategy). The turn of the curve is at r=1, the target cycle time.
