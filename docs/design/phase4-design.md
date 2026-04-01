# Phase 4: Stats and Feedback — Design

## Overview

Two features: XP stats on the Character tab, and XP distribution display on completion celebrations. Both are read-only — no new tables, no schema changes.

## 1. XP Stats

### Backend

**New function:** `get_xp_stats(conn) -> Result<XpStats, String>`

```rust
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XpStats {
    pub today_xp: i64,
    pub last_day_xp: i64,       // most recent non-zero day before today
    pub avg_xp_per_day: f64,    // across all days with completions, excluding zero days
    pub high_score_xp: i64,     // single best day
    pub all_time_xp: i64,       // character.xp (not derived from history)
}
```

**Computation:**

Query `quest_completion` grouped by local date:
```sql
SELECT date(completed_at, 'localtime') as day, SUM(xp_earned) as daily_xp
FROM quest_completion
GROUP BY date(completed_at, 'localtime')
```

Wait — we can't use SQLite's `'localtime'` modifier reliably (it depends on the SQLite build). Instead, compute in Rust: iterate completions, convert each `completed_at` to local date using the existing `utc_iso_to_local_days` → `local_today_str`-style logic, accumulate XP per local day.

Actually simpler: fetch all completions' `(completed_at, xp_earned)` pairs, group by local date in Rust, then compute the stats. The completion table is small enough for this.

Saga and campaign completion bonuses are now also recorded in `quest_completion` (see section 3 below), so the stats computation just queries `quest_completion` and gets everything.

**`today` boundary:** Use `local_today_days()` (same as quest due dates) for consistency.

**New command:** `get_xp_stats` wrapper. Register in main.rs.

### Frontend — Character tab

Add stats section to `renderCharacterView`. Position: below the character meter, above attributes.

```
Adventurer                        1,247 XP all-time
Lvl 3  [========------] (447/800)

Today: 168 XP    Last: 602 XP    Avg: 387 XP/day    Best: 602 XP

Attributes
  ...
```

Stats displayed inline in a single row. Compact, informational.

**All-time XP:** Show next to character name. Uses `character.xp` from the existing character data (already loaded).

**Stats row:** Fetched via new `get_xp_stats` call added to `loadCharacterView`'s parallel fetches.

## 2. XP Distribution on Celebrations

### Backend

**Change `Completion` struct** to include XP distribution detail:

```rust
pub struct XpAward {
    pub name: String,       // "Character", attribute name, or skill name
    pub xp: i64,
    pub award_type: String, // "character", "attribute", or "skill"
}
```

Add `xp_awards: Vec<XpAward>` to the `Completion` struct returned by `complete_quest`.

**Build the awards list in `complete_quest`:** After `award_xp`, build the list from what was distributed:
- Always: `XpAward { name: character.name, xp: xp_earned, award_type: "character" }`
- Per linked attribute: `XpAward { name: attr.name, xp: xp_earned, award_type: "attribute" }`
- Per linked skill: `XpAward { name: skill.name, xp: xp_earned, award_type: "skill" }`

The quest's linked skill/attribute IDs are already loaded in `complete_quest`. Look up names from the IDs.

### Frontend — celebration display

Replace the single "+25 XP" with a list of awards, each colored by its attribute:

```javascript
function xpAwardsHtml(awards) {
  if (!awards || awards.length <= 1) {
    // Just character XP, show simple format
    return awards?.[0] ? `+${awards[0].xp} XP` : "";
  }
  return awards.map(a => {
    const color = awardColor(a); // attribute color for attrs/skills, default for character
    return `<span style="color: ${color}">+${a.xp} ${a.name}</span>`;
  }).join("  ");
}
```

Color logic: skills use their parent attribute's color. Attributes use their own color. Character uses default text color.

Apply to all five completion paths: quest list, quest giver lanes, timer, saga tab, overlay.

Also apply to saga completion bonus and campaign completion bonus celebrations (these already show "+N bonus XP" — add the distribution detail).

## 3. Bonus XP in Completion History

### Problem

Saga completion bonuses and campaign completion bonuses are awarded to character XP but don't appear in `quest_completion`. This means they're invisible in the history list and missing from daily XP stats.

### Change

When a bonus is awarded, also insert a `quest_completion` record:

**Saga completion** (in `check_saga_completion`):
```rust
// After awarding bonus XP
conn.execute(
    "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at, xp_earned) VALUES (?1, NULL, ?2, ?3, ?4)",
    params![Uuid::new_v4().to_string(), format!("{} complete!", saga.name), now, bonus_xp],
)?;
```

**Campaign completion** (in `check_campaign_progress`):
```rust
// After awarding bonus XP
conn.execute(
    "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at, xp_earned) VALUES (?1, NULL, ?2, ?3, ?4)",
    params![Uuid::new_v4().to_string(), format!("{} complete!", campaign_name), now, bonus_xp],
)?;
```

These use `quest_id: NULL` (same as orphaned completions) and a descriptive title. They show up in:
- History list on Character tab
- Daily XP stats computation
- The "Last Score" and "High Score" calculations

## Implementation Order

1. **Bonus XP in history** — Insert quest_completion records for saga and campaign bonuses. Testing: complete a saga, verify bonus appears in history with descriptive title and correct XP.

2. **XP stats backend + Character tab display** — New `get_xp_stats` function, `XpStats` struct, command. Character tab shows all-time XP and stats row. Testing: see stats on Character tab, verify today's score includes saga/campaign bonuses.

3. **XP distribution on celebrations** — Add `XpAward` struct and `xp_awards` to `Completion`. Update `complete_quest` to build awards list. Frontend `xpAwardsHtml` helper. Update all five completion paths. Testing: complete a quest with linked skills — see colored per-skill XP in the celebration.

### Summary

Three vertical slices. Step 1 fixes the data gap (bonus XP in history). Step 2 adds the stats display. Step 3 enriches celebration feedback.
