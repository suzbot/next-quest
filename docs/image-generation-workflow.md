# Quest Giver Image Generation Workflow

## Goal

Create female versions of existing Bard's Tale pixel art quest giver portraits. Source images come from free web-based image generators; a Python/Pillow pipeline handles the mechanical post-processing (crop, flatten, pixelate) and GIMP handles the final hand touch-up.

---

## Palette rules

Every generated image must follow the same palette constraints as the original Bard's Tale Atari ST portraits:

1. **Per-image limit: 16 colors max.** Each original portrait uses at most 16 colors (some as few as 8). Generated images must stay within this limit. This is a hard constraint — it's what makes these look like authentic indexed-color pixel art rather than modern approximations.

2. **Master palette: 172 colors.** The 16 colors for each image must be drawn from the 172-color master palette in `ui/images/sd-references/palette.png`. These are the exact colors the original Bard's Tale artists chose across all Atari ST portraits. Every value sits on the Atari ST's 3-bit-per-channel hardware grid (8 levels per channel: 0, 36, 73, 109, 146, 182, 219, 255).

3. **No off-grid colors.** Any color not in the 172-color palette — or not on the ST's 8-level grid — is wrong. The palette reduction step must map to these exact values, not approximate them.

A visual reference swatch is at `ui/images/sd-references/palette-swatch.png`.

---

## Pipeline overview

Generation and post-processing are two distinct phases:

1. **Generate reference images** using any free web tool (Bing Image Creator, Copilot, etc.). Aim for portraits with the subject near the top of the frame and a simple background.
2. **Run the Pillow pipeline** (two Python scripts) to crop, boost contrast, flatten to the master palette, and pixelate to 112×88.
3. **Hand touch-up in GIMP** to clean remaining artifacts.
4. **Export and drop into the lane folder.**

Why two scripts instead of one: the flatten step runs on the full-resolution image first, so contrast/saturation/palette-swap can do their work with all the source data, and the pixelate step runs last so nearest-neighbor sampling picks from an already-flattened image. Doing it in this order avoids noisy backgrounds and detail loss that showed up when resize happened first.

---

## Step 1: Generate reference images

Use any free web-based image generator that accepts text prompts. What we want:

1. **Bust or upper-body portrait** with the subject's head near the top of the frame (the pipeline's crop anchors to the top edge).
2. **Simple, mostly-solid background.** Busy backgrounds eat palette budget and produce artifacts.
3. **High contrast, saturated colors.** The pipeline will boost these further, but a washed-out source stays washed out.
4. **Save as JPEG or PNG** into the matching lane folder: `ui/images/sd-references/lane1/`, `lane2/`, or `lane3/`.

Generate many, keep many — the pipeline runs on everything in the folder, so it's cheap to evaluate lots of candidates in parallel.

---

## Step 2: Flatten (full-resolution palette swap)

Run the step 1 script from the `ui/images/sd-references/` directory:

```bash
cd ui/images/sd-references
python3 process_refs_v2_step1.py                    # all lanes
python3 process_refs_v2_step1.py lane1              # one lane
python3 process_refs_v2_step1.py lane1 lane2        # multiple lanes
```

For each JPEG/JPG in the target lane folders, this script:

1. **Top-anchored crop** to 1.27:1 aspect ratio (112:88). Vertical crops trim the bottom only, preserving the topline — this keeps heads intact when the subject is near the top of the frame.
2. **Contrast ×1.4, saturation ×1.1** to push the source toward decisive color blocking before palette mapping.
3. **Median filter, radius 2** (5×5 kernel) to kill JPEG noise in flat areas. This is what prevents "solid" backgrounds from fragmenting into near-identical palette entries. Radius 3 was too aggressive — faces looked oversmoothed.
4. **Map every pixel to the 172-color master palette** using squared-RGB nearest-neighbor matching.
5. **Reduce to the top-16 most-used colors** for the image, then remap any non-top-16 pixels to their nearest neighbor within those 16.
6. **Save as PNG** at full source resolution to `<lane>/step1_flattened/`.

Output is a full-res PNG that already uses exactly the 16 colors the final GIF will have. This is the right moment to review — if a background still looks noisy or a face tone has been swapped to the wrong shade, it's cheaper to re-tune the settings here than to fix artifacts after pixelation.

### Tuning knobs (top of the script)

1. `CONTRAST` — currently 1.4. Higher flattens more regions but risks losing fine detail.
2. `SATURATION` — currently 1.1. Higher makes faces more vivid but can push skin tones off toward cartoonish reds.
3. `MEDIAN_RADIUS` — currently 2. Radius 3 was too aggressive (over-smoothed faces); radius 1 was not enough to clean JPEG artifacts in backgrounds.

---

## Step 3: Review step 1 output

Open `lane{1,2,3}/step1_flattened/*.png` and check:

1. **Heads intact** — is the top of the head visible in every image?
2. **Backgrounds flat** — has each "solid" background landed on a single color, or are there still speckles?
3. **Face tones correct** — has any face color been snapped to a background color? This usually means the background is eating too much palette budget.
4. **Details preserved** — are weapon edges, eyes, hair lines still readable at full res?

If any of these fail, tune the knobs in `process_refs_v2_step1.py` and rerun. Common fixes:

1. Background artifacts persist → bump `MEDIAN_RADIUS` to 3, knowing it will smooth faces more.
2. Face tones too flat → lower `SATURATION` or `CONTRAST`.
3. Details mushy → lower `MEDIAN_RADIUS` to 1 and accept some background speckle.
4. Head cropped off → the source image is wrong, regenerate in Step 1 with the subject higher in the frame.

