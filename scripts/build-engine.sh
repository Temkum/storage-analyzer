#!/usr/bin/env bash
#
# Configures, builds and tests the C++ engine, then stages the sidecar binary
# where the Tauri build expects it (apps/desktop/src-tauri/binaries).
#
# Usage: scripts/build-engine.sh [extra cmake args...]
# Env:   ENGINE_GENERATOR=Ninja (default), ENGINE_BUILD_TYPE=Debug|Release

set -euo pipefail

cd "$(dirname "$0")/.."

GENERATOR="${ENGINE_GENERATOR:-Ninja}"
BUILD_TYPE="${ENGINE_BUILD_TYPE:-Debug}"

cmake -S . -B build -G "$GENERATOR" -DCMAKE_BUILD_TYPE="$BUILD_TYPE" "$@"
cmake --build build
ctest --output-on-failure --test-dir build

echo ""
echo "Engine ready. Sidecar staged under apps/desktop/src-tauri/binaries/"