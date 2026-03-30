# Step Spec: Phase 4-3 — XP distribution on celebrations ✅

## Goal

Completion celebrations show which skills and attributes received XP, each in its attribute color. Instead of just "+25 XP", show "+25 Character  +25 Cooking  +25 Health".

---

## Substep 1: Backend — add xp_awards to Completion struct

**New struct:**

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct XpAward {
    pub name: String,
    pub xp: i64,
    pub award_type: String, // "character", "attribute", or "skill"
}
```

**Add to `Completion`:** `pub xp_awards: Vec<XpAward>`

**Build in `complete_quest`:** After `award_xp` and before creating the completion record, build the awards list:

1. Always: `XpAward { name: character.name, xp: xp_earned, award_type: "character" }`
2. For each linked attribute: look up name, `XpAward { name, xp: xp_earned, award_type: "attribute" }`
3. For each linked skill: look up name, `XpAward { name, xp: xp_earned, award_type: "skill" }`

The quest's linked skill/attribute IDs are already available — `award_xp` loads them internally. Either refactor `award_xp` to return the names, or look up names separately in `complete_quest`. Looking up separately is simpler (just query attribute/skill name by ID).

**Tests:**

1. `completion_xp_awards_with_links` — Create a quest linked to a skill and attribute. Complete it. Verify `xp_awards` has 3 entries (character + attribute + skill) with correct names and XP.
2. `completion_xp_awards_no_links` — Complete a quest with no links. Verify `xp_awards` has 1 entry (character only).

**Testing checkpoint:** `cargo test` passes.

---

## Substep 2: Frontend — colored XP display at all completion paths

**Helper function** (index.html and overlay.html):

```javascript
function xpAwardsHtml(awards, defaultColor) {
  if (!awards || awards.length === 0) return "";
  if (awards.length === 1) {
    return `<span class="xp-flash" style="color: ${defaultColor}">+${awards[0].xp} XP</span>`;
  }
  return awards.map(a => {
    const color = awardColor(a);
    return `<span class="xp-flash" style="color: ${color}">+${a.xp} ${a.name}</span>`;
  }).join("  ");
}
```

**Color logic (`awardColor`):**
- `award_type: "character"` → default text color (#111)
- `award_type: "attribute"` → attribute's text color from `attrTextColors` (looked up by name)
- `award_type: "skill"` → parent attribute's text color (looked up via `skillAttrMap` by name)

Both index.html and overlay.html have the attribute color infrastructure. The overlay has `skillAttrMap` and `attrTextColors` already loaded.

**Replace single XP display at all five completion paths:**

Currently: `+${completion.xp_earned} XP`
Replace with: `xpAwardsHtml(completion.xpAwards, color)`

Paths to update:
1. `completeQuest()` (quest list) — feedbackHtml XP line
2. `laneDone()` (quest giver lanes) — textHtml XP line
3. `timerDone()` — textHtml XP line
4. `completeSagaStep()` (saga tab) — feedbackHtml XP line
5. `questDone()` (overlay) — textHtml XP line

**Saga/campaign bonus celebrations:** These already show "+N bonus XP". For saga bonus, the distribution goes to character + final step's linked skills/attributes — but the `SagaCompletionResult` doesn't currently include award details. Leave saga/campaign bonus display as-is for now (just the total). The per-skill breakdown is most valuable for regular quest completions.

**Testing checkpoint:** Build app. Complete a quest with linked skills — see "+25 Cooking  +25 Health  +25 Character" in attribute colors. Complete a quest with no links — see "+25 XP" (simple format). Works from quest list, quest giver, timer, saga tab, overlay.

---

## NOT in this step

- Saga/campaign bonus XP distribution detail (future enhancement)

## Done When

Quest completion celebrations show per-skill/attribute XP in attribute colors. All five paths updated. Quests with no links show simple "+N XP". `cargo test` passes.
