# Lane Refactor — Requirements

**Status:** Draft

**Context:** The current lane system groups quests by difficulty tier: Castle Duties = trivial, Adventures = easy, Royal Quests = moderate/challenging/epic. This reflects a "how hard is it?" mental model.

The new system regroups quests by engagement cadence: what you do daily regardless of effort, what's low-effort but not daily, and what's non-trivial and not daily. This better reflects the user's actual decision-making: "what needs to happen today?" is a distinct question from "how hard is this task?"

---

## Goal

Shift lane assignment from difficulty-based to cadence-plus-difficulty-based, so that all daily-recurring quests live together (regardless of difficulty) and non-daily quests split by difficulty.

---

## New Lane Rules

### Lane 1 — Castle Duties

Contains all quests that recur daily, regardless of difficulty.

- **Recurring quests with `cycle_days == 1`**, any difficulty
- **Recurring sagas with `cycle_days == 1`**, regardless of step difficulty (the whole saga lives in Lane 1 if it's a daily saga)

### Lane 2 — Adventures

Contains low-effort quests that aren't daily.

- **Trivial or Easy** difficulty, AND
- Not daily: recurring with `cycle_days != 1`, OR one-off, OR no cycle
- Sagas with hardest-step difficulty = Trivial or Easy AND not a daily recurring saga

### Lane 3 — Royal Quests

Contains harder non-daily quests.

- **Moderate, Challenging, or Epic** difficulty, AND
- Not daily (same cadence rule as Lane 2)
- Sagas with hardest-step difficulty = Moderate+ AND not a daily recurring saga

---

## Decision Table

| Difficulty | One-off | Daily (cycle=1) | Weekly (cycle=7) | Monthly |
|---|---|---|---|---|
| Trivial | Lane 2 | **Lane 1** | Lane 2 | Lane 2 |
| Easy | Lane 2 | **Lane 1** | Lane 2 | Lane 2 |
| Moderate | Lane 3 | **Lane 1** | Lane 3 | Lane 3 |
| Challenging | Lane 3 | **Lane 1** | Lane 3 | Lane 3 |
| Epic | Lane 3 | **Lane 1** | Lane 3 | Lane 3 |

Daily column is the new behavior — everything daily collapses into Lane 1.

---

## Saga Lane Assignment

Sagas are assigned to lanes with a two-step rule:

1. **If the saga is recurring with `cycle_days == 1`** → Lane 1, regardless of step difficulty
2. **Otherwise** → use the hardest step's difficulty to pick Lane 2 (trivial/easy) or Lane 3 (moderate+)

This is consistent with the quest rule: daily cadence takes precedence, difficulty is the tiebreaker for non-daily items.

---

## What Does NOT Change

- **Scoring logic.** Within a lane, quests are still scored by overdue ratio, importance boost, list order, membership, balance, and skip penalty. The lane refactor only affects which quests are eligible for which lane — not how they're ranked.

- **Encounters overlay.** The overlay continues to surface from Lane 1. Under the new rules, this means the overlay may sometimes offer moderate+ daily quests (e.g., daily exercise). This is intentional — if something is daily and important, it should surface, and the user can dismiss with Hide in the Shadows or Run when the timing isn't right.

- **Lane names and themes.** Castle Duties, Adventures, Royal Quests stay. "Castle Duties" still fits semantically — the daily work of keeping the castle running is exactly what goes there, now broadened to include daily duties of any difficulty.

- **Lane collapse behavior.** Default on app launch (Lane 1 expanded, Lanes 2 & 3 collapsed) still applies. The chess piece icons (♜ ♞ ♚) and themed prompts stay.

- **XP calculation.** XP is still `base × difficulty_multiplier × cycle_multiplier`. A moderate daily quest still earns moderate XP and a daily cycle multiplier.

- **Time-of-day and day-of-week filters.** Still apply as hard filters on top of lane membership.

---

## Intentional Trade-offs

1. **The overlay will sometimes surface harder daily quests.** Most daily quests are trivial or easy anyway. The ones that aren't are things that need to happen today — and if they're also high importance, surfacing them is the whole point. The user can always dismiss.

2. **Lane 1 becomes more varied in effort level.** A daily trivial quest and a daily epic quest coexist in Castle Duties. This is a feature, not a bug — they share the "today, again" cadence.

3. **Royal Quests becomes "big stuff that isn't daily"** rather than "big stuff, period." A daily epic quest moves out of Royal Quests into Castle Duties. Royal Quests becomes where you find the non-routine big things (taxes, move project, etc.).

---

## Migration

None needed. Lane assignment is computed at query time from each quest's `cycle_days` and `difficulty` — no stored lane field to update. On first launch after the change, all existing quests automatically redistribute to their new lanes based on their current attributes.

---

## Success Criteria

1. All daily recurring quests appear in Lane 1, regardless of difficulty
2. Trivial and easy non-daily quests appear in Lane 2
3. Moderate+ non-daily quests appear in Lane 3
4. Daily recurring sagas appear in Lane 1 regardless of their steps
5. Non-daily sagas still use hardest-step difficulty to pick Lane 2 vs Lane 3
6. Encounters overlay continues to surface Lane 1 quests
7. All existing tests pass (updated for new rules where needed)
8. Documentation reflects the new rules (mechanics, data model, vision)
