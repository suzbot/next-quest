# Phase 0.5: "Table Stakes" — Requirements

## Goal

Add character progression so completing quests feels meaningful. XP, levels,
skills, and attributes give the quest list a reason to exist beyond a checkbox.
This brings the app in line with what's already out there (Dominate Life) so
it becomes the app of choice.

## Core Concepts

- **Character** = the player's RPG avatar. One per app, auto-created on first launch.
  Has a name, overall XP, and a level derived from XP.
- **Attribute** = a personal value category (Health, Pluck, Knowledge, Connection,
  Responsibility). Has its own XP and level. Not editable yet — 5 fixed defaults.
- **Skill** = a directional goal (Cooking, Acrobatics, Bureaucracy, etc.). Has its own
  XP and level. Each skill maps to one attribute. Not editable yet — 12 fixed defaults.
- **Difficulty** = how hard a quest is. Required on every quest, default: Easy.
  Affects XP earned on completion.

## New Quest Properties

Every quest gains:
- **Difficulty**: Trivial, Easy, Moderate, Challenging, or Epic. Default: Easy.
- **Linked skills**: Zero or more skills this quest contributes XP to.
- **Linked attributes**: Zero or more attributes this quest contributes XP to.

## XP and Leveling

See [mechanics.md](mechanics.md) for full formulas and tuning values.

On quest completion:
1. Calculate XP: `base(10) * difficulty_multiplier * cycle_multiplier`
2. Award full XP to the character
3. Award full XP to each linked attribute
4. Award full XP to each linked skill

When a skill levels up, its mapped attribute receives a bonus XP bump.

Quests with no links earn only character XP.

## Character View

A separate page/tab from the quest list. Text-only, minimum viable display.

Shows:
- Character name (editable)
- Overall level and XP (with progress to next level)
- All 5 attributes with their levels and XP
- All 12 skills with their levels, XP, and mapped attribute

## Default Attributes

| Attribute |
|---|
| Health |
| Pluck |
| Knowledge |
| Connection |
| Responsibility |

## Default Skills and Attribute Mappings

| Skill | Attribute |
|---|---|
| Nature | Connection |
| Bureaucracy | Responsibility |
| Language | Knowledge |
| Animal Handling | Responsibility |
| Cooking | Health |
| Community | Connection |
| Cleaning | Pluck |
| Sociality | Connection |
| Logistics | Responsibility |
| Healing | Health |
| Crafting | Pluck |
| Acrobatics | Health |

## Quest List Changes

- Add form gains: difficulty dropdown (default: Easy), skill/attribute multi-select
- Edit mode gains: difficulty, skill/attribute links
- Quest rows show difficulty label
- Completion feedback: show XP earned (text, not animated — animation is Phase 2)

## What This Phase Does NOT Include

| Deferred Item | Planned For |
|---|---|
| Add/remove/rename attributes | Phase 2 |
| Add/remove/rename skills | Phase 2 |
| Edit skill-to-attribute mappings | Phase 2 |
| Character class | Phase 2 |
| Time-elapsed XP modifier | Phase 2 |
| Elemental Alignment / Values | Phase 2 |
| Visual XP feedback (animation, sound) | Phase 2 |
| Badges | Phase 2 |
