---
name: nq-retro
description: "Post-feature reflection to identify process changes that reduce future friction and context overhead. Typically invoked from /nq-update-docs at the end of a significant feature, or directly by the user for a mid-session check-in."
user-invocable: true
argument-hint: Brief context about what was just completed (optional)
---

## NQ Retro Skill

Perform a reflection on collaboration friction. Invoked in two ways:
- **From `/nq-update-docs`**: offered as an optional final step after docs are updated for a significant feature or phase group completion
- **Directly by user**: when the user requests a mid-session process check-in

The goal is to identify **high-confidence, high-leverage refinements** to
collaboration that are likely to save more context in the future than they
cost to reflect on, suggest, and implement.

Avoid speculative or "nice-to-have" improvements.

---

### Step 0: Solicit User Impressions + History Search

Start with one combined question:
- "Do you have feedback on this session? And do you want previous session history searched for friction?"

**If the user provides feedback:** This is a **full retro**. Their impressions are primary input — integrate them with session analysis and any history search findings in Step 1. The user's observations often identify friction that signal analysis misses.

**If the user says "nothing major" / "no" / similar:** This is a **lite retro**. Do a quick scan of the current session, keep Step 1 brief, and only propose changes in Step 2 if something significant surfaces. Skip the pause-for-alignment between Step 1 and Step 2.

If the user wants history searched, launch a **subagent** to search history autonomously:

```
Agent tool call:
  subagent_type: Explore
  description: "Search session history for friction"
  prompt: |
    Search conversation transcripts for friction signals from recent sessions.

    Use Read, Glob, and Grep tools for all file operations — these are auto-allowed
    and won't interrupt the user with approval prompts. Avoid Bash for searching/reading.

    1. Use Read to check /Users/suzanneerin/.claude/projects/-Users-suzanneerin-projects-nq/memory/last-retro.txt
       for the cutoff timestamp (file may not exist — that's fine, just search recent files).

    2. Use Glob to find transcript files:
       pattern: "*.jsonl"
       path: "/Users/suzanneerin/.claude/projects/-Users-suzanneerin-projects-nq"

       Take the 5 most recent files (Glob returns sorted by modification time).

    3. For each transcript file, use Grep (case insensitive) with output_mode: "content"
       and -C: 2 for surrounding context. Search for this pattern:

       (no, |not what|already|don't forget|we discussed|friction|missed|should have|too strong|too weak|that's not|I meant|confused|why are|wait,|wrong|actually|problem|issue|stuck|churn|note for later|noting for later|retro)

       Focus on matches that appear in user messages (lines containing "type":"user").
       Use head_limit: 50 per file to keep output manageable.

    4. For files with interesting friction signals, use Read with offset/limit to examine
       surrounding context more deeply. Look for patterns:
       - User corrections or pushback
       - Repeated attempts at the same thing
       - Tool failures requiring workarounds
       - User redirections or clarifications
       - Context lost or misunderstood
       - Explicit notes flagged for retro ("note for later", "retro", etc.)

    5. Return a summary of friction points found, with brief context for each.
       Only report friction — not things that went smoothly.
       If a cutoff timestamp was found, note which signals are from after that cutoff.
```

Wait for the search agent to return, then integrate its findings into Step 1.

If the user says no, proceed directly to Step 1 using only the current session's observable signals.

---

### Step 1: Collaboration Assessment

Reflect on observable signals from **this session** (you have full conversation context) and any **history search findings** from Step 0.

Address the following **only if applicable**:

#### 1. Interrupted Actions
- Were any actions interrupted by the user?
- If yes:
  - What action was interrupted?
  - What signal suggested *why* the interruption occurred?
    (e.g., wrong direction, missing constraint, pacing issue)

#### 2. Clarifications Requested
- Did the user request clarifications?
- If yes:
  - What type of clarification was requested?
    (e.g., scope, assumptions, terminology, intent, constraints)

#### 3. Reminders of Prior Context
- Did the user remind you of something already read or provided this session?
- If yes:
  - Where did context slip?
  - How could documents, skills, or memory provide better breadcrumbs so the
    information is used when needed, without adding unnecessary overhead?

#### 4. Human-Caught Bugs or Issues
- Were any bugs, logical gaps, or misalignments caught during user testing checkpoints?
- If yes:
  - What recurring pattern or pitfall caused them?
  - Could a lightweight guardrail have prevented them?

#### 5. Intensive Processes
- Were there any suspiciously high-token or high-touch processes for actions that were expected to be more straightforward? Examples:
  - Internal churning going back and forth over a question without surfacing it
  - Many rounds of investigation for something that should have been findable faster
  - Speculating about causes before gathering evidence
- If yes:
  - Was it merited due to actual complexity, or could different prompt structure or information availability have circumvented the extra effort?

If none of items 1-5 occurred, explicitly state that **no meaningful
collaboration friction was observed**.

#### 6. Value Signals (always assess)
Unlike items 1-5, this applies even when no friction occurred. Observe what choices the user made during the session and name the implicit value behind each:
- When the user selected between options: what did they optimize for?
  (e.g., simplicity, flexibility, explicitness, speed, extensibility)
