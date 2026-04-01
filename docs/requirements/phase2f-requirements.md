# Phase 2F: Logic Enhancement — Requirements (WIP)

**Status:** Complete

**Goal:** Make the quest giver smarter. Quests gain time-of-day, day-of-week, and overdue context. The selector shifts from list-order-first to a scoring system that weighs these factors. Skipped quests sink for the day. The quest list gets manual filtering.

---

## 1. Time-of-day windows

Each quest has a preferred time window:

- **Anytime** (default) — no restriction
- **Morning** — 4:00 AM to 11:59 AM local time
- **Afternoon** — 12:00 PM to 4:59 PM local time
- **Evening** — 5:00 PM to 3:59 AM local time

Behavior:

- The quest giver only suggests quests whose window matches the current local time, or quests set to Anytime.
- **Hard filter** — out-of-window quests are excluded from the candidate pool entirely. No fallback to out-of-window quests in this phase (deferred to 2G advanced logic).
- Editable on the quest add/edit form.

## 2. Day-of-week affinity

Each quest can specify which days of the week it's relevant:

- **Every day** (default) — no restriction
- **Specific days** — any combination of Mon/Tue/Wed/Thu/Fri/Sat/Sun

Behavior:

- The quest giver only suggests quests whose day set includes today (local time).
- **Hard filter** — off-day quests are excluded. No fallback in this phase.
- "Every day" quests are always eligible regardless of day.
- Editable on the quest add/edit form.
- UI meta shows M Tu W Th F St Sn

## 3. Overdue escalation

Due quests are no longer equal — the selector favors quests that are further past their cycle.

- **Overdue ratio**: `days_since_last_completed / cycle_days`
- Higher ratio = higher priority score.
- Quests with no completions use (days since last created+cycle) in place of days since last completed
- One-off quests that are active and never completed are always "due", and use '(days since created+9)/9' (I believe 9 days is consistent with how they are scored for xp)
- Quests that are due but just barely (ratio near 1.0) score lower than quests that are 3x overdue.
- Exact scoring formula to be determined in design, but the principle is: **the longer you've been dodging it, the more it insists.**

## 4. Last-offered freshness

Track when the quest giver last offered each quest. Repeated skips sink a quest's priority for the day.

- **Offered** means the quest was shown by the quest giver (Next Quest tab or Encounter overlay) and the user chose Something Else / Run.
- Each skip increments a daily skip counter for that quest.
- Higher skip count = lower priority score for the day.
- **Daily reset**: skip counters reset at local midnight (or on app launch if the last reset was a prior calendar day).
- **Exhaustion fallback**: if every eligible quest has been skipped, the selector cycles back through skipped quests (highest-scored first, skip penalty still applied so the order is stable). The app always has something to give you.

## 5. Quest selector scoring system

The current selector (first due quest in list order, fallback to longest-ago-completed) is replaced by a scoring system.

**Candidate pool:**

1. Start with all active quests.
2. Hard-filter: remove quests outside their time-of-day window.
3. Hard-filter: remove quests outside their day-of-week.
4. Separate into **due** and **not-due** pools. Due quests are always preferred over not-due.

**Scoring (due quests):**

- **Overdue weight** — primary factor. Higher overdue ratio = higher score.
- **Freshness penalty** — subtract for each skip today. Skipped quests sink but don't disappear (see exhaustion fallback above).
- **List order tiebreaker** — when scores are close, higher sort_order wins.

**Scoring (not-due fallback):**

- Only reached when no due quests remain in the due candidate pool -- or have all been skipped past.
- Sorted by longest-ago-completed (current behavior), with freshness penalty applied.

**"Something Else" behavior:**

- Advances to the next-highest-scored quest in the pool (no change to the user-facing interaction, just the ordering behind it).

Exact weights and formula to be determined in design. The intent is that overdue-ness is the dominant signal, freshness keeps variety, and list order settles ties.

**Testing**

- Create a debug mode. When it is on, in the quest giver, show the score and the values in the formula that got it there, to help user testing and refinement of the scoring formula

## 6. Manual quest list filtering

The quest list tab gets filter controls so the user can narrow the visible list.

**Filter dimensions:**

- **Attribute** — show only quests linked to a specific attribute (dropdown)
- **Skill** — show only quests linked to a specific skill (dropdown)
- **Due status** — show only quests that are currently due
- **Time of Day**
- **Day of Week**
- Filters are combinable (e.g., "Health quests that are due")

**Behavior:**

- Filters apply to the quest list view only. They do not affect the quest giver's suggestions.
- Filters are session-only — not persisted across app restarts.
- An unlinked quest (no attribute or skill) is hidden when filtering by attribute or skill.
- A clear/reset control removes all active filters.

---

## Out of scope (deferred)

- **Soft preference / fallback relaxation** for time-of-day and day-of-week — 2G
- **Attribute/skill balancing** in the selector — 2G
- **Difficulty ramping** (easy quests first, harder after momentum) — 2G
- **Sagas** — 2G
- **Edit last-done date** — 2G
- **Category tags and context filtering** — Phase 3
