# Step Spec: Phase 3-3 — Add Lane 2 and Lane 3 ✅

## Goal

The Next Quest tab shows three stacked lanes — Castle Duties (trivial), Adventures (easy/fair), Royal Quests (hard/epic). Each lane functions independently with its own images, text, quest state, and Done/Quest Now/Something Else controls.

---

## Substep 1: Frontend — HTML structure and per-lane state

**HTML:** Replace the single `qg-content` div with three lane sections:

```html
<div id="quest-giver-view">
  <div class="qg-lane" id="qg-lane1">
    <div class="qg-lane-header">Castle Duties</div>
    <div class="qg-lane-content"></div>
  </div>
  <div class="qg-lane" id="qg-lane2">
    <div class="qg-lane-header">Adventures</div>
    <div class="qg-lane-content"></div>
  </div>
  <div class="qg-lane" id="qg-lane3">
    <div class="qg-lane-header">Royal Quests</div>
    <div class="qg-lane-content"></div>
  </div>
</div>
```

**Per-lane state:** Replace single `currentQGImage` / `currentQGLine` with per-lane state:

```javascript
const laneState = {
  castle_duties: { image: null, line: null, images: lane1Images, lines: lane1Lines, lane: "castle_duties", label: "Castle Duties", emptyText: "The walls are secure." },
  adventures:    { image: null, line: null, images: lane2Images, lines: lane2Lines, lane: "adventures",    label: "Adventures",    emptyText: "I haven't heard any new rumors." },
  royal_quests:  { image: null, line: null, images: lane3Images, lines: lane3Lines, lane: "royal_quests",  label: "Royal Quests",  emptyText: "The realm is at peace." },
};
```

Image/line arrays must be assigned after `loadPools` completes (pools are loaded async).

**CSS:** Lanes separated by a subtle border or spacing. Each lane is compact — image + text side-by-side, same layout as current quest giver but potentially smaller to fit three.

---

## Substep 2: Frontend — renderLane function

**`renderLane(laneKey, container)`** — renders a single lane into its container div. Essentially the current `renderQuestGiverWith` logic scoped to one lane's state:

1. Call `get_next_quest` with the lane's key
2. If no result: show empty state (quest giver image + empty text)
3. If result: show quest giver image, flavor text, quest name (with saga name if applicable), Done/Quest Now/Something Else buttons

Each button calls lane-specific handlers: `laneDone(laneKey, questId, difficulty, sagaId)`, `laneQuestNow(laneKey, questId)`, `laneSomethingElse(laneKey, questId)`.

**`renderQuestGiver()`** calls `renderLane` for all three lanes.

**Empty state:** Shows a quest giver image (from the lane's pool) with the hardcoded empty text. Same split layout as a normal offer.

---

## Substep 3: Frontend — lane-specific Done/Quest Now/Something Else

**`laneDone(laneKey, questId, difficulty, sagaId)`** — same logic as current `qgDone` but:
- Calls `check_campaign_progress` as today
- Shows completion feedback within the lane's container
- After feedback, re-renders all lanes (via `renderQuestGiver`)

**`laneQuestNow(laneKey, questId)`** — same as current `qgQuestNow`:
- Starts timer
- Locks tabs
- All three lane containers hidden, timer view shown in their place

**`laneSomethingElse(laneKey, questId)`** — same as current `qgSomethingElse` but scoped to the lane:
- Calls `skip_quest`
- Calls `get_next_quest` with the lane's key and `excludeQuestId`
- Re-renders only that lane

**Timer mode:** When timer is active, all lane containers are hidden and the timer view renders in `quest-giver-view` (same as today). On completion/cancel, all lanes re-render.

---

## Substep 4: Debug scoring across all lanes

Update the debug score loading in `loadAll` to fetch scores for all three lanes and merge into `questScoreMap`:

```javascript
if (debugScoring) {
  const [s1, s2, s3] = await Promise.all([
    invoke("get_quest_scores", { lane: "castle_duties" }),
    invoke("get_quest_scores", { lane: "adventures" }),
    invoke("get_quest_scores", { lane: "royal_quests" }),
  ]);
  questScoreMap = {};
  [...s1, ...s2, ...s3].forEach(s => { questScoreMap[s.quest.id] = s; });
}
```

This ensures quest list debug shows scores for all quests regardless of difficulty.

---

## Testing checkpoint

Build app. See all three lanes stacked. Castle Duties shows trivial quests. Adventures shows easy/moderate. Royal Quests shows hard/epic (or empty state if none eligible). Complete a trivial from Lane 1 — XP flash, reloads. Start Quest Now from Lane 2 — timer replaces all lanes, tabs locked. Complete from timer — all lanes re-render. Something Else in Lane 3 — only Lane 3 changes. Enable debug scoring — quest list shows scores for quests at all difficulty levels.

---

## NOT in this step

- Skip exclusion fix / overlay sync (step 4)

## Done When

Three lanes visible and functional. Each lane independent with own images, text, and controls. Timer works from any lane. Debug scoring covers all lanes. Empty states display correctly.
