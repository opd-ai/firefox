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

if [ "$MOZ_RUST_HASHBYTES" = "1" ]; then
    echo "Enabling Rust HashBytes implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_hashbytes" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_hashbytes to $SHARED_CARGO"
        cat local/cargo-patches/hashbytes-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_hashbytes already present in $SHARED_CARGO"
    fi
    
    echo "Rust HashBytes overlay applied successfully"
else
    echo "MOZ_RUST_HASHBYTES not set, skipping Rust HashBytes overlay"
fi

if [ "$MOZ_RUST_FLOATINGPOINT" = "1" ]; then
    echo "Enabling Rust FloatingPoint implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_floatingpoint" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_floatingpoint to $SHARED_CARGO"
        cat local/cargo-patches/floatingpoint-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_floatingpoint already present in $SHARED_CARGO"
    fi
    
    echo "Rust FloatingPoint overlay applied successfully"
else
    echo "MOZ_RUST_FLOATINGPOINT not set, skipping Rust FloatingPoint overlay"
fi

if [ "$MOZ_RUST_UTF8_VALIDATOR" = "1" ]; then
    echo "Enabling Rust UTF-8 Validator implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_utf8_validator" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_utf8_validator to $SHARED_CARGO"
        cat local/cargo-patches/utf8-validator-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_utf8_validator already present in $SHARED_CARGO"
    fi
    
    echo "Rust UTF-8 Validator overlay applied successfully"
else
    echo "MOZ_RUST_UTF8_VALIDATOR not set, skipping Rust UTF-8 Validator overlay"
fi

if [ "$MOZ_RUST_JSONWRITER" = "1" ]; then
    echo "Enabling Rust JSONWriter implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_jsonwriter" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_jsonwriter to $SHARED_CARGO"
        cat local/cargo-patches/jsonwriter-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_jsonwriter already present in $SHARED_CARGO"
    fi
    
    echo "Rust JSONWriter overlay applied successfully"
else
    echo "MOZ_RUST_JSONWRITER not set, skipping Rust JSONWriter overlay"
fi

if [ "$MOZ_RUST_OBSERVER_ARRAY" = "1" ]; then
    echo "Enabling Rust Observer Array implementation..."
    
    # Append Cargo dependencies (idempotent check)
    SHARED_CARGO="toolkit/library/rust/shared/Cargo.toml"
    if ! grep -q "firefox_observer_array" "$SHARED_CARGO" 2>/dev/null; then
        echo "  Adding firefox_observer_array to $SHARED_CARGO"
        cat local/cargo-patches/observer-array-deps.toml >> "$SHARED_CARGO"
    else
        echo "  firefox_observer_array already present in $SHARED_CARGO"
    fi
    
    echo "Rust Observer Array overlay applied successfully"
else
    echo "MOZ_RUST_OBSERVER_ARRAY not set, skipping Rust Observer Array overlay"
fi

echo "Done applying overlays"
