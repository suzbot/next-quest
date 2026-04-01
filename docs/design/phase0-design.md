# Phase 0: "The List" — Tech Design

## Overview

A Tauri desktop app with a vanilla HTML/CSS/JS frontend and a Rust backend
using SQLite for persistence. The frontend calls Rust commands to read and
write data. The Rust backend owns all data logic and storage.

## Data Layer

### Database: SQLite

Single file stored in the app's data directory (managed by Tauri — typically
`~/Library/Application Support/com.nextquest.desktop/`). Created automatically
on first launch.

### Schema

```sql
CREATE TABLE quest (
    id          TEXT PRIMARY KEY,   -- UUID as text
    title       TEXT NOT NULL,
    quest_type  TEXT NOT NULL DEFAULT 'recurring',  -- 'recurring' or 'one_off'
    cycle_days  INTEGER,           -- days between refreshes (NULL for one-off)
    sort_order  INTEGER NOT NULL,  -- higher = more prominent
    active      INTEGER NOT NULL DEFAULT 1,  -- 1 = active, 0 = inactive
    created_at  TEXT NOT NULL       -- ISO 8601 timestamp
);

CREATE TABLE quest_completion (
    id            TEXT PRIMARY KEY,  -- UUID as text
    quest_id      TEXT,              -- FK to quest (nullable — quest may be deleted)
    quest_title   TEXT NOT NULL,     -- snapshot of title at completion time
    completed_at  TEXT NOT NULL      -- ISO 8601 timestamp
);
```

**Notes:**
- SQLite doesn't have native UUID or boolean types — we use TEXT and INTEGER.
- `quest_type` is an explicit enum field: `'recurring'` or `'one_off'`. This replaced
  an earlier design where type was inferred from `cycle_days` being null/0 (ambiguous).
- `cycle_days` is always set for recurring quests, always NULL for one-off quests.
- `active` is set to 0 when a one-off quest is completed (deactivated, not deleted).
  The quest template is preserved. Can still be explicitly deleted via [Del].
- `quest_completion.quest_title` is snapshotted at completion time so the record
  is self-contained even if the quest is later deleted or renamed.
- `quest_completion.quest_id` is nullable — if the quest is deleted, completions
  remain with a null foreign key.
- Timestamps stored as ISO 8601 strings (e.g. "2026-03-12T22:30:00Z").

### Key Design Decision: Quests and Completions Are Independent

- Deleting a quest does NOT delete its completions.
- Deleting a completion does NOT affect the quest.
- Completions are visible, first-class rows in the UI — not hidden metadata.

### Derived Values (computed, not stored)

- **Last completed**: `SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = ?`
- **Is due/refreshed**: `last_completed IS NULL OR last_completed + cycle_days <= now`

## Backend (Rust)

### Tauri Commands

| Command | Input | Returns | What it does |
|---|---|---|---|
| `get_quests` | — | List of active quests with last_completed and due status | Fetches active quests ordered by sort_order descending |
| `get_completions` | — | List of completion records | Fetches all completions ordered by completed_at descending |
| `add_quest` | title, quest_type, cycle_days? | The created quest | Creates quest with next available sort_order |
| `update_quest` | id, title?, quest_type?, cycle_days? | The updated quest | Updates provided fields |
| `delete_quest` | id | — | Deletes quest row. Nullifies quest_id on its completions. |
| `complete_quest` | id | The created completion | Creates a quest_completion record with title snapshot. If one-off, sets active = 0. |
| `delete_completion` | id | — | Deletes a single completion record |
| `reorder_quests` | list of {id, sort_order} | — | Batch updates sort_order for active quests |

### Return Shapes

Quest (active list):
```json
{
    "id": "uuid",
    "title": "Take a shower",
    "quest_type": "recurring",
    "cycle_days": 1,
    "sort_order": 10,
    "active": true,
    "created_at": "2026-03-12T22:30:00Z",
    "last_completed": "2026-03-12T08:00:00Z",
    "is_due": false
}
```

Completion (history list):
```json
{
    "id": "uuid",
    "quest_id": "uuid-or-null",
    "quest_title": "Take a shower",
    "completed_at": "2026-03-12T08:00:00Z"
}
```

## Frontend (Vanilla HTML/CSS/JS)

### Layout

Single page with two sections:
- **Add quest form**: at top
- **Active quests**: ordered by sort_order descending. Reorderable.
- **Completion history**: below active quests. Reverse chronological. Not reorderable.

### Visual States

