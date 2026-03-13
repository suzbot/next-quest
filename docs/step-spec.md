# Step Spec: Phase 1, Step 3 — "Backend Foundations for Menu Bar"

## Goal

Move timer state and quest selection logic into the Rust backend so that
both the main window and the upcoming tray menu share one source of truth.
Frontend migrates to use the new commands. No tray yet — this step is
invisible to the user but sets up the architecture.

## Scope

### New State: TimerState

In-memory state (not persisted to database) managed alongside DbState:

```rust
pub struct TimerState {
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,  // Unix millis
}
```

Wrapped in `Mutex<TimerState>` and managed by Tauri.

### New Commands: Timer

| Command | Params | Returns | Behavior |
|---|---|---|---|
| `start_timer` | `quest_id: String` | `TimerInfo` | Looks up quest title, sets timer state |
| `cancel_timer` | — | `()` | Clears timer state |
| `complete_timer` | — | `Completion` | Completes the timed quest, clears timer, returns completion |
| `get_timer_state` | — | `TimerInfo` | Returns current timer state |

`TimerInfo` struct returned to frontend:
```rust
pub struct TimerInfo {
    pub active: bool,
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,
}
```

`complete_timer` calls `db::complete_quest` internally, then clears the
timer. If no timer is active, returns an error.

### New Command: Quest Selection

```rust
pub fn get_next_quest(skip_count: i32) -> Result<Option<Quest>, String>
```

Applies the selection logic currently in frontend JS:
1. Get all active quests (via existing `get_quests`)
2. Filter to due quests (`is_due == true`), in sort order
3. Skip `skip_count` entries (for Something Else cycling), wrapping around
4. If no due quests: fall back to the active quest completed longest ago
5. Return the selected quest, or None if no quests exist

`skip_count` is managed by the frontend (incremented on Something Else,
reset to 0 on Done/load). This keeps the backend stateless for selection
while centralizing the logic.

### Frontend Migration

**Quest Giver view** changes from filtering `cachedQuests` to calling
`get_next_quest`:
- `renderQuestGiver()` calls `invoke("get_next_quest", { skipCount: nextQuestIndex })`
- Done: calls `invoke("complete_quest")`, resets `nextQuestIndex = 0`, reloads
- Something Else: increments `nextQuestIndex`, re-renders
- Quest Now: calls `invoke("start_timer", { questId })`, renders timer mode

**Timer mode** changes:
- `qgQuestNow()` → `invoke("start_timer", { questId })`
- `timerDone()` → `invoke("complete_timer")` (combined stop + complete)
- `cancelTimer()` → `invoke("cancel_timer")`
- Timer display: frontend stores `startedAt` locally after starting,
  computes elapsed from `Date.now() - startedAt`. No polling.

**Quest Now from list** → same: calls `invoke("start_timer")`, switches tab.

**Navigate away cancels** → calls `invoke("cancel_timer")` instead of
clearing local state.

### Timer Cancel on Tab Switch

The existing behavior (navigating away from Quest Giver cancels the timer)
now calls the backend `cancel_timer` command instead of clearing local JS
state.

## NOT in this step

- System tray icon and menu (Step 4)
- Close-to-tray lifecycle (Step 4)
- Event sync between tray and window (Step 4)
- Call to Adventure overlay (Group C)

## Done When

- `start_timer`, `cancel_timer`, `complete_timer`, `get_timer_state` commands work
- `get_next_quest` command returns the correct quest based on skip_count
- Frontend Quest Giver uses `get_next_quest` instead of local filtering
- Frontend timer uses backend state instead of local JS state
- Timer cancel on tab switch calls backend
- Quest Now from list calls backend timer
- All existing tests pass, new backend tests for selection and timer
- App behavior is identical to before from the user's perspective
