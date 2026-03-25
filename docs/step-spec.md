# Step Spec: Phase 2H.1-3b — Importance field ✅

## Goal

Quests and saga steps gain a persistent importance field (0–5). Importance boosts scoring by `importance × 0.4`, making important quests surface more reliably. Displayed as "!" marks in the quest list and saga step list.

---

## Substep 1: Backend — migration, structs, scoring

**Migration:**

```sql
ALTER TABLE quest ADD COLUMN importance INTEGER NOT NULL DEFAULT 0;
```

Detection: check if quest table has `importance` column. Add `importance INTEGER NOT NULL DEFAULT 0` to `create_tables` quest schema.

**Struct changes:**

- `Quest` — add `importance: i32`
- `NewQuest` — add `importance: i32` with `#[serde(default)]`, default 0
- `QuestUpdate` — add `importance: Option<i32>`
- `NewSagaStep` — add `importance: i32` with `#[serde(default)]`, default 0
- `Default` impls — set `importance: 0`

**Query changes** — add `q.importance` to the **end** of each SELECT list to avoid shifting existing column indices:
- `get_quests` — append after `last_completed`, read as index 11
- `get_saga_steps` — append after `last_completed` (index 12 after step_order), read as index 13
- `query_single_quest` — append after `last_completed` (index 12 after saga_id), read as index 13

**Write changes:**
- `add_quest` — include `importance` in INSERT
- `update_quest` — include `importance` in conditional UPDATE SET clauses
- `add_saga_step` — include `importance` in INSERT

**Scoring — due pool (`score_quests_due`):**

```rust
let importance_boost = q.importance as f64 * 0.4;
let score = overdue_ratio + importance_boost - skip_penalty + list_order_bonus;
```

Return tuple grows to include `importance_boost`. Update the tuple type throughout `get_next_quest`.

**Scoring — not-due pool (`score_quests_not_due`):** Same addition.

**Scoring — saga steps in `get_next_quest`:**

```rust
let importance_boost = quest.importance as f64 * 0.4;
let score = overdue_ratio + importance_boost - skip_penalty + list_order_bonus;
```

**`ScoredQuest` struct:** Add `importance_boost: f64`. Update construction in `get_next_quest`.

**Debug display** (frontend, quest giver): Update debug HTML to show importance component:
```
Score: 3.60 (overdue: 2.00 | importance: +1.20 | skips: -0.60 | order: +0.01)
```

**Tests:**

1. `importance_boosts_score` — Create two quests with same overdue. Set one to importance 3. Call `get_next_quest`. Verify the importance-3 quest is selected and its score is higher by ~1.2 (3 × 0.4).
2. `default_importance_is_zero` — Create a quest with defaults. Verify `importance` is 0.
3. `update_quest_importance` — Create a quest, update importance to 4, verify it persists.
4. `saga_step_importance` — Create a saga step with importance 2. Verify it's stored and returned correctly.

**Testing checkpoint:** `cargo test` — all existing + new tests pass.

---

## Substep 2: Frontend — display and editing

**Quest list display:** After the title in `renderQuestRow`, show importance as exclamation marks:

```javascript
const importanceDisplay = q.importance > 0
  ? `<span class="quest-importance">${"!".repeat(q.importance)}</span>`
  : "";
```

Insert after the title span.

**Saga step display:** Same pattern in the saga step row rendering.

**CSS:** `.quest-importance { color: #a85; font-size: 10px; margin-left: 2px; }`

**Quest add form:** Add importance selector — `<select>` with options:
- 0: "—" (default)
- 1–5: "!" through "!!!!!"

**Quest edit mode:** Add importance selector (same options), populated from current value.

**Saga step add form:** Same importance selector.

**Saga step edit mode:** Same importance selector.

**Debug scoring display:** Update the debug HTML in `renderQuestGiverWith` to include importance:
```javascript
Score: ${result.score.toFixed(2)} (overdue: ${result.overdue_ratio.toFixed(2)} | importance: +${result.importance_boost.toFixed(1)} | skips: -${result.skip_penalty.toFixed(1)} | order: +${result.list_order_bonus.toFixed(3)})
```

**Testing checkpoint:** Build app. Create a quest with importance 3 — see "!!!" next to title. Edit to importance 0 — marks disappear. Create saga step with importance — shows in step list. Toggle debug scoring — importance component visible.

---

## NOT in this step

- List-order weight increase (3c)
- Saga/campaign membership bonus (3d)
- Attribute/skill balancing (3e)

## Done When

Both substeps complete. Importance field (0–5) on quests and saga steps. Scoring includes `importance × 0.4`. Displayed as "!" marks. Editable in add/edit forms. Debug scoring shows the component. `cargo test` passes.
