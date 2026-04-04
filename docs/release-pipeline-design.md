# Release Pipeline — Design

**Status:** Draft
**Requirements:** [release-pipeline-requirements.md](release-pipeline-requirements.md)

---

## Structure

Shell scripts in `relscripts/`, each doing one thing. Same pattern as petri.

```
relscripts/
├── release-all.sh    # orchestrator — runs everything in order
├── version.sh        # bump version in all files, commit, tag
├── build.sh          # release builds of GUI and CLI
├── package.sh        # archive with .app + nq + install instructions
├── checksums.sh      # SHA256 of archives
├── release-notes.sh  # generate markdown from conventional commits
├── release.sh        # create GitHub release with assets
└── install.sh        # copy to /Applications and /usr/local/bin
```

### Output directories

```
dist/
├── Next Quest.app/                              # copied from Tauri build output
├── nq                                           # copied from cargo build output
└── archives/
    ├── next-quest-vX.Y.Z-darwin-arm64.tar.gz    # distributable archive
    └── checksums.txt
```

`dist/` is created by `build.sh` and cleaned before each build. Added to `.gitignore`.

---

## Script Details

### version.sh

**Usage:** `./relscripts/version.sh v0.2.0`

**What it does:**

1. Validate input format (`vX.Y.Z`)
2. Strip `v` prefix → `X.Y.Z`
3. Update version in four files:
   - `tauri.conf.json`: replace `"version": "..."` via `sed`
   - `src-tauri/Cargo.toml`: replace first `version = "..."` via `sed`
   - `nq-core/Cargo.toml`: replace first `version = "..."` via `sed`
   - `src-cli/Cargo.toml`: replace first `version = "..."` via `sed`
4. If any file changed: `git add` the four files, commit with `chore: Bump version to vX.Y.Z`
5. If tag doesn't exist: create annotated tag `vX.Y.Z` with message `Release vX.Y.Z`

**Idempotent:** Skips commit if version already matches. Skips tag if it already exists.

### build.sh

**Usage:** `./relscripts/build.sh`

**What it does:**

1. Clean and create `dist/` directory
2. Run `cargo tauri build` (release mode) — produces `.app` bundle at `target/release/bundle/macos/Next Quest.app`
3. Run `cargo build --release -p nq` — produces CLI binary at `target/release/nq`
4. Copy `Next Quest.app` to `dist/`
5. Copy `nq` to `dist/`

### package.sh

**Usage:** `./relscripts/package.sh v0.2.0`

**What it does:**

1. Determine architecture (`arm64` or `amd64` from `uname -m`, mapping `x86_64` → `amd64`)
2. Create temp directory: `next-quest-vX.Y.Z/`
3. Copy `dist/Next Quest.app` and `dist/nq` into it
4. Generate a small `INSTALL.md` inside with install instructions:
   ```
   # Install
   cp -r "Next Quest.app" /Applications/
   cp nq /usr/local/bin/
   ```
5. Create `dist/archives/` directory
6. `tar czf dist/archives/next-quest-vX.Y.Z-darwin-{arch}.tar.gz next-quest-vX.Y.Z/`
7. Clean up temp directory

### checksums.sh

**Usage:** `./relscripts/checksums.sh`

**What it does:**

1. `cd dist/archives/`
2. `shasum -a 256 *.tar.gz > checksums.txt` (macOS uses `shasum`, not `sha256sum`)

### release-notes.sh

**Usage:** `./relscripts/release-notes.sh v0.2.0 [previous-tag]`

**What it does:**

1. If no previous tag provided, auto-detect: `git describe --tags --abbrev=0 vX.Y.Z^`
2. If no previous tag exists (initial release), use all commits up to tag
3. Parse `git log` between tags for conventional commits
4. Categorize:
   - `feat` → **Features**
   - `fix` → **Bug Fixes**
   - `docs` → **Documentation**
   - everything else → **Other Changes**
5. Parse optional scope: `feat(cli): message` → `**cli:** message`
6. Output markdown with sections (only non-empty sections), plus GitHub compare link
7. Handle initial release (no previous tag) with "Initial Release" header

### release.sh

**Usage:** `./relscripts/release.sh v0.2.0`

**What it does:**

1. Check `gh` CLI is installed
2. Validate tag exists locally
3. Check tag is pushed to remote; prompt to push if not
4. Generate release notes via `release-notes.sh`
5. `gh release create vX.Y.Z --title "Release vX.Y.Z" --notes "$NOTES" dist/archives/*.tar.gz dist/archives/checksums.txt`
6. Output release URL

