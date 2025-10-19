#!/bin/bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Apply local build system overlays
# Run this before building with Rust components enabled

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT"

echo "Applying local build overlays..."

if [ "$MOZ_RUST_DAFSA" = "1" ]; then
    echo "Enabling Rust Dafsa implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_dafsa" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_dafsa to $SHARED_CARGO"
        cat local/cargo-patches/shared-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_dafsa already present in $SHARED_CARGO"
    fi
    
    echo "Rust Dafsa overlay applied successfully"
else
    echo "MOZ_RUST_DAFSA not set, skipping Rust Dafsa overlay"
fi

if [ "$MOZ_RUST_CHAOSMODE" = "1" ]; then
    echo "Enabling Rust ChaosMode implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_chaosmode" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_chaosmode to $SHARED_CARGO"
        cat local/cargo-patches/chaosmode-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_chaosmode already present in $SHARED_CARGO"
    fi
    
    echo "Rust ChaosMode overlay applied successfully"
else
    echo "MOZ_RUST_CHAOSMODE not set, skipping Rust ChaosMode overlay"
fi

if [ "$MOZ_RUST_XORSHIFT128PLUS" = "1" ]; then
    echo "Enabling Rust XorShift128+ implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_xorshift128plus" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_xorshift128plus to $SHARED_CARGO"
        cat local/cargo-patches/xorshift128plus-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_xorshift128plus already present in $SHARED_CARGO"
    fi
    
    echo "Rust XorShift128+ overlay applied successfully"
else
    echo "MOZ_RUST_XORSHIFT128PLUS not set, skipping Rust XorShift128+ overlay"
fi

echo "Done applying overlays"
