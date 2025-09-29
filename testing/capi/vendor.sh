#!/usr/bin/env bash

set -euo pipefail

INSTRUMENT_HOOKS_COMMIT="0d3de57fe46ef97714a41ed028096f6f84fdbd2a"
INSTRUMENT_HOOKS_URL="https://github.com/CodSpeedHQ/instrument-hooks/archive/${INSTRUMENT_HOOKS_COMMIT}.tar.gz"
VENDOR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMP_DIR=$(mktemp -d)

rm -rf "${VENDOR_DIR}/instrument-hooks"

# Download and extract the instrument-hooks library
curl -L "${INSTRUMENT_HOOKS_URL}" -o "${TEMP_DIR}/instrument-hooks.tar.gz"
tar -xzf "${TEMP_DIR}/instrument-hooks.tar.gz" -C "${TEMP_DIR}"

# Copy only the dist and includes directories to the vendor directory
mkdir -p "${VENDOR_DIR}/instrument-hooks/"
cp -r "${TEMP_DIR}/instrument-hooks-${INSTRUMENT_HOOKS_COMMIT}/dist" "${VENDOR_DIR}/instrument-hooks/"
cp -r "${TEMP_DIR}/instrument-hooks-${INSTRUMENT_HOOKS_COMMIT}/includes" "${VENDOR_DIR}/instrument-hooks/"

# Clean up
rm -rf "${TEMP_DIR}"