- When the user redirected an approach: what principle were they enforcing?
- When the user added or removed scope: what trade-off were they making?

For each signal, state: the choice made and the value it expresses. Note if CLAUDE.md's Design Principles section has something similar, but don't force-fit — if it feels like a distinct principle, present it as one.

Carry friction insights (items 1-5) and value signals (item 6) forward.

#### Value Persistence (after identifying value signals)

Values identified in item 6 should be routed to persistent storage. Two categories:

**Design values** (about the product — how features should work, how problems should be solved):
- The principle itself lives as a one-liner in **CLAUDE.md Design Principles**
- A companion **reference memory file** (`value_<name>.md`) accumulates dated examples from retros
- If the value maps to an existing principle, add the example to its companion file
- If accumulated examples suggest the principle should be broadened or narrowed, propose the edit

**Collaboration values** (about how to work together — communication, presentation, decision-making):
- The principle itself lives in **CLAUDE.md Communication Style**
- A companion **reference memory file** (`value_<name>.md`) accumulates dated examples
- Same pattern as design values: examples accumulate, principle evolves

**Personal values** (about the user — aesthetic preferences, interaction style):
- Live in **user memory files** (e.g., `user_aesthetic_values.md`)
- Accumulate examples under named headings within the file

Before proposing a new principle, check existing ones — many new observations are illustrations of existing values, not new values. Read the companion memory files (in `memory/value_*.md`) to see if the pattern is already tracked.

**Full retro: STOP here.** Present the assessment and wait for user reaction before proceeding. The user may correct observations, add missing context, or confirm. Their input shapes what gets proposed — don't formulate proposals from unvalidated analysis.

**Lite retro:** Skip the pause — proceed directly to Step 1.5 and Step 2. Only propose changes if something significant surfaced.

---

### Step 1.5: Read Existing Context (REQUIRED before proposing changes)

Before formulating proposals, read what already exists so proposals build on or strengthen existing content rather than duplicating it:

- **`CLAUDE.md`** — design principles, collaboration norms, pacing rules. Check whether a friction signal maps to an existing principle that could be strengthened rather than a new one.
- **`DATA_MODEL.md`** — entity lifecycle rules. Check if the friction was caused by misunderstanding a lifecycle rule that's already documented.
- **`docs/mechanics.md`** — user-facing mechanics. Check if guidance already exists but wasn't surfaced at the right time.
- **Memory files in `.claude/projects/-Users-suzanneerin-projects-nq/memory/`** — existing feedback memories. Check if the friction is already captured.

The goal is: **strengthen or surface existing content first, create new content only when nothing existing covers the gap.**

---

### Step 2: Process Refinement

A. Assess whether Step 1 insights revealed **clear, recurring, or token/context-heavy friction**
- If yes:
  - Are improvements best addressed by one or more of the following:

    - Updates to CLAUDE.md (collaboration norms, design principles)
    - Updates to DATA_MODEL.md (entity rules, lifecycle)
    - Updates to docs/mechanics.md (user-facing behavior)
    - Updates to memory files (feedback, project context)
    - Updates to docs/tech-debt.md (deferred cleanup)
    - Creation of a new skill

    For each proposed change:
    - Explain **why it is worth the cost**
    - Specify **what future context, effort, or friction it saves**
    - Keep recommendations minimal, concrete, and scoped
    - **Prefer structural enforcement over prose instructions.** If a correction was needed because something was forgotten mid-session, the fix should be a checklist item, a memory file, or a concrete rule — not a paragraph to remember.
    - **Route proposals to the right home:**
      - **Design principles** (how to think about the app/player) → CLAUDE.md Design Principles
      - **Communication norms** (how to present, qualify, escalate) → CLAUDE.md Communication Style
      - **Data model rules** (entity lifecycle, relationships) → DATA_MODEL.md
      - **User-facing mechanics** (how things work for the player) → docs/mechanics.md
      - **Recurring friction patterns** (things to remember across sessions) → memory files
      - **Deferred cleanup** (things that work but could be cleaner) → docs/tech-debt.md
    - **Build on what exists** — if CLAUDE.md or a doc already covers the topic, propose strengthening (new example, broader wording, better placement) rather than creating parallel content. Cite what you found in Step 1.5.
    - **Match the target file's format and density.** Show the exact edit (old text → new text) when modifying existing content.

- If No: changes do not clearly pass the cost-benefit threshold
  - Did the user request the retro? If so, provide the analysis and the reasoning for no change.
  - Was the retro automatic? No output is needed, do not solicit user response.

---

### Step 3: Implement Approved Changes

After presenting proposals, wait for explicit user approval.

- Implement only what the user approves — use Edit/Write tools directly
- Update `memory/last-retro.txt` with current ISO timestamp:
  ```bash
  date -u +"%Y-%m-%dT%H:%M:%SZ" > /Users/suzanneerin/.claude/projects/-Users-suzanneerin-projects-nq/memory/last-retro.txt
  ```

---

## Output Rules

- Be concise and concrete
- Avoid speculative suggestions
- Present all changes as **proposals**, not decisions
- All suggestions require **explicit user approval** before implementation
- Use numbered lists so the user can respond by number
