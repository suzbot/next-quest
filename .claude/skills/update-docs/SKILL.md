---
name: nq-update-docs
description: "Update project documentation after feature implementation. Run at the end of a feature, phase group, or when asked to update docs."
user-invocable: true
argument-hint: Brief summary of what changed (e.g., "Added completion history snapshots and CLI list-history command")
---

## Update Documentation

You are updating project documentation after a feature implementation or bug fix. You will be given a summary of what changed as your argument. If no argument is provided, use the current session's work as context.

### Files to Update

Read each of these files, then apply only the changes warranted by the summary:

| File | Audience | Update Rules |
|------|----------|-------------|
| `README.md` | External readers | Only new user-facing capabilities. One short line per feature. No implementation details. Bug fixes are not README material. |
| `CLAUDE.md` | AI context (always loaded) | Roadmap status only: update "Current phase" line. No feature lists, no completed-work summaries. |
| `DATA_MODEL.md` | Behavioral ground truth | New entities or fields. Lifecycle rule changes. Relationship changes. Keep in sync with what the code actually does. |
| `docs/mechanics.md` | Player behavior | User-visible mechanics only. What does the app do, not how the code works. No function names, no code flow. |
| `docs/cli-guide.md` | CLI users | New commands, new flags, new output fields. Example output for new commands. |
| `VISION.md` | Roadmap | Mark completed items with ~~strikethrough~~ and checkmark. Update phase status. |
| `docs/design/*-design.md` | Phase design | Mark the relevant step's status to "Complete" if applicable. |
| `docs/step-spec.md` | Current step | Clear or update when the step is done. |
| `docs/tech-debt.md` | Deferred cleanup | Add items surfaced during implementation. Remove items that were addressed. |

### Principles

1. **Audience awareness**: Each doc has a different reader. Apply the update rules strictly.
2. **Minimal changes**: Only add/update what the summary warrants. Do not reorganize, rewrite, or "improve" existing content.
3. **Consistency**: Match the style and formatting of the existing content in each file.
4. **No new files**: Only edit existing files listed above. If a file doesn't need changes, skip it.
5. **Don't commit docs alone**: Note which docs were updated so they can be committed alongside code.

### Process

1. Read all files listed above
2. For each file, determine what (if anything) needs updating based on the summary
3. Make edits
4. Present a summary of what was changed in each file (or "no changes needed")

### Retro Trigger

After docs are updated, assess whether this was a **significant feature or phase group completion** (not a bug fix or small tweak). If so, ask:

> "This was a significant milestone. Want to run `/nq-retro` to review how it went?"

If the user says yes, invoke the retro skill. If no, done.
