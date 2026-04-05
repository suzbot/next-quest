# Phase 5D Item 2: Show/Hide Lanes

**Goal:** Each quest giver lane can be individually collapsed to a themed card. Default on app launch: Lane 1 expanded, Lanes 2 & 3 collapsed. User's toggles stick through the session; everything resets on app relaunch.

**Requirements:** [phase-5d-group1-requirements.md](phase-5d-group1-requirements.md)
**Mockup:** [mockups/lane-collapse-option-2-themed-card.html](mockups/lane-collapse-option-2-themed-card.html)

---

## Design Notes

### State

Frontend-only JS state, no backend, no persistence. A single object tracks the collapsed flag for each lane:

```js
const laneCollapsed = {
  lane1: false,   // Castle Duties — expanded by default
  lane2: true,    // Adventures — collapsed by default
  lane3: true,    // Royal Quests — collapsed by default
};
```

Initialized at app load. User interactions mutate it. Page reload resets to defaults (which is what we want — "new session, new day, fresh start").

### Collapsed card markup

Each collapsed lane reuses the existing `.qg-lane` container and `.qg-lane-header`, with a new `.qg-lane-collapsed-body` replacing `.qg-lane-content`. This keeps the header (lane name) visible so the user always knows which lane is dormant.

```html
<div id="qg-lane1" class="qg-lane">
  <div class="qg-lane-header">
    <span>Castle Duties</span>
    <button class="qg-lane-toggle" onclick="toggleLane('lane1')" title="Collapse">▲</button>
  </div>
  <!-- When expanded: -->
  <div class="qg-lane-content">...quest giver...</div>
  <!-- When collapsed (replaces qg-lane-content): -->
  <div class="qg-lane-collapsed-body" onclick="toggleLane('lane1')">
    <div class="qg-lane-icon">♜</div>
    <div class="qg-lane-prompt-text">Attend to duties</div>
    <div class="qg-lane-prompt-hint">Click when you're ready</div>
  </div>
</div>
```

Icons and prompts per lane:

| Lane | Icon | Prompt |
|---|---|---|
| lane1 (Castle Duties) | ♜ | Attend to duties |
| lane2 (Adventures) | ♞ | Ask around town |
| lane3 (Royal Quests) | ♚ | Approach the throne |

The toggle button in the header shows ▲ when expanded, ▼ when collapsed. Clicking it (or clicking the collapsed body) toggles state.

### Slide transition

Simple CSS `max-height` transition on `.qg-lane-content` and `.qg-lane-collapsed-body`. On collapse, animate the quest giver's max-height from its rendered value down to 0, then swap the DOM to show the collapsed body (which animates in from 0 to its natural height). On expand, the reverse.

To keep it simple without measuring actual heights, use a generous max-height ceiling (e.g., 500px) and rely on `overflow: hidden` to clip. Transition duration 250ms.

### CSS additions

```css
.qg-lane-content,
.qg-lane-collapsed-body {
  overflow: hidden;
  transition: max-height 250ms ease-in-out;
  max-height: 500px;
}

.qg-lane-content.lane-hidden,
.qg-lane-collapsed-body.lane-hidden {
  max-height: 0;
}

.qg-lane-toggle {
  background: none;
  border: none;
  font-family: inherit;
  font-size: 11px;
  color: #444;
  cursor: pointer;
  padding: 0 4px;
}
.qg-lane-toggle:hover { color: #000; }

.qg-lane-collapsed-body {
  padding: 20px 16px;
  text-align: center;
  cursor: pointer;
  background: #b0b0b0;
  border: 1px solid #999;
}
.qg-lane-collapsed-body:hover { background: #bcbcbc; }

.qg-lane-icon {
  font-size: 32px;
  line-height: 1;
  margin-bottom: 10px;
  color: #555;
}
.qg-lane-prompt-text {
  font-size: 16px;
  color: #222;
  font-weight: 700;
  letter-spacing: 1px;
  margin-bottom: 6px;
}
.qg-lane-prompt-hint {
  font-size: 9px;
  color: #666;
  font-style: italic;
}
```

### Existing layout changes

The existing `.qg-lane-header` needs `display: flex; justify-content: space-between; align-items: center;` to put the collapse button on the right. Current header has no flex, so we'll add it.

