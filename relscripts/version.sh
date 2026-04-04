#!/bin/bash
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: ./relscripts/version.sh v0.2.0"
  exit 1
fi

VERSION="$1"

# Validate format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must match vX.Y.Z (e.g., v0.2.0)"
  exit 1
fi

# Strip v prefix for file updates
BARE="${VERSION#v}"

echo "==> Bumping version to ${VERSION} (${BARE})"

# Check if already at this version
CURRENT_TAURI=$(grep '"version"' tauri.conf.json 2>/dev/null | head -1 | sed 's/.*"version": *"\([^"]*\)".*/\1/' || echo "")

# tauri.conf.json is inside src-tauri/
CURRENT_TAURI=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*"version": *"\([^"]*\)".*/\1/')
CURRENT_CORE=$(grep '^version' nq-core/Cargo.toml | head -1 | sed 's/version = "\([^"]*\)"/\1/')
CURRENT_TAURI_CARGO=$(grep '^version' src-tauri/Cargo.toml | head -1 | sed 's/version = "\([^"]*\)"/\1/')
CURRENT_CLI=$(grep '^version' src-cli/Cargo.toml | head -1 | sed 's/version = "\([^"]*\)"/\1/')

if [ "$CURRENT_TAURI" = "$BARE" ] && [ "$CURRENT_CORE" = "$BARE" ] && [ "$CURRENT_TAURI_CARGO" = "$BARE" ] && [ "$CURRENT_CLI" = "$BARE" ]; then
  echo "Already at ${VERSION}, skipping commit."
else
  # Update tauri.conf.json
  sed -i '' "s/\"version\": *\"[^\"]*\"/\"version\": \"${BARE}\"/" src-tauri/tauri.conf.json
  echo "  Updated src-tauri/tauri.conf.json"

  # Update Cargo.toml files (only the first version = line in each)
  for f in nq-core/Cargo.toml src-tauri/Cargo.toml src-cli/Cargo.toml; do
    # Use awk to replace only the first occurrence of version = "..."
    awk -v new="$BARE" 'BEGIN{done=0} /^version = "/ && !done {sub(/version = "[^"]*"/, "version = \"" new "\""); done=1} {print}' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    echo "  Updated ${f}"
  done

  git add src-tauri/tauri.conf.json nq-core/Cargo.toml src-tauri/Cargo.toml src-cli/Cargo.toml
  git commit -m "chore: Bump version to ${VERSION}"
  echo "  Committed version bump"
fi

# Create tag if it doesn't exist
if git rev-parse "$VERSION" >/dev/null 2>&1; then
  echo "Tag ${VERSION} already exists, skipping."
else
  git tag -a "$VERSION" -m "Release ${VERSION}"
  echo "  Created tag ${VERSION}"
fi

echo "==> Done: ${VERSION}"
