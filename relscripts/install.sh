#!/bin/bash
set -euo pipefail

echo "==> Installing Next Quest"

# Check build artifacts exist
if [ ! -d "dist/Next Quest.app" ]; then
  echo "Error: dist/Next Quest.app not found. Run build.sh first."
  exit 1
fi
if [ ! -f "dist/nq" ]; then
  echo "Error: dist/nq not found. Run build.sh first."
  exit 1
fi

# Install GUI app
if [ -d "/Applications/Next Quest.app" ]; then
  echo "  Removing previous /Applications/Next Quest.app"
  rm -rf "/Applications/Next Quest.app"
fi
cp -r "dist/Next Quest.app" "/Applications/Next Quest.app"
echo "  Installed /Applications/Next Quest.app"

# Install CLI
if cp dist/nq /usr/local/bin/nq 2>/dev/null; then
  echo "  Installed /usr/local/bin/nq"
else
  echo ""
  echo "  Could not copy to /usr/local/bin/nq (permission denied)."
  echo "  Try: sudo cp dist/nq /usr/local/bin/nq"
  echo ""
fi

echo ""
echo "==> Done. Launch 'Next Quest' from Spotlight or run 'nq' from any terminal."
