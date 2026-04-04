#!/bin/bash
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: ./relscripts/release-all.sh v0.2.0"
  exit 1
fi

VERSION="$1"

# Validate format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must match vX.Y.Z (e.g., v0.2.0)"
  exit 1
fi

echo "========================================="
echo "  Next Quest Release Pipeline: ${VERSION}"
echo "========================================="
echo ""

# Step 1: Tests
echo "==> Step 1/7: Running tests..."
cargo test 2>&1
echo ""

# Step 2: Version bump
echo "==> Step 2/7: Version bump..."
./relscripts/version.sh "$VERSION"
echo ""

# Step 3: Build
echo "==> Step 3/7: Building release..."
./relscripts/build.sh
echo ""

# Step 4: Package
echo "==> Step 4/7: Packaging..."
./relscripts/package.sh "$VERSION"
echo ""

# Step 5: Checksums
echo "==> Step 5/7: Generating checksums..."
./relscripts/checksums.sh
echo ""

# Step 6: Release notes preview
echo "==> Step 6/7: Release notes preview..."
echo ""
echo "-----------------------------------------"
./relscripts/release-notes.sh "$VERSION"
echo "-----------------------------------------"
echo ""

# Approval gate
read -rp "Proceed with release? (y/n) " APPROVE
if [ "$APPROVE" != "y" ]; then
  echo "Release cancelled. Build artifacts remain in dist/."
  exit 0
fi

# Push commits and tag
echo ""
echo "==> Step 7/7: Publishing release..."
git push
git push origin "$VERSION"

./relscripts/release.sh "$VERSION"

echo ""
echo "========================================="
echo "  Release ${VERSION} complete!"
echo "========================================="
echo ""
echo "  Run ./relscripts/install.sh to install locally."
