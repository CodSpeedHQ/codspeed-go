#!/usr/bin/env bash
set -e

# Script to regenerate patch files for benchmark instrumentation
# This downloads unpatched Go benchmark.go files and diffs them against our patched versions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSIONS=("1.24.0" "1.25.0")

for VERSION in "${VERSIONS[@]}"; do
    PATCH_FILE="$SCRIPT_DIR/benchmark${VERSION}.patch"
    PATCHED_FILE="$SCRIPT_DIR/benchmark${VERSION}.go"
    UNPATCHED_FILE=$(mktemp)

    # Download the unpatched benchmark.go from Go's repository
    wget -q -O "$UNPATCHED_FILE" \
        "https://github.com/golang/go/raw/refs/tags/go${VERSION}/src/testing/benchmark.go" \
        || { echo "Failed to download benchmark.go for Go $VERSION"; rm "$UNPATCHED_FILE"; exit 1; }

    # Set both files to have the same timestamp (epoch) to avoid timestamp differences in the patch
    touch -d "@0" "$UNPATCHED_FILE" "$PATCHED_FILE"

    # Generate the patch file (unpatched -> patched)
    # Use --label to set consistent filenames in the diff header
    diff -a -u -N \
        --label "benchmark${VERSION}.go" \
        --label "overlay/benchmark${VERSION}.go" \
        "$UNPATCHED_FILE" "$PATCHED_FILE" > "$PATCH_FILE" || true

    rm "$UNPATCHED_FILE"
done

pre-commit run --all-files > /dev/null 2>&1 || true
