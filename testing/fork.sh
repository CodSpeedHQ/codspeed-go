#!/usr/bin/env bash
set -e  # Exit on error

# Ensure we're running from the testing/ directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

GO_VERSION="${1:-1.25}"  # Allow version override: ./fork.sh 1.25

# Copy internal packages from Go source to local internal/ directory
# Takes package names as arguments (e.g., "cpu" "fuzz" "race")
copy_internal_packages() {
    for pkg in "$@"; do
        mkdir -p "internal/$pkg"
        cp -r "go/src/internal/$pkg"/* "internal/$pkg/"
    done
}

# Backup files to the hardcoded backup directory (.codspeed-backup)
# Usage: backup_files <file_or_dir> [file_or_dir ...]
backup_files() {
    local backup_dir=".codspeed-backup"
    mkdir -p "$backup_dir"

    for file in "$@"; do
        if [ -e "$file" ]; then
            cp -r "$file" "$backup_dir/$(basename "$file")"
            echo "Backed up: $file"
        fi
    done
}

# Restore files from the hardcoded backup directory (.codspeed-backup)
# Usage: restore_files <file_or_dir> [file_or_dir ...]
restore_files() {
    local backup_dir=".codspeed-backup"

    echo "Restoring files from backup..."
    for file in "$@"; do
        local backup_file="$backup_dir/$(basename "$file")"
        if [ -e "$backup_file" ]; then
            # Remove destination first to avoid creating nested directories
            rm -rf "$file"
            cp -r "$backup_file" "$file"
            echo "Restored: $file"
        fi
    done
}

# Apply a patch file with error handling
# Usage: apply_patch <patch_file> [fuzz_factor] [working_directory]
# If working_directory is provided, patch is applied from there
apply_patch() {
    local patch_file="$1"
    local fuzz="${2:-2}"  # Default fuzz factor is 2
    local workdir="${3:-.}"

    if [ ! -f "$patch_file" ]; then
        echo "ERROR: Patch file not found: $patch_file"
        return 1
    fi

    echo "Applying patch: $(basename "$patch_file")..."

    if [ "$workdir" != "." ]; then
        (cd "$workdir" && patch -p1 --forward --fuzz="$fuzz" < "../$patch_file") || {
            echo "ERROR: $patch_file failed to apply cleanly"
            return 1
        }
    else
        patch -p1 --forward --fuzz="$fuzz" < "$patch_file" || {
            echo "ERROR: $patch_file failed to apply cleanly"
            return 1
        }
    fi
    echo "Successfully applied: $(basename "$patch_file")"
}

echo "Forking Go testing package from version ${GO_VERSION}..."

# Backup CodSpeed-specific files before removing directories
backup_files "testing/codspeed.go"

# We need to copy the testing/ package:
git clone -b "release-branch.go${GO_VERSION}" --depth 1 https://github.com/golang/go/
rm -rf internal testing
cp -r go/src/testing testing/

# Copy all required internal packages. We need them to have a clean `go mod tidy` output.
copy_internal_packages "cpu" "fuzz" "goarch" "race" "sysinfo" "testlog" "testenv" "syscall/windows" "godebug" "synctest" "bisect" "godebugs" "abi" "cfg" "platform" "diff" "txtar"

# Replace all `"internal/*"` imports with 'github.com/CodSpeedHQ/codspeed-go/testing/internal/'
find . -type f -name "*.go" -exec sed -i 's|"internal/|"github.com/CodSpeedHQ/codspeed-go/testing/internal/|g' {} +

# Apply the race package patch to remove abi dependency
apply_patch "patches/internal_race.patch" 0

# Apply CodSpeed modifications to testing package (split into separate files)
apply_patch "patches/benchmark.patch" 10 "testing"
apply_patch "patches/testing.patch" 10 "testing"

# Restore CodSpeed-specific files
restore_files "testing/codspeed.go"

# Cleanup
rm -rf go .codspeed-backup

# Run pre-commit to format the code
pre-commit run --all-files > /dev/null 2>&1 || true
