# Phase 3: The Three Quest Givers — Design

## Overview

The Next Quest tab splits from a single quest giver into three stacked lanes, each with its own image pool, flavor text, and difficulty filter. The scoring algorithm is unchanged — it's called three times with different candidate pools.

## 1. Lane definitions

| Lane | Difficulties | Quest giver text file | Quest giver images | Empty state text |
|---|---|---|---|---|
| 1 | Trivial | `ui/text/lane1/quest-giver-lines.txt` | `ui/images/lane1/` | "The walls are secure." |
| 2 | Easy, Moderate | `ui/text/lane2/quest-giver-lines.txt` | `ui/images/lane2/` | "I haven't heard any new rumors." |
| 3 | Challenging, Epic | `ui/text/lane3/quest-giver-lines.txt` | `ui/images/lane3/` | "The realm is at peace." |

Lane names displayed in the UI: "Castle Duties", "Adventures", "Royal Quests".

## 2. Backend — lane-filtered scoring

### Lane enum

```rust
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Lane {
    CastleDuties,  // trivial
    Adventures,    // easy, moderate
    RoyalQuests,   // challenging, epic
}
```

### Difficulty-to-lane mapping

```rust
impl Lane {
    fn includes_difficulty(&self, d: &Difficulty) -> bool {
        match self {
            Lane::CastleDuties => matches!(d, Difficulty::Trivial),
            Lane::Adventures => matches!(d, Difficulty::Easy | Difficulty::Moderate),
            Lane::RoyalQuests => matches!(d, Difficulty::Challenging | Difficulty::Epic),
        }
    }
}
```

### Saga lane inference

A saga's lane is determined by its hardest step:

```rust
fn saga_lane(steps: &[Quest]) -> Lane {
    let max_difficulty = steps.iter().map(|s| difficulty_rank(&s.difficulty)).max().unwrap_or(0);
    match max_difficulty {
        4 | 5 => Lane::RoyalQuests,   // challenging or epic
        2 | 3 => Lane::Adventures,     // easy or moderate
        _ => Lane::CastleDuties,       // trivial
    }
}
```

Where `difficulty_rank` maps Trivial=1, Easy=2, Moderate=3, Challenging=4, Epic=5.

### Filtered scoring

`get_quest_scores` and `get_next_quest` gain a `lane: Lane` parameter. The lane filter is applied as a hard filter alongside time-of-day and day-of-week:

- Regular quests: filter by `lane.includes_difficulty(&quest.difficulty)`
- Saga steps: filter by `saga_lane(saga_steps) == lane`. This requires computing the saga's lane from all its steps (not just the active step). `get_active_saga_steps` already loads all steps per saga to find the active one — add the max difficulty to the returned tuple.

New return type for `get_active_saga_steps`: `Vec<(Quest, String, String, Option<i32>, Lane)>` — adds the saga's inferred lane.

### Commands

- `get_next_quest` gains `lane: String` parameter (deserialized as Lane enum)
- `get_quest_scores` gains `lane: String` parameter
- Existing overlay calls pass `Lane::CastleDuties`

## 3. Skip exclusion fix

### Current (broken) behavior

`set_offered_quest` stores the quest being *shown*. Overlay excludes it. Result: overlay shows a *different* quest from the quest giver.

### Fixed behavior

Rename to `set_last_skipped_quest` / `get_last_skipped_quest`. Store the quest ID that was just *skipped*. Both the quest giver (via `excludeQuestId` on the immediate next call) and the overlay (via reading the last skipped ID) exclude the same quest. Result: both show the same quest.

`SkipStateInner.offered_quest_id` → `SkipStateInner.last_skipped_id`

Set by: `qgSomethingElse` (before calling get_next_quest), and the overlay's "Run" button.

Cleared by: any completion (Done), or when the quest giver re-renders without a skip context (loadAll). This prevents stale exclusion — if you skip A, do something else, then come back, A should be eligible again.

Per-lane: each lane has its own last-skipped. Store as `last_skipped_per_lane: HashMap<String, String>` (lane name → quest ID). The overlay reads Lane 1's last-skipped.

## 4. Frontend — three-lane quest giver

### HTML structure

```html
<div id="quest-giver-view">
  <div id="qg-lane1" class="qg-lane">
    <div class="qg-lane-header">Castle Duties</div>
    <div class="qg-lane-content"></div>
  </div>
  <div id="qg-lane2" class="qg-lane">
    <div class="qg-lane-header">Adventures</div>
    <div class="qg-lane-content"></div>
  </div>
  <div id="qg-lane3" class="qg-lane">
    <div class="qg-lane-header">Royal Quests</div>
    <div class="qg-lane-content"></div>
  </div>
</div>
```