### release-all.sh

**Usage:** `./relscripts/release-all.sh v0.2.0`

**What it does:**

1. Validate version argument provided
2. Run `cargo test` — abort on failure
3. Call `version.sh $VERSION`
4. Call `build.sh`
5. Call `package.sh $VERSION`
6. Call `checksums.sh`
7. Call `release-notes.sh $VERSION` — display for preview
8. **Prompt:** "Proceed with release? (y/n)"
9. Push tag to remote: `git push origin $VERSION`
10. Call `release.sh $VERSION`
11. Print completion banner with release URL

### install.sh

**Usage:** `./relscripts/install.sh`

**What it does:**

1. Check `dist/Next Quest.app` and `dist/nq` exist (must build first)
2. Copy `dist/Next Quest.app` to `/Applications/Next Quest.app` (replaces if exists)
3. Copy `dist/nq` to `/usr/local/bin/nq` (replaces if exists)
4. Print confirmation with locations

Does not require `sudo` if `/usr/local/bin` has user write permissions (typical with Homebrew). If not, prints a message suggesting `sudo ./relscripts/install.sh` or manual copy.

---

## Version Number Locations

| File | Format | Field |
|---|---|---|
| `tauri.conf.json` | JSON | `"version": "X.Y.Z"` |
| `src-tauri/Cargo.toml` | TOML | `version = "X.Y.Z"` (in `[package]`) |
| `nq-core/Cargo.toml` | TOML | `version = "X.Y.Z"` (in `[package]`) |
| `src-cli/Cargo.toml` | TOML | `version = "X.Y.Z"` (in `[package]`) |

All four must stay in sync. `version.sh` is the only way to change them.

---

## .gitignore Addition

```
dist/
```

---

## Prerequisites

| Tool | Purpose | How to install |
|---|---|---|
| `cargo` | Rust builds | Already installed |
| `cargo-tauri` | Tauri CLI | Already installed |
| `gh` | GitHub releases | `brew install gh` |
| `shasum` | Checksums | Ships with macOS |
| `git`, `tar`, `sed` | Standard tools | Ships with macOS |

`gh` is the only tool that might not be installed. The release script checks for it and gives a clear message if missing.

---

## Implementation Steps

### Step 1: version.sh + build.sh

Version bumping and release builds. The foundation everything else depends on.

**Verify:** Run `version.sh v0.2.0`, check all four files updated, commit and tag created. Run `build.sh`, verify `dist/` contains the `.app` bundle and `nq` binary. Launch `dist/Next Quest.app` to confirm it works.

### Step 2: package.sh + checksums.sh + install.sh

Packaging, checksums, and local installation.

**Verify:** Run `package.sh v0.2.0`, extract the archive and confirm contents. Run `checksums.sh`, verify checksums.txt. Run `install.sh`, launch Next Quest from Spotlight, run `nq list-tags` from a new terminal.

### Step 3: App icon and tray icon

Generate a proper `.icns` file from existing PNGs for the app bundle, and ensure the tray icon is included. Without this, the app shows a generic icon in `/Applications`, Spotlight, and the Dock.

**What's needed:**
- Convert existing PNGs (in `src-tauri/icons/`) to `.icns` format (macOS app icon)
- Configure Tauri to include the `.icns` in the bundle
- Verify the system tray icon is bundled correctly

**Verify:** Rebuild with `build.sh`, install, and confirm the app has a proper icon in `/Applications`, the Dock, and the system tray.

### Step 4: release-notes.sh + release.sh + release-all.sh + README update

Release notes generation, GitHub release, orchestrator, and the unsigned app note in README.

**Verify:** Run `release-notes.sh` against existing tags, review output. Do a full `release-all.sh` run for the first versioned release. Verify release appears on GitHub with archive, checksums, and release notes. Verify README has the first-launch workaround note.

### Step 5: Documentation

Update CLAUDE.md Quick Commands section with release pipeline usage (release-all, install, individual scripts). Update README with first-launch workaround note for unsigned app.

---

## Summary

Eight scripts, five implementation steps. The pipeline takes you from `./relscripts/release-all.sh v0.2.0` to a published GitHub release with a downloadable archive. Locally, `install.sh` puts the app in `/Applications` and `nq` on your PATH.
