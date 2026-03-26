# Step Spec: Phase 3-2 — Lane 1 replaces current quest giver ✅

## Goal

The quest giver filters to trivial quests only (Lane 1 / Castle Duties). Backend gains a lane enum and difficulty filter. Saga steps appear in whichever lane matches their saga's hardest step. The quest giver uses lane1 image and text pools.

---

## Substep 1: Backend — lane enum, difficulty filter, saga lane inference

**Lane enum** (`db.rs`):

```rust
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Lane {
    CastleDuties,
    Adventures,
    RoyalQuests,
}
```

With a method:
```rust
fn includes_difficulty(&self, d: &Difficulty) -> bool {
    match self {
        Lane::CastleDuties => matches!(d, Difficulty::Trivial),
        Lane::Adventures => matches!(d, Difficulty::Easy | Difficulty::Moderate),
        Lane::RoyalQuests => matches!(d, Difficulty::Challenging | Difficulty::Epic),
    }
}
```

**Difficulty ranking helper** for saga inference:

```rust
fn difficulty_rank(d: &Difficulty) -> u8 {
    match d { Trivial => 1, Easy => 2, Moderate => 3, Challenging => 4, Epic => 5 }
}

fn lane_for_difficulty_rank(rank: u8) -> Lane {
    match rank {
        4 | 5 => Lane::RoyalQuests,
        2 | 3 => Lane::Adventures,
        _ => Lane::CastleDuties,
    }
}
```

**Saga lane inference:** `get_active_saga_steps` already loads all steps per saga. Compute the max difficulty rank across all steps and include the inferred `Lane` in the returned tuple. New return type: `Vec<(Quest, String, String, Option<i32>, Lane)>`.

**Filter in `get_quest_scores`:** Add `lane: &Lane` parameter.

- Regular quests: add `lane.includes_difficulty(&q.difficulty)` to the hard filter alongside TOD/DOW
- Saga steps: filter by `saga_lane == *lane`

**Commands:** `get_quest_scores` and `get_next_quest` gain a `lane: String` parameter (deserialized as Lane).

**Tests:**

1. `lane_filter_trivial_only` — Create quests at trivial, easy, and epic. Call `get_quest_scores` with CastleDuties. Verify only trivial returned.
2. `lane_filter_adventures` — Same setup. Adventures lane returns only easy/moderate.
3. `lane_filter_royal` — Royal lane returns only challenging/epic.
4. `saga_lane_inference_from_hardest_step` — Create saga with trivial and epic steps. Verify the saga step only appears in RoyalQuests lane (not CastleDuties).
5. `saga_with_only_easy_steps_in_adventures` — Saga with all easy steps. Verify appears in Adventures lane.

**Testing checkpoint:** `cargo test` passes.

---

## Substep 2: Frontend — quest giver uses Lane 1

**`renderQuestGiver`** now calls `get_next_quest` with `lane: "castle_duties"`.

**`renderQuestGiverWith`** uses `lane1Images` and `lane1Lines` instead of `questGiverImages` and `questGiverLines`.

**`qgDone`**, **`qgQuestNow`**, **`qgSomethingElse`** pass `lane: "castle_duties"` on their `get_next_quest` calls.

**`set_offered_quest`** still works as-is for now (fixed in step 4).

**`get_quest_scores` call** (for debug) passes `lane: "castle_duties"`. The quest list debug view should show scores for all lanes — but for now just showing Lane 1 scores is acceptable (full multi-lane debug comes in step 3).

**Lane header:** Add "Castle Duties" header text above the quest giver content.

**Testing checkpoint:** Build app. Quest giver only shows trivial quests. Easy/hard quests don't appear. Images come from lane1 folder. Flavor text comes from lane1 file. Done/Quest Now/Something Else still work. Timer still works.

---

## NOT in this step

- Lane 2 and Lane 3 UI (step 3)
- Skip exclusion fix (step 4)

## Done When

Quest giver filters to trivial only. Saga steps appear in correct lane. Lane1 images and text used. All existing functionality (Done, Quest Now, Something Else, timer) works within the filtered pool. `cargo test` passes.
