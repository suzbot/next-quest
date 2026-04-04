# Release Pipeline — Requirements

**Status:** Draft

**Context:** Next Quest currently runs from a debug build launched from the terminal. The goal is an automated release pipeline that produces optimized builds, installs the app like a normal macOS application, and publishes releases to GitHub for portfolio visibility and sharing with peers.

**Model:** Adapted from [petri's relscripts](https://github.com/cerin/petri) — same modular script-per-step pattern, adapted for Tauri + Cargo workspace instead of Go.

---

## Goals

1. **Run like a normal app.** The GUI lives in `/Applications` and can be launched from Spotlight or the Dock. The CLI (`nq`) is on PATH and can be called from any terminal.
2. **Versioned releases.** Semantic versioning, Git tags, version numbers visible in the app/CLI.
3. **One-command release.** A single script runs the full pipeline with a manual approval gate before publishing.
4. **GitHub releases.** Tagged releases with downloadable archive, checksums, and auto-generated release notes. Professional presentation for portfolio/peers.
5. **Unsigned app workaround.** The app is not code-signed. The README documents the macOS Gatekeeper workaround (right-click → Open) for first launch.

---

## Pipeline Steps

### 1. Test

Run the full test suite. Abort if any test fails.

```
cargo test
```

### 2. Version Bump

Update the version string in all four locations:

- `tauri.conf.json` → `"version": "X.Y.Z"`
- `src-tauri/Cargo.toml` → `version = "X.Y.Z"`
- `nq-core/Cargo.toml` → `version = "X.Y.Z"`
- `src-cli/Cargo.toml` → `version = "X.Y.Z"`

Commit the change (`chore: Bump version to vX.Y.Z`) and create an annotated Git tag (`vX.Y.Z`).

Idempotent: if the version already matches, skip the commit. If the tag already exists, skip tag creation.

### 3. Build

Produce optimized release builds of both binaries:

- **GUI:** `cargo tauri build` (release mode) → produces `.app` bundle
- **CLI:** `cargo build --release -p nq` → produces `nq` binary

### 4. Package

Create a distributable archive containing both artifacts:

```
next-quest-vX.Y.Z-darwin-arm64.tar.gz
  └── next-quest-vX.Y.Z/
      ├── Next Quest.app/    (the GUI app bundle)
      ├── nq                 (the CLI binary)
      └── README.md          (install instructions)
```

The archive includes a short README with install instructions (copy `.app` to Applications, copy `nq` to `/usr/local/bin`).

Architecture is `arm64` (Apple Silicon) since that's the build machine. An `amd64` (Intel) build could be added later if needed.

### 5. Checksums

Generate SHA256 checksums for all archives:

```
sha256sum next-quest-vX.Y.Z-darwin-arm64.tar.gz > checksums.txt
```

### 6. Release Notes

Auto-generate release notes from conventional commits between the previous tag and the current tag.

Commit types:
- `feat:` → Features
- `fix:` → Bug Fixes
- `docs:` → Documentation
- Everything else → Other Changes

Scope parsing: `feat(cli): add batch command` → `**cli:** add batch command`

Output: markdown with sections, plus a GitHub compare link.

### 7. Release

Create a GitHub release using `gh release create`:

- Title: `Release vX.Y.Z`
- Body: generated release notes
- Assets: archive(s) + `checksums.txt`

Preconditions: tag exists and is pushed to remote, `gh` CLI authenticated.

---

## Install

Separate from the release pipeline — a convenience for the local machine.

### Install script

Copies the release build artifacts to their permanent locations:

- `Next Quest.app` → `/Applications/Next Quest.app`
- `nq` → `/usr/local/bin/nq`

Replaces previous versions if they exist. Does not require the release pipeline to have run — can install from any successful build.

### Uninstall

Remove `/Applications/Next Quest.app` and `/usr/local/bin/nq`.

---

## Orchestration

### release-all.sh

The primary entry point. Runs all steps in order with a manual approval gate before the GitHub release:

```
./relscripts/release-all.sh v0.2.0
```

1. Run tests — abort on failure
2. Bump version in all files, commit, tag
3. Build release binaries (GUI + CLI)
4. Package into archive
5. Generate checksums
6. Generate and display release notes for preview
7. **Prompt for approval**
8. Push tag to remote
9. Create GitHub release with assets

### Individual scripts

Each step is a standalone script that can be run independently:

```
./relscripts/version.sh v0.2.0
./relscripts/build.sh
./relscripts/package.sh v0.2.0
./relscripts/checksums.sh
./relscripts/release-notes.sh v0.2.0
./relscripts/release.sh v0.2.0
./relscripts/install.sh
```

---

## What's Out of Scope (for now)

- **Cross-platform builds** — macOS only. Linux/Windows builds would require CI (GitHub Actions). Can be added later.
- **Code signing and notarization** — requires Apple Developer account ($99/year). Not justified for current use. README documents the Gatekeeper workaround.
- **DMG packaging** — `.tar.gz` is sufficient. DMG with drag-to-install is polish for later.
- **Auto-update** — Tauri supports built-in auto-update, but requires a server or GitHub releases with a specific manifest format. Future enhancement.
- **Intel (amd64) macOS build** — can be added as a second build target if needed.

---

## README Update

Add a "First Launch" section to the README explaining the unsigned app workaround:

> **First launch (macOS):** The app isn't code-signed, so macOS will show a warning that says "Apple could not verify 'Next Quest' is free of malware." If you still want to run it, right-click the app → Open → click Open in the dialog. You only need to do this once — after that it launches normally.

---

## Dependencies

No new Rust dependencies. The pipeline uses shell scripts and existing tools:

- `cargo` and `cargo-tauri` (already installed)
- `gh` (GitHub CLI — likely already installed)
- `sha256sum` or `shasum` (ships with macOS)
- Standard Unix tools (tar, sed, grep, git)
