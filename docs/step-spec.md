# Step Spec: Phase 2F-3 — Quest List Filtering

## Goal

Add filter controls above the quest list so the user can narrow the visible quests. Frontend-only — no backend changes.

---

## Implementation

**Filter bar** (`index.html`):
- Row of controls above the quest list, below the add form:
  ```
  [Attribute ▾] [Skill ▾] [Time ▾] [Day ▾] [☐ Due only] [Clear]
  ```

**Filter dropdowns:**
- **Attribute**: "All" + list of user's attributes (populated from `cachedAttributes`)
- **Skill**: "All" + list of user's skills (populated from `cachedSkills`)
- **Time of Day**: "All", Morning, Afternoon, Evening, Anytime
- **Day of Week**: "All" + individual days (Mon–Sun)
- **Due only**: checkbox — when checked, only show quests where `is_due` is true

**Filter logic:**
- On any filter change, re-render the quest list showing only quests that pass all active filters (AND-combined).
- Attribute filter: quest must have this attribute in `attribute_ids`. Unlinked quests hidden when filtering by attribute.
- Skill filter: quest must have this skill in `skill_ids`. Unlinked quests hidden when filtering by skill.
- Time of Day filter: quest's `time_of_day` matches selection. "Anytime" filter shows only quests set to `anytime`. "All" shows everything.
- Day of Week filter: quest's `days_of_week` bitmask includes the selected day, or quest is every day (127). "All" shows everything.
- Due only: quest's `is_due` is true.

**Clear button**: resets all filters to defaults (All / unchecked).

**Session-only**: filter state held in JS variables, not persisted.

**Populate dynamically**: attribute and skill dropdowns rebuilt on each `loadAll` (handles adds/deletes/renames).

---

## NOT in this step

- Filtering the quest giver suggestions (2F-4)
- Scoring system (2F-4)
- Skip tracking (2F-5)

## Done When

Filter bar renders with all controls. Each filter narrows the visible list. Filters combine with AND. Clear resets all. No backend changes.
