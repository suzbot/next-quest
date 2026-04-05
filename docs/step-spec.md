# Phase 5D Item 1: Reset Skips Button

**Goal:** Add a Settings button that immediately clears all skip state (skip counts + last-skipped ID).

**Requirements:** [phase-5d-group1-requirements.md](phase-5d-group1-requirements.md)

---

## Design Notes

### Backend

Skip state lives in `AppSkipState` (`src-tauri/src/commands.rs` line 49):

```rust
pub struct SkipStateInner {
    pub skip_counts: HashMap<String, i32>,
    pub reset_date: String,
    pub last_skipped_id: Option<String>,
}
```

All three pieces are in-memory only. A reset clears `skip_counts` and `last_skipped_id`. `reset_date` is used by `skip_quest()` to know when to auto-clear at midnight — we leave it as today (not reset to empty) so automatic midnight handling continues to work.

### Frontend

Settings tab already has a Reset section with three buttons (`Reset Char`, `Reset Quests`, `Reset History`), all gated by a two-click confirmation via `resetWithConfirm()`. The new button goes in the same section but uses a **direct click** (no confirmation) since skip reset is low-stakes — they'd reset at midnight anyway.

Visual feedback: brief text swap on the button ("Reset" → "Done!" for 1 second) so the user knows it fired.

---

## Changes

### 1. Backend: new `reset_skips` Tauri command

In `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn reset_skips(skip_state: State<AppSkipState>) -> Result<(), String> {
    let mut skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    skips.skip_counts.clear();
    skips.last_skipped_id = None;
    Ok(())
}
```

Register in `main.rs` `invoke_handler!` list alongside other skip commands.

### 2. Frontend: Settings button

Add to the Reset section in `ui/index.html`:

```html
<button onclick="resetSkips(this)">Reset Skips</button>
```

JS handler:

```js
async function resetSkips(btn) {
  await invoke("reset_skips");
  // Refresh the quest giver to reflect cleared skip state
  await loadNextQuest();
  // Brief visual confirmation
  const original = btn.textContent;
  btn.textContent = "Done!";
  setTimeout(() => { btn.textContent = original; }, 1000);
}
```

`loadNextQuest()` is the existing function that re-fetches and re-scores quests in all three lanes. Calling it after reset ensures the user immediately sees the effect.

---

## Verification

1. Skip a trivial quest on the Castle Duties lane a few times (click "Something Else" repeatedly)
2. Go to Settings → click "Reset Skips"
3. Button briefly shows "Done!"
4. Return to Next Quest tab — the previously skipped quest is back in rotation, scoring reflects zero skips
5. "Last skipped" exclusion is also cleared (verified by the same quest being eligible again in the next pick)