---

## Changes

### 1. HTML: update lane containers

In `ui/index.html`, find the three lane blocks:

```html
<div id="qg-lane1" class="qg-lane">
  <div class="qg-lane-header">Castle Duties</div>
  <div class="qg-lane-content"></div>
</div>
```

Update each to include the toggle button and collapsed body:

```html
<div id="qg-lane1" class="qg-lane">
  <div class="qg-lane-header">
    <span>Castle Duties</span>
    <button class="qg-lane-toggle" onclick="toggleLane('lane1')" title="Collapse">▲</button>
  </div>
  <div class="qg-lane-content"></div>
  <div class="qg-lane-collapsed-body lane-hidden" onclick="toggleLane('lane1')">
    <div class="qg-lane-icon">♜</div>
    <div class="qg-lane-prompt-text">Attend to duties</div>
    <div class="qg-lane-prompt-hint">Click when you're ready</div>
  </div>
</div>
```

Same pattern for lane2 (♞, "Ask around town", "Adventures") and lane3 (♚, "Approach the throne", "Royal Quests").

### 2. CSS: add new rules

Add the styles above. Update `.qg-lane-header` to flex layout.

### 3. JavaScript: state and toggle

Add near the top of the script block, alongside other frontend state:

```js
const laneCollapsed = {
  lane1: false,
  lane2: true,
  lane3: true,
};
```

Add `toggleLane` function and a helper to apply the current state to the DOM:

```js
function applyLaneState(lane) {
  const container = document.getElementById(`qg-${lane}`);
  const content = container.querySelector(".qg-lane-content");
  const collapsed = container.querySelector(".qg-lane-collapsed-body");
  const toggle = container.querySelector(".qg-lane-toggle");

  if (laneCollapsed[lane]) {
    content.classList.add("lane-hidden");
    collapsed.classList.remove("lane-hidden");
    toggle.textContent = "▼";
    toggle.title = "Expand";
  } else {
    content.classList.remove("lane-hidden");
    collapsed.classList.add("lane-hidden");
    toggle.textContent = "▲";
    toggle.title = "Collapse";
  }
}

function toggleLane(lane) {
  laneCollapsed[lane] = !laneCollapsed[lane];
  applyLaneState(lane);
  // If expanding, refresh the lane's quest in case scoring changed
  if (!laneCollapsed[lane]) {
    const cfg = laneConfig.find(l => l.key === lane.replace("lane", "") || l.key === lane);
    if (cfg) renderLane(cfg, true);
  }
}
```

Wait — I need to verify the `laneConfig` key format. Let me note: the lane ID in the DOM is `qg-lane1`/`qg-lane2`/`qg-lane3`, but `laneConfig` uses a different key. The step spec should use whatever `laneConfig` already uses. **I'll verify this when implementing and adjust the call to `renderLane()` to use the right lookup.**

Call `applyLaneState` for all three lanes once at app startup (after the DOM is ready but before the first `renderQuestGiver`). Also call it at the start of `renderQuestGiver` so collapsed lanes skip the quest-giver render (they'd be hidden anyway, but this avoids wasted calls).

Actually, simpler: `renderLane` can early-return if `laneCollapsed[lane]` is true. That's the cleanest place to guard.

### 4. Initial application

At app startup, after the existing init code:

```js
applyLaneState("lane1");
applyLaneState("lane2");
applyLaneState("lane3");
```

---

## Verification

1. Launch the app. Lane 1 is expanded, Lanes 2 and 3 are collapsed and show their themed cards.
2. Click the ▼ button or the collapsed card body for Lane 2 — it slides open with a smooth transition and shows its quest giver.
3. Click the ▲ button in Lane 1's header — it slides closed and shows the "Attend to duties" card.
4. Complete a quest in Lane 1 while Lane 2 is open — Lane 2 doesn't flicker or reload.
5. Collapse a lane, then re-expand it — the re-expanded lane shows the latest quest (not stale state).
6. Close and reopen the app — defaults return (Lane 1 expanded, Lanes 2 & 3 collapsed), regardless of last session's state.
7. The Encounters overlay still fires independently when enabled — lane collapse state doesn't affect it.
