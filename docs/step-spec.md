# Release Step 4: release-notes.sh + release.sh + release-all.sh + README update

**Goal:** Auto-generate release notes from conventional commits, publish GitHub releases, and tie it all together with a single orchestrator script. Add the unsigned app workaround to the README.

**Design:** [release-pipeline-design.md](release-pipeline-design.md)

---

## What We're Building

Three scripts and a README update.

```
relscripts/
├── release-notes.sh   # generate markdown from conventional commits
├── release.sh         # create GitHub release with assets
└── release-all.sh     # orchestrate the full pipeline
```

---

## Changes

### 1. release-notes.sh

**Usage:** `./relscripts/release-notes.sh v0.2.0 [previous-tag]`

**Flow:**
1. If no previous tag argument, auto-detect: `git describe --tags --abbrev=0 v0.2.0^`
2. If no previous tag exists (initial release), use all commits up to the tag
3. Parse `git log --format="%s" previous..version` for conventional commits
4. Categorize:
   - `feat` → **Features**
   - `fix` → **Bug Fixes**
   - `docs` → **Documentation**
   - everything else → **Other Changes**
5. Parse optional scope: `feat(cli): message` → **cli:** message
6. Output markdown to stdout with non-empty sections only
7. Append GitHub compare link: `https://github.com/{owner}/{repo}/compare/{prev}...{version}`

### 2. release.sh

**Usage:** `./relscripts/release.sh v0.2.0`

**Flow:**
1. Check `gh` CLI is installed (error with install instructions if not)
2. Validate tag exists locally
3. Check tag is pushed to remote; prompt to push if not
4. Generate release notes via `release-notes.sh`
5. Create GitHub release: `gh release create v0.2.0 --title "Release v0.2.0" --notes "$NOTES" dist/archives/*.tar.gz dist/archives/checksums.txt`
6. Print release URL

### 3. release-all.sh

**Usage:** `./relscripts/release-all.sh v0.2.0`

**Flow:**
1. Validate version argument
2. `cargo test` — abort on failure
3. `./relscripts/version.sh $VERSION`
4. `./relscripts/build.sh`
5. `./relscripts/package.sh $VERSION`
6. `./relscripts/checksums.sh`
7. `./relscripts/release-notes.sh $VERSION` — display preview
8. **Prompt:** "Proceed with release? (y/n)"
9. `git push && git push origin $VERSION`
10. `./relscripts/release.sh $VERSION`
11. Print completion banner

### 4. README update

Add a "First Launch" note under the Build and Run section:

> **First launch (macOS):** The app isn't code-signed, so macOS will show a warning that says "Apple could not verify 'Next Quest' is free of malware." If you still want to run it, right-click the app → Open → click Open in the dialog. You only need to do this once.

---

## Verification

1. **release-notes.sh:** `./relscripts/release-notes.sh v0.2.0` — preview release notes from existing commits
2. **release-all.sh:** run the full pipeline, approve at the prompt, verify release appears on GitHub with archive, checksums, and notes
3. **README:** first-launch note is present
