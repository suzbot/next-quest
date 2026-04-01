# Phase 2H.1: Second Logic Pass — Requirements

**Status:** Draft

**Goal:** Three independent improvements after living with the Phase 2F/2G quest selector: correct stale last-done dates, finer time-of-day control, and smarter quest selection that accounts for importance, saga membership, and skill balance.

---

## 1. Edit last-done date

### User story

> I just added "take vitamins" to my quest list, but I actually took them this morning. I don't want to be bugged about it again today. I don't need XP for the past completion — I just want the due date to be correct.

### Behavior

- A date picker appears next to the "last done" display in the quest list row (not in edit mode — this isn't a "due date" setting, it's a "I last did this on..." correction).
- Picking a date updates the quest's effective `last_completed` timestamp directly.
- No completion record is created. No XP is awarded. The only effect is shifting when the quest next becomes due.
- The date picker should allow selecting any past date. Selecting a future date is not allowed.
- If the quest has never been completed, the last-done display shows nothing — the date picker still appears and sets the initial last-done.
- Clearing the date (if supported) resets to "never completed."

### What it doesn't do

- Does not create a completion record in the history.
- Does not award XP.
- Does not affect any campaign or saga progress.

---

## 2. Evening/Night time-of-day split

### User story

> Some quests are evening activities (cooking dinner, evening walk) and some are night activities (brush teeth, bedtime routine). Right now "Evening" covers 5pm to 4am, which is too broad.

### Behavior

- The current 3-window model (Morning, Afternoon, Evening) becomes a 4-window model:
  - **Morning (MO):** 4am – noon
  - **Afternoon (AF):** noon – 5pm
  - **Evening (EV):** 5pm – 9pm
  - **Night (NT):** 9pm – 4am
- Bitmask values: Morning=1, Afternoon=2, Evening=4, Night=8. "All times" = 15 (was 7).
- The time-of-day multiselect on quests and saga steps gains a fourth option.
- Display labels: MO, AF, EV, NT.
- The quest selector hard-filter uses the same 4-window boundaries.

### Migration

- Existing quests with the old Evening bit (4) set get both Evening (4) and Night (8) set — the old evening window covered both, so no quest should lose eligibility.
- Quests with "all times" (mask=7) migrate to the new "all times" (mask=15).
- Quests with mask=0 (also treated as "all times") remain 0.

---

## 3. Quest selector tuning

Five changes to the scoring algorithm, designed to be layered in one at a time.

### 3a. Saga step scoring uses saga cycle

**Problem:** Saga steps use a hardcoded 9-day base for overdue scoring (`(days + 9) / 9`), regardless of the saga's actual cycle. A daily laundry saga step scores 1.11 after 1 day, while a daily recurring quest scores 2.0. Saga steps get crowded out.

**Change:** Saga steps use their saga's cycle_days: `(days_since_activated + saga_cycle) / saga_cycle`. One-off sagas (cycle_days=null) continue using 9 to match other one-off behavior.

**Effect:** A step in a daily saga scores 2.0 after 1 day (same as a daily quest). A step in a weekly saga scores more gradually. This aligns saga step urgency with the saga's natural pace.

### 3b. Importance field

**Problem:** Some quests matter more than others, but the scoring system only knows about urgency (overdue ratio). A trivial daily quest that's 2 days overdue outscores a critical weekly quest that's 1 day overdue.

**Change:** Add a persistent `importance` field to quests (and saga steps), 0–5 scale. Default: 0 (urgency-only, no importance boost). Incorporated into scoring as an additive boost.

Display: 0 = no indicator, 1–5 = "!" through "!!!!!" in the quest list.

**How it interacts with urgency:** Importance and urgency are independent signals. Importance is a stable weight ("this always matters"), urgency is dynamic ("this is overdue right now"). The exact formula will be determined in design — the key requirement is that a quest at importance 3 should reliably surface above same-urgency quests at importance 0, but a massively overdue quest should still surface regardless of importance.

### 3c. Increase list-order weight

**Problem:** List order bonus is currently `0.01 * sort_order / max_sort_order` — too small to meaningfully influence selection. Users put important quests at the top of their list, but the scoring barely reflects this.

**Change:** Increase the list-order bonus to be comparable to the skip penalty (0.5). The exact value will be determined in design, but the intent is that list position is a meaningful (not dominant) signal.

### 3d. Saga/campaign membership weight

**Problem:** Quests that are part of an active saga or campaign may deserve a slight boost, since completing them advances a larger goal.

**Change:** Add a small scoring bonus for quests that are active saga steps or that appear as criteria in active campaigns. The exact weight will be determined in design. This should be a tiebreaker, not a dominant factor.

### 3e. Attribute/skill balancing

**Problem:** The quest selector has no awareness of character progression. Over time, some attributes and skills fall behind others — whether because fewer quests feed into them, or because those quests come up less often. The scoring system treats a quest linked to a maxed-out attribute the same as one linked to a neglected one.

**Change:** Add a balancing factor that gives a slight boost to quests linked to underleveled attributes or skills. "Underleveled" = lower level or lower XP relative to other attributes/skills. The exact formula will be determined in design, but the intent is gentle variety — not overriding urgency or importance, but nudging underleveled-attribute quests up when everything else is roughly equal.

---

## Implementation order

1. **Edit last-done date** — smallest, most independent change
2. **Evening/Night split** — time model change with migration, touches quest selector hard filter
3. **Quest selector tuning** — layered in sub-steps:
   - 3a: Saga step scoring (saga cycle)
   - 3b: Importance field
   - 3c: List-order weight
   - 3d: Saga/campaign membership weight
   - 3e: Attribute/skill balancing

Each step is independently testable after implementation.

---

## Future extensions (not in this phase)

- Soft preference / fallback relaxation for TOD and DOW filters
- Difficulty ramping (easier quests first, harder after momentum)
- Energy modes (Daily Maintenance vs Boss Fight)
