#!/bin/bash
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: ./relscripts/release.sh v0.2.0"
  exit 1
fi

VERSION="$1"

# Check gh CLI
if ! command -v gh &>/dev/null; then
  echo "Error: GitHub CLI (gh) is not installed."
  echo "Install with: brew install gh"
  echo "Then authenticate: gh auth login"
  exit 1
fi

# Validate tag exists locally
if ! git rev-parse "$VERSION" >/dev/null 2>&1; then
  echo "Error: Tag ${VERSION} does not exist locally."
  echo "Run ./relscripts/version.sh ${VERSION} first."
  exit 1
fi

# Check tag is pushed to remote
if ! git ls-remote --tags origin "$VERSION" | grep -q "$VERSION"; then
  echo "Tag ${VERSION} is not pushed to remote."
  read -rp "Push now? (y/n) " PUSH
  if [ "$PUSH" = "y" ]; then
    git push origin "$VERSION"
  else
    echo "Aborting — tag must be pushed before creating a release."
    exit 1
  fi
fi

# Check archives exist
if [ ! -d "dist/archives" ] || ! ls dist/archives/*.tar.gz >/dev/null 2>&1; then
  echo "Error: No archives found in dist/archives/. Run build.sh, package.sh, and checksums.sh first."
  exit 1
fi

# Generate release notes
echo "==> Generating release notes..."
NOTES=$(./relscripts/release-notes.sh "$VERSION")

echo ""
echo "$NOTES"
echo ""

# Create GitHub release
echo "==> Creating GitHub release..."
gh release create "$VERSION" \
  --title "Release ${VERSION}" \
  --notes "$NOTES" \
  dist/archives/*.tar.gz dist/archives/checksums.txt

REPO_URL=$(gh repo view --json url -q .url 2>/dev/null || echo "")
echo ""
echo "==> Released: ${REPO_URL}/releases/tag/${VERSION}"
