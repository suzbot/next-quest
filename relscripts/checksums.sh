#!/bin/bash
set -euo pipefail

if [ ! -d "dist/archives" ]; then
  echo "Error: dist/archives/ not found. Run package.sh first."
  exit 1
fi

echo "==> Generating checksums"

cd dist/archives
shasum -a 256 *.tar.gz > checksums.txt

echo ""
cat checksums.txt
echo ""
echo "==> Checksums written to dist/archives/checksums.txt"