Active quests:
- **Due/refreshed**: emphasized text — cycle elapsed or never completed
- **Recently completed**: muted text — cycle not elapsed, still completable

Completions:
- **Strikethrough text** with timestamp and [Del] button

### Keyboard Navigation

- **Tab / Shift+Tab**: Move focus between quests and actions
- **Enter**: Mark focused quest as done / confirm edit
- **Alt+Up / Alt+Down**: Move focused quest up/down in sort order
- **Delete or Backspace**: Delete focused quest (with confirmation)
- **E or Enter on title**: Enter edit mode for focused quest
- **Escape**: Cancel edit
- Specific bindings refined during implementation.

### Data Flow

1. On page load: call `get_quests` + `get_completions`, render both sections
2. User action: call the appropriate command
3. On response: re-fetch and re-render both sections

No local state caching — the backend is the source of truth.

## Dependencies

### Rust (new in Phase 0)

| Crate | What it does | Why we need it |
|---|---|---|
| rusqlite | SQLite database access | Read/write quest data |
| uuid | Generate unique IDs | Quest and completion IDs |

### Frontend (new)

None. Vanilla HTML/CSS/JS.

## What This Design Does NOT Address

| Deferred | Why |
|---|---|
| Database migrations / schema versioning | Simple schema, changes handled in code until Phase 0.5 |
| Error handling UI (toasts, alerts) | Bare-text phase, console errors sufficient |
| Offline/sync | Local-only app, always "offline" |
| Performance optimization | List will be <100 items, no optimization needed |

---

## Implementation Steps

Each step is a vertical slice — buildable, testable, and committable on its own.

### Step 1: Add and See Quests [COMPLETE]

**Requirements covered:**
- Quest properties: title, cycle *(req: Quest Properties)*
- Add quest with defaults *(req: Actions > Add quest)*
- View quest list ordered by sort order *(req: List Layout)*
- Display title and cycle per quest *(req: List Layout)*
- Data persists across restarts
- Monospace font *(req: Visual Design)*

**What was built:**
- SQLite schema (quest + quest_completion tables)
- `get_quests` and `add_quest` backend commands
- Frontend list rendering and add form
- Tests for data layer + config validation

### Step 2: Complete Quests [COMPLETE]

**Requirements covered:**
- Mark done button *(req: Actions > Mark done)*
- Completion records created with timestamp *(req: Actions > Mark done)*
- Multiple completions per day *(req: Behaviors > Multiple completions)*
- Last completed display *(req: List Layout)*
- Visual states for active quests *(req: Active Quest Visual States)*
- Cycle refresh logic *(req: Behaviors > Cycle refresh)*

**What was built:**
- `complete_quest` backend command
- `get_quests` returns last_completed and is_due
- Frontend visual states (due/de-emphasized/strikethrough)
- Tests for completion logic and is_due calculation

### Step 3: Redesign Completions + Edit/Delete [COMPLETE]

**Requirements covered:**
- Completions as visible, independent records *(req: Core Concepts, Completion Records)*
- Completion history section in list *(req: List Layout)*
- Delete completion individually *(req: Actions > Delete completion)*
- Delete quest without deleting completions *(req: Core Concepts)*
- One-off quest deactivates on completion *(req: Actions > Mark done)*
- Edit quest title, type, and cycle *(req: Actions > Edit quest)*
- Delete quest *(req: Actions > Delete quest)*

**What was built:**
- Explicit `quest_type` enum (recurring/one_off) replacing null/0 cycle_days ambiguity
- `get_completions`, `delete_quest`, `delete_completion`, `update_quest` backend commands
- Schema migrations: quest_title on completions, quest_type on quests, nullable quest_id
- Two-section UI: active quests + completion history
- Inline edit mode with type dropdown (keyboard: E to edit, Enter to save, Esc to cancel)
- 25 tests covering all data operations

### Step 4: Reorder Quests [COMPLETE]

**Requirements covered:**
- Re-sequence active quests via keyboard *(req: Actions > Re-sequence)*
- Drag-and-drop reordering *(req: Actions > Re-sequence)*
- Only active quests reorderable *(req: List Layout)*
- Full keyboard navigation *(req: Interaction Requirements)*

**What was built:**
- `reorder_quests` backend command with batch sort_order updates in a transaction
- Arrow key focus navigation between quest rows
- Alt+Up/Down keyboard reordering with focus follow
- Pointer-event based drag-and-drop (HTML5 drag-and-drop unreliable in Tauri's WKWebView)
- Inactive one-off quests excluded from reordering
- 28 tests total
