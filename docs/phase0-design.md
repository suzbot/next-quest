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
    cycle_days  INTEGER,           -- NULL = one-off quest
    sort_order  INTEGER NOT NULL,  -- higher = more prominent
    active      INTEGER NOT NULL DEFAULT 1,  -- 1 = active, 0 = inactive
    created_at  TEXT NOT NULL       -- ISO 8601 timestamp
);

CREATE TABLE quest_completion (
    id            TEXT PRIMARY KEY,  -- UUID as text
    quest_id      TEXT NOT NULL REFERENCES quest(id),
    completed_at  TEXT NOT NULL      -- ISO 8601 timestamp
);
```

**Notes:**
- SQLite doesn't have native UUID or boolean types — we use TEXT and INTEGER.
- `cycle_days` is nullable: present = recurring, null = one-off.
- `active` is set to 0 when a one-off quest is completed. Keeps the record for
  strikethrough display. Deletion removes the row entirely.
- Timestamps stored as ISO 8601 strings (e.g. "2026-03-12T22:30:00Z").

### Derived Values (computed, not stored)

- **Last completed**: `SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = ?`
- **Is due/refreshed**: `last_completed IS NULL OR last_completed + cycle_days <= now`

## Backend (Rust)

### Tauri Commands

These are the functions the frontend can call. Each one handles its own
database access and returns JSON to the frontend.

| Command | Input | Returns | What it does |
|---|---|---|---|
| `get_quests` | — | List of quests with last_completed and due status | Fetches all active quests + completed one-offs, ordered by sort_order descending |
| `add_quest` | title, cycle_days | The created quest | Creates quest with next available sort_order |
| `update_quest` | id, title?, cycle_days?, sort_order? | The updated quest | Updates provided fields |
| `delete_quest` | id | — | Deletes quest and its completion records |
| `complete_quest` | id | The updated quest with new last_completed | Creates a quest_completion record. If one-off, sets active = 0. |
| `reorder_quests` | list of {id, sort_order} | — | Batch updates sort_order for multiple quests |

### Return Shape

Each quest returned to the frontend looks like:

```json
{
    "id": "uuid-here",
    "title": "Take a shower",
    "cycle_days": 1,
    "sort_order": 10,
    "active": true,
    "created_at": "2026-03-12T22:30:00Z",
    "last_completed": "2026-03-12T08:00:00Z",
    "is_due": false
}
```

`last_completed` and `is_due` are computed by the backend at query time,
not stored.

## Frontend (Vanilla HTML/CSS/JS)

### Layout

Single page with:
- **Quest list**: ordered list of quests
- **Add quest form**: inline at top or bottom of list
- **Edit mode**: inline editing on a quest (click or keyboard to enter edit mode)

### Visual States

Quests render differently based on state:
- **Due/refreshed** (recurring, cycle elapsed or never completed): default/emphasized text
- **Recently completed** (recurring, cycle not elapsed): muted/de-emphasized text
- **Completed one-off**: strikethrough text
- All states just use basic text styling — no graphics, no icons for now

### Keyboard Navigation

- **Tab / Shift+Tab**: Move focus between quests and actions
- **Enter**: Mark focused quest as done / confirm edit
- **Alt+Up / Alt+Down**: Move focused quest up/down in sort order
- **Delete or Backspace**: Delete focused quest (with confirmation)
- **E or Enter on title**: Enter edit mode for focused quest
- **Escape**: Cancel edit
- Specific bindings can be refined during implementation — the principle is
  that every action is keyboard-reachable.

### Data Flow

1. On page load: call `get_quests`, render the list
2. User action (add/edit/delete/complete/reorder): call the appropriate command
3. On command response: re-render the list with returned data

No local state caching in Phase 0 — the backend is the source of truth,
and we re-fetch after every mutation. Simple and correct.

## Dependencies

### Rust (new)

| Crate | What it does | Why we need it |
|---|---|---|
| rusqlite | SQLite database access | Read/write quest data |
| uuid | Generate unique IDs | Quest and completion IDs |

### Frontend (new)

None. Vanilla HTML/CSS/JS.

## What This Design Does NOT Address

| Deferred | Why |
|---|---|
| Database migrations / schema versioning | One table, no changes expected until Phase 0.5 |
| Error handling UI (toasts, alerts) | Bare-text phase, console errors sufficient |
| Offline/sync | Local-only app, always "offline" |
| Performance optimization | List will be <100 items, no optimization needed |
