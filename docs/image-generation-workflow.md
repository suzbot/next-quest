# Quest Giver Image Generation Workflow

## Goal

Create female versions of existing Bard's Tale pixel art quest giver portraits using Stable Diffusion (img2img) and GIMP for post-processing.

---

## Step 0: Install ImageMagick + prep all references (bulk)

### Install ImageMagick

```bash
brew install imagemagick
```

Verify: `convert --version` should show ImageMagick 7.x.

### Upscale all existing images to 4x as SD references

```bash
cd ui/images

# Create a working directory for references
mkdir -p sd-references/lane1 sd-references/lane2 sd-references/lane3

# Upscale lane1
for f in lane1/*.gif; do
  convert "$f" -filter point -resize 400% "sd-references/lane1/$(basename "${f%.gif}"-4x.png)"
done

# Upscale lane2
for f in lane2/*.gif; do
  convert "$f" -filter point -resize 400% "sd-references/lane2/$(basename "${f%.gif}"-4x.png)"
done

# Upscale lane3
for f in lane3/*.gif; do
  convert "$f" -filter point -resize 400% "sd-references/lane3/$(basename "${f%.gif}"-4x.png)"
done
```

`-filter point` = nearest-neighbor (keeps pixels crispy). The `sd-references/` folder keeps working files separate from the app's image folders. Add `sd-references/` to `.gitignore`.

### Extract a palette reference image

Pick one representative Atari image and create a palette swatch:

```bash
# Creates a tiny image containing only the unique colors from the source
convert "lane1/Bard 1 01 Atari.gif" -unique-colors palette-atari.png
```

This `palette-atari.png` is used later for batch palette reduction. Do the same for Amiga if needed:

```bash
convert "lane2/Bard 2 04 Amiga.gif" -unique-colors palette-amiga.png
```

---

## Validation: test the post-processing pipeline first

Before installing SD or ImageMagick, validate that the GIMP pipeline produces acceptable results using any free web-based image generator (Bing Image Creator, Copilot, etc.):

1. Generate a test image with a prompt like: "pixel art portrait of a woman knight in silver armor, 16-bit retro RPG style, dark teal background, limited color palette"
2. Save the result (it'll be high-res, wrong palette — that's fine)
3. Open in GIMP
4. `Image → Scale Image` → 112×88, interpolation "None"
5. `Image → Mode → Indexed` → "Generate optimum palette", max 16 colors, Floyd-Steinberg dithering
6. Compare side-by-side with an existing Bard's Tale image

If the result looks like it belongs in the same app, the pipeline works and it's worth setting up SD for better reference-based generation. If it looks wrong even after palette reduction, we'll know to adjust the approach before investing in setup.

---

## One-time setup (after validation)

### 1. Install Stable Diffusion locally

