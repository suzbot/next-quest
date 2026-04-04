#!/bin/bash
set -euo pipefail

echo "==> Building release binaries"

# Clean dist/
rm -rf dist
mkdir -p dist

# Build GUI (Tauri release)
echo "  Building GUI (cargo tauri build)..."
cargo tauri build --bundles app 2>&1
echo "  GUI build complete"

# Build CLI
echo "  Building CLI (cargo build --release -p nq)..."
cargo build --release -p nq 2>&1
echo "  CLI build complete"

# Copy artifacts to dist/
APP_PATH="target/release/bundle/macos/Next Quest.app"
CLI_PATH="target/release/nq"

if [ ! -d "$APP_PATH" ]; then
  echo "Error: Expected .app bundle at ${APP_PATH} but not found"
  exit 1
fi

if [ ! -f "$CLI_PATH" ]; then
  echo "Error: Expected CLI binary at ${CLI_PATH} but not found"
  exit 1
fi

cp -r "$APP_PATH" "dist/Next Quest.app"
cp "$CLI_PATH" dist/nq

echo ""
echo "==> Build complete:"
echo "  dist/Next Quest.app"
echo "  dist/nq"
