# Step Spec: Phase 3-1 — Asset structure for lane images and text

## Goal

Create the directory structure for lane-specific quest giver images and flavor text. Move existing quest giver images to lane1. Update build.rs to generate the manifest with lane entries. User can start adding images and text in parallel with code work.

---

## Substep 1: Directory structure and text files

Create directories:
```
ui/images/lane1/
ui/images/lane2/
ui/images/lane3/
ui/text/lane1/
ui/text/lane2/
ui/text/lane3/
```

Move all files from `ui/images/quest-givers/` to `ui/images/lane1/`.

Create placeholder flavor text files:
- `ui/text/lane1/quest-giver-lines.txt` — copy content from existing `ui/text/quest-giver-lines.txt`
- `ui/text/lane2/quest-giver-lines.txt` — single placeholder line (e.g., "A task awaits beyond the walls...")
- `ui/text/lane3/quest-giver-lines.txt` — single placeholder line (e.g., "The realm has need of you...")

Keep existing `ui/text/quest-giver-lines.txt` and `ui/images/quest-givers/` for now (removed in a later step when the frontend switches over).

---

## Substep 2: Update build.rs manifest

Add three new categories to the manifest scan:

```rust
("lane1", "lane1"),
("lane2", "lane2"),
("lane3", "lane3"),
```

Keep existing `("quest-givers", "quest-givers")` entry for now so current frontend still works.

The manifest.json will now include `lane1`, `lane2`, `lane3` keys alongside the existing `quest-givers` key.

---

## Substep 3: Update frontend loadPools to load lane assets

Add lane-specific image and text arrays:

```javascript
let lane1Images = [];
let lane2Images = [];
let lane3Images = [];
let lane1Lines = [];
let lane2Lines = [];
let lane3Lines = [];
```

In `loadPools`, fetch the three lane text files and read the three lane image arrays from the manifest:

```javascript
lane1Images = manifestText["lane1"] || [];
lane2Images = manifestText["lane2"] || [];
lane3Images = manifestText["lane3"] || [];
```

Keep existing `questGiverImages` loading so current quest giver still works until step 2 switches it.

**Testing checkpoint:** Build app. Current quest giver works unchanged. No visual difference. Console shows lane arrays loading (verify via debug if needed).

---

## NOT in this step

- Lane filtering in scoring (step 2)
- Three-lane quest giver UI (step 3)
- Skip/overlay fixes (step 4)

## Done When

Lane directories exist. Existing quest giver images copied to lane1. Build.rs generates manifest with lane1/lane2/lane3 keys. Frontend loads lane-specific pools at startup. Current quest giver still works unchanged. Build succeeds.
