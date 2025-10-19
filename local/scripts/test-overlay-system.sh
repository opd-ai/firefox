#!/bin/bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Test script to verify the overlay system is working correctly
# This script tests the Rust Dafsa implementation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "========================================"
echo "Testing Firefox Rust Overlay System"
echo "========================================"
echo ""

# Test 1: Check directory structure
echo "Test 1: Checking directory structure..."
if [ -d "local/rust/firefox_dafsa" ]; then
    echo "  ✓ local/rust/firefox_dafsa exists"
else
    echo "  ✗ local/rust/firefox_dafsa does not exist"
    exit 1
fi

if [ -f "local/rust/firefox_dafsa/Cargo.toml" ]; then
    echo "  ✓ Cargo.toml exists"
else
    echo "  ✗ Cargo.toml does not exist"
    exit 1
fi

if [ -f "local/local.mozbuild" ]; then
    echo "  ✓ local.mozbuild exists"
else
    echo "  ✗ local.mozbuild does not exist"
    exit 1
fi

echo ""

# Test 2: Build the Rust crate
echo "Test 2: Building Rust Dafsa crate..."
cd local/rust/firefox_dafsa
if cargo build --quiet; then
    echo "  ✓ Rust build succeeded"
else
    echo "  ✗ Rust build failed"
    exit 1
fi
cd "$REPO_ROOT"

echo ""

# Test 3: Run Rust tests
echo "Test 3: Running Rust tests..."
cd local/rust/firefox_dafsa
if cargo test --quiet 2>&1 | grep -q "test result: ok"; then
    echo "  ✓ Rust tests passed"
else
    echo "  ✗ Rust tests failed"
    exit 1
fi
cd "$REPO_ROOT"

echo ""

# Test 4: Check if cbindgen is available
echo "Test 4: Checking for cbindgen..."
if command -v cbindgen &> /dev/null; then
    echo "  ✓ cbindgen is installed"
    
    # Try to generate header
    cd local/rust/firefox_dafsa
    if cbindgen --config cbindgen.toml --crate firefox_dafsa --output /tmp/rust_dafsa.h 2>&1; then
        echo "  ✓ Header generation succeeded"
        rm -f /tmp/rust_dafsa.h
    else
        echo "  ✗ Header generation failed (non-fatal)"
    fi
    cd "$REPO_ROOT"
else
    echo "  ⚠ cbindgen not installed (install with: cargo install cbindgen)"
fi

echo ""

# Test 5: Verify overlay script
echo "Test 5: Testing overlay application script..."
export MOZ_RUST_DAFSA=1
if bash local/scripts/apply-build-overlays.sh > /dev/null 2>&1; then
    echo "  ✓ Overlay script executed successfully"
else
    echo "  ✗ Overlay script failed"
    exit 1
fi

echo ""

# Test 6: Check if top-level moz.build includes our overlay
echo "Test 6: Checking moz.build integration..."
if grep -q "local/local.mozbuild" moz.build; then
    echo "  ✓ Top-level moz.build includes local overlay"
else
    echo "  ✗ Top-level moz.build does not include local overlay"
    exit 1
fi

echo ""
echo "========================================"
echo "All tests passed! ✓"
echo "========================================"
echo ""
echo "The overlay system is correctly installed."
echo ""
echo "To use the Rust Dafsa implementation:"
echo "  1. Source the mozconfig: source local/mozconfig.rust-dafsa"
echo "  2. Build Firefox: ./mach build"
echo ""
echo "Or use the wrapper:"
echo "  ./local/scripts/mach-rust build"
echo ""
