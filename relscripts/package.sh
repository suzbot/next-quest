#!/bin/bash
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: ./relscripts/package.sh v0.2.0"
  exit 1
fi

VERSION="$1"

# Validate format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must match vX.Y.Z (e.g., v0.2.0)"
  exit 1
fi

# Determine architecture
ARCH=$(uname -m)
case "$ARCH" in
  arm64)  ARCH="arm64" ;;
  x86_64) ARCH="amd64" ;;
  *)      echo "Error: Unknown architecture ${ARCH}"; exit 1 ;;
esac

echo "==> Packaging ${VERSION} (darwin-${ARCH})"

# Check build artifacts exist
if [ ! -d "dist/Next Quest.app" ]; then
  echo "Error: dist/Next Quest.app not found. Run build.sh first."
  exit 1
fi
if [ ! -f "dist/nq" ]; then
  echo "Error: dist/nq not found. Run build.sh first."
  exit 1
fi

# Create temp directory with archive contents
DIRNAME="next-quest-${VERSION}"
rm -rf "$DIRNAME"
mkdir -p "$DIRNAME"

cp -r "dist/Next Quest.app" "$DIRNAME/"
cp dist/nq "$DIRNAME/"

cat > "$DIRNAME/INSTALL.md" << 'INSTALLEOF'
# Next Quest — Install

## GUI App

```
cp -r "Next Quest.app" /Applications/
```

## CLI

```
cp nq /usr/local/bin/
```

## First Launch (macOS)

The app isn't code-signed, so macOS will show a warning that says
"Apple could not verify 'Next Quest' is free of malware."
If you still want to run it, right-click the app → Open → click Open
in the dialog. You only need to do this once.
INSTALLEOF

# Create archive
mkdir -p dist/archives
ARCHIVE="next-quest-${VERSION}-darwin-${ARCH}.tar.gz"
tar czf "dist/archives/${ARCHIVE}" "$DIRNAME"

# Clean up temp directory
rm -rf "$DIRNAME"

SIZE=$(ls -lh "dist/archives/${ARCHIVE}" | awk '{print $5}')
echo "==> Packaged: dist/archives/${ARCHIVE} (${SIZE})"