### Per-lane state

Each lane maintains its own:
- `currentImage` — current quest giver image (from lane-specific pool)
- `currentLine` — current flavor text (from lane-specific pool)
- `currentResult` — the ScoredQuest being shown (or null for empty state)

### Rendering

`renderQuestGiver()` calls `renderLane(1)`, `renderLane(2)`, `renderLane(3)`. Each `renderLane` is essentially the current `renderQuestGiverWith` logic scoped to that lane's state, image pool, text pool, and DOM container.

### Timer mode

When Quest Now is started from any lane, all three lane containers are hidden and the timer view renders in their place (same as today). Tab locking works the same. On completion/cancel, all three lanes re-render.

### Lane-specific actions

- `laneDone(lane, questId, difficulty, sagaId)` — same as current `qgDone` but scoped to lane
- `laneQuestNow(lane, questId)` — same as `qgQuestNow`
- `laneSomethingElse(lane, questId)` — skips within that lane, stores last-skipped for that lane

### Empty state rendering

When a lane has no eligible quest, it shows a quest giver image (random from lane pool) and the hardcoded empty state text. Same split layout as a normal quest offer, just with the empty text instead of quest name + buttons.

## 5. Image and text file structure

### Images

```
ui/images/lane1/    — Castle Duties quest giver images (gifs)
ui/images/lane2/    — Adventures quest giver images (gifs)
ui/images/lane3/    — Royal Quests quest giver images (gifs)
ui/images/monsters/ — shared (Quest Now encounters)
ui/images/victory/  — shared
ui/images/defeat/   — shared
```

### Build manifest

`build.rs` adds three new categories:

```rust
("lane1", "lane1"),
("lane2", "lane2"),
("lane3", "lane3"),
```

The old `quest-givers` category can remain for backward compatibility or be removed once images are redistributed.

### Flavor text

```
ui/text/lane1/quest-giver-lines.txt
ui/text/lane2/quest-giver-lines.txt
ui/text/lane3/quest-giver-lines.txt
ui/text/encounter-lines.txt            — shared (Quest Now / overlay)
```

Each lane's text file is loaded separately at startup. One line per entry.

## 6. Overlay

The overlay calls `get_next_quest` with `lane: "castle_duties"` (trivial only). Reads `last_skipped_per_lane["castle_duties"]` as its `excludeQuestId`.

No other changes to overlay behavior.

## 7. Debug scoring

When debug mode is on, each lane shows the score breakdown for its displayed quest (same format as today). The quest list debug view continues showing all quests regardless of lane.

## Implementation order

Each step is a testable vertical slice — backend + frontend + assets as needed, something the user can functionally verify.

1. **Asset structure** — Create all lane directories (`ui/images/lane1/`, `lane2/`, `lane3/`, `ui/text/lane1/`, `lane2/`, `lane3/`). Create placeholder quest-giver-lines.txt per lane. Move existing quest giver images to lane1. Update build.rs manifest to scan lane folders. Commit so the user can start adding images and text in parallel. Testing: build succeeds, lane1 images load in current quest giver.

2. **Lane 1 replaces current quest giver** — Backend: lane enum, difficulty filter on get_quest_scores/get_next_quest, saga lane inference. Frontend: quest giver renders a single lane (Lane 1 / Castle Duties) using the new lane-filtered call and lane1 image/text pools. Testing: quest giver only shows trivial quests. Easy/hard quests don't appear.

3. **Add Lane 2 and Lane 3** — Frontend: quest giver renders all three stacked lanes. Each lane calls get_next_quest with its lane filter. Done/Quest Now/Something Else work per lane. Timer from any lane hides all three. Empty states display when a lane has no quests. Testing: see all three lanes. Complete a trivial from Lane 1, a moderate from Lane 2, start Quest Now from Lane 3. Verify lanes are independent.

4. **Skip exclusion fix + overlay** — Backend: rename offered_quest to last_skipped, per-lane storage. Frontend: Something Else stores last-skipped per lane. Overlay filters to Lane 1 and reads Lane 1's last-skipped as excludeQuestId. Testing: skip a trivial in Lane 1, verify overlay shows the same new quest (not the skipped one). Skip in Lane 2, verify Lane 1 unaffected.

### Summary

Four vertical slices. Step 1 sets up the asset structure so images/text can be created in parallel. Step 2 converts the quest giver to lane-filtered (only trivials show). Step 3 adds the two additional lanes. Step 4 fixes skip/overlay sync.
