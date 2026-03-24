# Phase 2G.2: Campaigns and Honors — Requirements (WIP)

**Status:** Draft

**Goal:** Track and reward accomplishing a series of related quests and sagas. Campaigns are user-defined collections of criteria. Meeting all criteria earns a bonus and adds a permanent accomplishment to the Character tab.

---

## User story

> I want to accomplish "Spring Cleaning 2026."
> This is a way to track and reward accomplishing a series of related, already existing quests or sagas.
> For example, the "Spring Cleaning 2026" campaign may consist of "4 completions of Laundry saga, 4 instances of vacuuming, and 2 instances of mopping floors."
> Completing the campaign gives me an XP bonus.
> On the Character tab, in a second column, I see a section called "Accomplishments."
> I see "Completed Spring Cleaning 2026" in the Accomplishments.
> This can later be built on for more complex and dynamic rewards, honors, titles, etc.

---

## 1. Campaign entity

A named collection of criteria that tracks progress toward a larger accomplishment.

| Field | Description |
|---|---|
| Name | User-given title ("Spring Cleaning 2026") |
| Created at | Timestamp — only completions after this count (no retroactive credit) |
| Completed at | Null until all criteria are met, then stamped |

- Campaigns are active until all criteria are met.
- One quest/saga completion can count toward multiple campaigns simultaneously.

## 2. Campaign criteria

Each campaign has one or more criteria. Initial criterion type: "N completions of quest or saga X."

| Field | Description |
|---|---|
| Target type | "quest" or "saga" |
| Target ID | References a specific quest or saga |
| Target name | Looked up from quest/saga at display time. Shows "Deleted quest" or "Deleted saga" if the target has been deleted. An orphaned criterion can never be satisfied — the campaign becomes stuck. The user's recourse is to duplicate the campaign (without the orphaned criterion) and delete the old one. |
| Target count | How many completions are required |
| Current count | Starts at 0, increments on each qualifying completion |

- A qualifying completion is one that occurs after the campaign's created_at.
- Current count only goes up — deleting a completion does not decrement (consistent with XP-only-goes-up).
- No duplicate criteria for the same quest/saga within a campaign.

## 3. Campaign lifecycle

Parallels the quest pattern: campaigns are the trackable thing, accomplishments are the historical record (like completions). Unlike quests (which are editable after creation), saved campaigns have locked criteria — edits are made by duplicating and creating a new version.

**Creation flow:**
1. User clicks "Add Campaign" — an inline creation form appears at the top of the Campaigns tab
2. User enters campaign name
3. User adds criteria one at a time: select quest or saga from dropdown, set target count, click Add. Criteria appear in an editable list below with remove (✕) buttons.
4. User clicks "Save" — campaign locks and immediately begins tracking. The creation form closes and the campaign appears in the list.

**After save, criteria are locked:**
- Criteria cannot be added, removed, or edited
- Campaign name can be renamed at any time (click to edit, same pattern as sagas)
- Campaign can be deleted at any time (from edit mode)

**Active campaigns:**
- Shown in the campaign list with progress bar and criteria count
- Expandable to view criteria progress (read-only)

**Completed campaigns:**
- When all criteria are met: campaign stamps completed_at, dims to inactive styling (like completed one-off quests), accomplishment appears on Character tab
- Completed campaigns remain on the Campaigns tab (dimmed, like inactive one-offs on the quest list)
- Expandable to review criteria (read-only)
- Can be duplicated or deleted

**Duplication flow:**
- Any campaign (active or completed) can be duplicated
- Duplicating opens the creation form pre-filled with the same name (+ " copy") and the same criteria
- User can rename, add/remove/adjust criteria in the form before saving
- Saving creates a new campaign with fresh counts (all 0) and a new created_at

## 4. Progress tracking

- When a quest is completed (`complete_quest`), scan all active campaign criteria for matching quest target_id, increment current_count.
- When a saga run completes (`check_saga_completion` returns completed=true), scan all active campaign criteria for matching saga target_id, increment current_count.
- After any increment, check if all criteria in the campaign are met. If so, stamp completed_at and award bonus XP.

## 5. Campaign completion bonus

When all criteria are met:
- Award bonus XP to character.
- Show a celebration notification (similar to saga completion — gold accent, pulsing).
- Formula TBD in design — similar approach to saga completion bonus (based on constituent criteria).

## 6. Campaigns tab

New tab (between Sagas and Character). Uses the expand/collapse list pattern consistent with sagas.

**Add Campaign button:**
- Opens the inline creation form (name input, criterion builder, save button)
- Also used for duplication (pre-filled)

**Campaign list — collapsed row:**
- Expand toggle (▸/▾), campaign name, progress bar, criteria met count (N/M)
- Active campaigns: normal styling
- Completed campaigns: dimmed/inactive styling, full progress bar, "Done" label

**Campaign list — expanded row:**
- Criteria checklist: each criterion shows ✓ when met, quest/saga name, tally (current/target)
- Read-only — no edit controls for criteria
- Actions: rename (click name), duplicate, delete (from edit mode)

**Creation/duplication form:**
- Campaign name input
- Criterion list with remove (✕) buttons (editable only before save)
- Add criterion: dropdown (quest or saga), count input, Add button
- Save button — locks criteria, activates tracking, closes form. Disabled until at least one criterion is added.

**Keyboard accessible:**
- Arrow keys navigate between campaign rows
- Enter/E to rename

## 7. Accomplishments on Character tab

- New section on the Character tab in a **second column** alongside the existing character/attribute/skill meters: "Accomplishments"
- Shows completed campaign names and completion dates
- Permanent — only removed by explicit deletion of the campaign
- In-progress campaigns are NOT shown here (they live on the Campaigns tab)
- Parallels how completions relate to quests: the accomplishment is the record, the campaign is the source

---

## Implementation notes

- **`target_type` should be a string enum**, not a boolean quest/saga flag. Initial values: `"quest_completions"`, `"saga_completions"`. Future values: `"skill_level"`, `"attr_level"`, `"attr_xp"`, `"skill_xp"`. This avoids a migration when extending criterion types. The `target_count` field generalizes naturally (completion count, level target, XP target). Time-bounded criteria would add an optional `time_window_days` field (null = no time window).

---

## Future extensions (not in initial implementation)

- **Level/XP criterion types** — "reach level 5 in Cleaning" or "earn 500 XP in Health"
- **Time-bounded criteria** — "N completions within 7 days" (no hard deadlines, no punishment for failure)
- **Quest giver campaign nudging** — prioritize quests that advance active campaigns
- **Richer accomplishment display** — titles, honors, badges, visual progression