---

## Step 4: Pixelate (nearest-neighbor downscale)

Once step 1 output looks good, run the step 2 script:

```bash
python3 process_refs_v2_step2.py                    # all lanes
python3 process_refs_v2_step2.py lane1              # one lane
```

For each PNG in `<lane>/step1_flattened/`, this script:

1. **Re-enforce the 16-color limit.** Counts distinct colors in the input; if more than 16 (common when you've hand-touched-up a step 1 PNG in GIMP and the touch-up introduced extra shades or anti-aliased brush edges), keeps the 16 most-used and remaps the rest to their nearest surviving color. Files already at or under 16 colors pass through unchanged, and the log prints the input→output count when a reduction happens.
2. **Nearest-neighbor resize** from the full source resolution to 112×88. Because the input is guaranteed to be at 16 colors by this point, every output pixel is a clean sample of an on-palette color — no averaging across palette boundaries.
3. **Save as indexed GIF** at 112×88 with the 16-color palette, to `<lane>/step2_pixelated/<name>.gif`.
4. **Save a 4× preview PNG** (448×352, nearest-neighbor) to the same folder. The preview is what you actually look at to judge the result — it shows individual pixels clearly.

The `.gif` files are the final candidates for the lane folders. The `_4x.png` files are for review only.

---

## Step 5: Review step 2 output, pick winners

Compare the 4× preview PNGs. Pick the 1–2 best candidates per character, per lane. Things to look for at 112×88:

1. **Clear silhouette** — can you read the character's shape?
2. **Background doesn't compete** with the subject.
3. **Face reads as a face** even though it's tiny (clear eye positions, hair shape).
4. **Color blocking feels right** — no color that's out of place with the others.

Reject anything that doesn't read clearly. Faces are tiny at this resolution; silhouette and color blocking matter more than fine detail.

---

## Step 6: Hand touch-up in GIMP

Open the winning GIF in GIMP for pixel-level cleanup:

1. Zoom to 800% or higher.
2. Use the pencil tool at 1px, hard edge.
3. Pick colors from the image itself with the color picker — the image is already on the 16-color palette, so any picked color is safe.
4. Fix obvious artifacts: stray speckles in backgrounds, broken outlines, unclear facial features, garbled hands.
5. Compare side-by-side with the original male version for color and silhouette consistency.

The touch-up step exists because the pipeline is statistical — it'll always leave a handful of pixels that a human can obviously improve. Budget a few minutes per image for this, not hours.

---

## Step 7: Export to lane folder

1. `File → Export As...`
2. Name it using the lane convention (e.g., `Female Knight Lane1.gif`).
3. Format: GIF (indexed, already set).
4. Place in the appropriate lane folder: `ui/images/lane1/`, `lane2/`, or `lane3/`.

---

## Step 8: Verify in app

1. Rebuild: `cargo tauri build --debug` — `build.rs` regenerates the image manifest.
2. Run the app and check the quest giver lane — the new image should appear in rotation.

---

## Folder layout

```
ui/images/sd-references/
├── palette.png                       # 172-color master palette (1×172)
├── palette-swatch.png                # visual reference grid
├── process_refs_v2_step1.py          # flatten script
├── process_refs_v2_step2.py          # pixelate script
├── lane1/
│   ├── *.jpeg                        # raw references from web generators
│   ├── step1_flattened/*.png         # full-res, 16-color
│   └── step2_pixelated/
│       ├── *.gif                     # final 112×88 candidates
│       └── *_4x.png                  # 448×352 previews for review
├── lane2/ ...
└── lane3/ ...
```

`ui/images/sd-references/` is in `.gitignore` — none of the reference or intermediate files are checked in. Only the final touched-up GIFs in `ui/images/lane{1,2,3}/` are committed.

---

## What was tried and abandoned

This section exists so future-you doesn't re-explore dead ends.

1. **ComfyUI / Stable Diffusion img2img.** Local SD setup hit too many friction points (model file naming, dependency mismatches, output quality). Free web-based generators gave comparable or better references with zero setup cost.
2. **Single-step resize (pixelate before palette swap).** Doing nearest-neighbor downscale first and then palette-mapping produced two persistent problems: "solid" backgrounds fragmented into multiple palette colors (because one random source pixel per cell straddled palette boundaries), and fine details were lost because nearest-neighbor picks one source pixel per cell regardless of what's around it. Splitting into flatten-first / pixelate-second fixed both.
3. **Centered vertical crop.** Most portraits have the subject's head near the top of the frame, so centered crops cut off the top of the head. Top-anchored crop fixed this without requiring source images to be reframed.
4. **Median filter radius 3.** Flattened backgrounds well but oversmoothed faces. Radius 2 is the sweet spot.
5. **Contrast 1.3 + saturation 1.2.** First pass. Slightly too much saturation, slightly too little contrast. Current values (1.4 / 1.1) are the result of one tuning pass.

---

## Lane mapping

| Lane | Existing images | Female versions to create |
|---|---|---|
| 1 (Castle Duties) | Bard 1 01 (knight), 1 05 (ranger), 1 06 (wizard), 1 26, 1 56, 1 60 | Female knight, female ranger, female wizard, + others |
| 2 (Adventures) | Bard 1 02 (crusader), 1 04 (bard), 1 48, 1 63, 2 04, 2 20 | Female crusader, female bard, + others |
| 3 (Royal Quests) | Bard 1 10 (sorcerer), 1 13, 1 54 (sage), 1 57, 1 61, 2 50 (king) | Female sorcerer, female sage, queen, + others |
