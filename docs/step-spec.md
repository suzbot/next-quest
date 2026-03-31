# Step Spec: Phase 5A-5 — Debug scoring on saga tab ✅

## Goal

When debug scoring is enabled, expanded saga steps show their score breakdown — same format as the quest list debug display.

---

## Substep 1: Frontend — load scores and display on saga steps

**Score loading:** The saga tab needs scores for active saga steps. On `loadSagas` (or when debug is on and sagas view is visible), fetch scores for all three lanes (same as the quest list does in `loadAll`) and merge into `questScoreMap`. Saga steps are quests — their IDs will be in the scored results.

Since `loadAll` already populates `questScoreMap` when debug is on, and `loadSagas` is called from `showView`, the scores may already be available. Verify: does `loadAll` run before `loadSagas` when switching to the saga tab? If not, ensure scores are fetched.

**Display:** In the saga step row rendering (the expanded step list), if `debugScoring` is true and `questScoreMap[step.id]` exists, show the score breakdown below the step's metadata — same `formatScoreDebug` helper used on the quest list.

```javascript
${debugScoring && questScoreMap[step.id] ? `<div class="quest-debug">${formatScoreDebug(questScoreMap[step.id])}</div>` : ''}
```

Add this inside the step row, after the existing metadata (DOW, TOD, difficulty, last done).

**Note:** Only active saga steps that are currently eligible in a lane will have scores. Completed steps or steps in non-active sagas won't appear in `questScoreMap` — that's correct, they're not in the scoring pool.

---

## Testing checkpoint

Build app. Enable debug scoring in Settings. Go to Saga tab, expand a saga with an active step. The active step (first incomplete in current run) should show its score breakdown. Completed steps in the same saga should NOT show scores (they're not in the pool).

---

## Done When

Active saga steps show debug score breakdown on the saga tab when debug scoring is enabled.
