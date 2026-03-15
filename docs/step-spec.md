# Step Spec: Phase 1, Step 6 — "Persistent Settings"

## Goal

Call to Adventure settings (on/off toggle and interval) survive app restarts.
Currently they reset to defaults (off, 20 min) every launch. Store them in
SQLite alongside existing data.

## Scope

### New Table: `settings`

A single-row key-value-ish table, seeded on first launch:

| Field | Type | Default | Description |
|---|---|---|---|
| id | Integer | 1 | Always 1 (single row) |
| cta_enabled | Integer | 0 | Call to Adventure on/off (0/1) |
| cta_interval_minutes | Integer | 20 | Polling interval in minutes |

### Backend Changes

**db.rs:**
- Add `settings` table to `create_tables`
- Seed default row in `seed_data` (if not exists)
- `get_settings_db(conn) -> (bool, u64)` — reads cta_enabled and interval
- `set_settings_db(conn, enabled, interval_minutes)` — updates the row

**commands.rs / main.rs:**
- On app startup, read settings from DB and initialize `TrayStateInner`
  with the persisted values instead of defaults
- `set_cta_interval` writes to DB after updating in-memory state
- `toggle_call_to_adventure` writes to DB after updating in-memory state

### Frontend Changes

None — the frontend already reads from `get_settings` and writes via
`set_cta_interval` / `toggle_call_to_adventure`. The persistence is
invisible to the frontend.

## NOT in this step

- Persisting other settings (future)
- Settings migration strategy (only one setting for now)

## Done When

- Toggle CTA on, quit app, relaunch → CTA is still on
- Change interval to 5 min, quit, relaunch → interval is still 5 min
- Fresh install seeds defaults (off, 20 min)
- All existing tests pass