- Install [AUTOMATIC1111 WebUI](https://github.com/AUTOMATIC1111/stable-diffusion-webui) or [ComfyUI](https://github.com/comfyanonymous/ComfyUI)
- Requires: Python 3.10+, ~8GB disk, GPU with 4GB+ VRAM (or CPU mode, slower)
- AUTOMATIC1111 is simpler to start with. Follow the repo's install guide for your OS.

### 2. Download a pixel art model or LoRA

- Base model: SD 1.5 or SDXL
- Recommended LoRA: search CivitAI for "pixel art" LoRAs (e.g., "Pixel Art XL", "16-bit pixel art")
- Place LoRA files in `models/Lora/` inside the SD installation

### 3. Extract the Bard's Tale palette from GIMP

1. Open any existing Atari image in GIMP (e.g., `Bard 1 01 Atari.gif`)
2. `Windows → Dockable Dialogs → Colormap` — this shows the indexed palette
3. `Windows → Dockable Dialogs → Palette Editor`
4. Click the palette menu → `Import Palette → Import from Image`
5. Name it "BardsTale-Atari" and save
6. Repeat for an Amiga image if the palette differs — name it "BardsTale-Amiga"

---

## Per-image workflow

### Step 1: Prepare the reference image

1. Open the existing male character image in GIMP
2. `Image → Mode → RGB` (convert from indexed so SD can read it cleanly)
3. `Image → Scale Image` — upscale to 448×352 (4x) using interpolation "None" (keeps pixel-sharp)
4. Export as PNG to a working folder

### Step 2: Craft the prompt (Cowork)

Use Claude in Cowork mode to generate a detailed SD prompt from the reference image. Cowork can see images, so it can describe composition, pose, color blocking, and details that generic prompts miss.

1. Open a Cowork session and share the upscaled reference image (or the original — Cowork can view either)
2. Ask Cowork to describe what it sees and draft an img2img prompt for a female version
3. Cowork will produce a tailored prompt covering: pose and composition, clothing/armor specifics, color palette and background, what to preserve vs. what to feminize
4. Review and adjust the prompt before feeding it to SD

This replaces the generic prompt templates below, which are kept as a fallback:

<details>
<summary>Fallback prompts (if not using Cowork)</summary>

Base prompt structure:
```
pixel art portrait of a woman, retro 16-bit RPG style, limited color palette,
dark background, bust portrait, [CHARACTER-SPECIFIC DETAILS]
```

Character-specific details:
- Knight (Bard 1 01): "wearing silver plate armor, strong, short hair"
- Ranger (Bard 1 05): "wearing green tunic and hood, holding axe, confident"
- Wizard (Bard 1 06): "wearing blue robes, holding wooden staff, wise elder woman"
- Crusader (Bard 1 02): "wearing blue and white tabard, holding sword upright, noble"
- Bard (Bard 1 04): "playing a lute, wearing colorful clothes, sitting cross-legged"
- Sorcerer (Bard 1 10): "wearing red and blue robes, casting spell, dramatic gesture"
- Sage (Bard 1 54): "white-haired elder woman, wearing magenta robes, wise"
- King → Queen (Bard 2 50): "wearing purple royal robes, crown, sitting on throne, holding scepter"

</details>

### Step 3: Generate in Stable Diffusion (img2img)

1. Open SD WebUI, go to the **img2img** tab
2. Upload the upscaled reference image
3. Use the prompt from Step 2 (Cowork) or the fallback prompts
4. Set negative prompt:

```
smooth, photorealistic, 3d render, high resolution, anti-aliased, blurry,
modern style, anime, cartoon
```

5. Settings:
   - **Denoising strength:** 0.55–0.70 (lower = more faithful to reference, higher = more creative)
   - **Steps:** 30–50
   - **CFG Scale:** 7–9
   - **Output size:** 448×352 (4x the target, we'll downscale later)
   - **Sampler:** Euler a or DPM++ 2M Karras
   - Enable the pixel art LoRA if using one (weight 0.6–0.8)

6. Generate several variations (batch of 4–8). Pick the best candidate.

### Step 4: Downscale in GIMP

1. Open the selected generated image in GIMP
2. `Image → Scale Image` → 112×88 pixels
3. Set interpolation to **"None"** (nearest-neighbor) for crispy pixel look
4. If the result is too messy at this size, try scaling to 224×176 first (2x), clean up, then scale to 112×88

### Step 5: Apply palette

1. `Image → Mode → Indexed...`
2. Select **"Use custom palette"**
3. Choose "BardsTale-Atari" (or Amiga, matching the lane's style)
4. Dithering: try **"Floyd-Steinberg (normal)"** first. If it's too noisy, try **"Positioned"** or **"None"**
5. Compare with the original male version side-by-side — adjust if the palette mapping looks off

### Step 6: Touch up

1. Zoom to 800%+ and review pixel-by-pixel
2. Fix any obvious artifacts: stray pixels, broken outlines, garbled features
3. Use the pencil tool (1px, hard) to touch up — pick colors from the palette
4. Key areas to check: face/hair, armor/clothing edges, hands, background boundary

### Step 7: Export

1. `File → Export As...` → name it to match the lane convention, e.g., `Female Knight Lane1.gif`
2. Format: GIF
3. Place in the appropriate lane folder (`ui/images/lane1/`, `lane2/`, or `lane3/`)

### Step 8: Verify in app

1. Rebuild the app (`cargo tauri build --debug`) — build.rs regenerates the manifest
2. Check the quest giver lane — new image should appear in rotation

---

---

## Bulk post-processing with ImageMagick

After SD generates a batch of candidates, use ImageMagick for the mechanical steps and GIMP for the artistic ones.

### Bulk downscale candidates to target size

```bash
# From wherever SD saved outputs (e.g., outputs/img2img/)
mkdir -p downscaled
for f in *.png; do
  convert "$f" -filter point -resize 112x88! "downscaled/$f"
done
```

`-resize 112x88!` forces exact dimensions (ignoring aspect ratio). `-filter point` keeps it pixel-sharp.

### Bulk palette reduction (rough pass)

```bash
# Apply the extracted palette to all downscaled candidates
cd downscaled
for f in *.png; do
  convert "$f" -remap ../palette-atari.png "palette-${f%.png}.gif"
done
```

This gives you a quick preview of how each candidate looks with the correct palette. Review these visually — pick the best 1–2 per character.

### Final touch-up in GIMP

Open the best candidate GIF in GIMP for manual touch-up:
1. Zoom 800%+
2. Fix artifacts with the pencil tool (1px, pick colors from the palette via color picker)
3. Compare side-by-side with the original male version
4. Export as GIF to the target lane folder

**Use ImageMagick for:** upscaling references, downscaling candidates, rough palette reduction, format conversion.
**Use GIMP for:** palette application with dithering control, visual comparison, manual touch-up.

---

## Tips

- **Start with high denoising (0.65–0.70)** to get a distinctly female character, then dial back if it strays too far from the reference composition.
- **The Atari images are grittier** (fewer colors, more dithering) than the Amiga ones. Match accordingly.
- **Faces are tiny** at 112×88 — don't worry about fine facial detail. Clear silhouette and color blocking matter more.
- **Generate many, pick few.** Expect to generate 8–16 candidates per character and keep 1–2.
- **Save your prompts** for each character so you can regenerate if needed.

---

## Lane mapping

| Lane | Existing images | Female versions to create |
|---|---|---|
| 1 (Castle Duties) | Bard 1 01 (knight), 1 05 (ranger), 1 06 (wizard), 1 26, 1 56, 1 60 | Female knight, female ranger, female wizard, + others |
| 2 (Adventures) | Bard 1 02 (crusader), 1 04 (bard), 1 48, 1 63, 2 04, 2 20 | Female crusader, female bard, + others |
| 3 (Royal Quests) | Bard 1 10 (sorcerer), 1 13, 1 54 (sage), 1 57, 1 61, 2 50 (king) | Female sorcerer, female sage, queen, + others |
