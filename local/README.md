# Local Firefox Customizations

This directory contains local customizations and overlays for the Firefox build system. These are designed to coexist with upstream mozilla-central without causing merge conflicts.

## Directory Structure

```
local/
├── README.md                           # This file
├── mozconfig.rust-dafsa                # Configuration to enable Rust Dafsa
├── local.mozbuild                      # Build system inclusion file
├── moz.build                           # Local build definitions
├── rust/                               # Local Rust crates
│   ├── Cargo.toml                      # Workspace definition
│   └── firefox_dafsa/                  # Rust implementation of Dafsa
│       ├── Cargo.toml
│       ├── cbindgen.toml
│       └── src/
│           ├── lib.rs
│           └── ffi.rs
├── cargo-patches/                      # Cargo.toml patches to apply
│   └── shared-deps.toml                # Additions for shared Cargo.toml
└── scripts/                            # Build automation scripts
    ├── apply-build-overlays.sh         # Apply overlay patches
    ├── mach-rust                       # Mach wrapper with overlays
    └── generate-dafsa-header.py        # Generate C++ headers from Rust
```

## Rust Dafsa Implementation

The Rust Dafsa implementation is a port of the C++ `Dafsa` class from `xpcom/ds/Dafsa.{h,cpp}`.

### Features

- **Zero-impact coexistence**: Can be enabled/disabled without modifying upstream files
- **API compatibility**: Matches the C++ API exactly
- **FFI layer**: C-compatible exports for seamless interop
- **Comprehensive tests**: Mirrors the C++ test suite

### Usage

#### Building with Rust Dafsa

```bash
# Option 1: Use the wrapper script
./local/scripts/mach-rust build

# Option 2: Source the mozconfig
source local/mozconfig.rust-dafsa
./mach build

# Option 3: Set environment variable
export MOZ_RUST_DAFSA=1
./local/scripts/apply-build-overlays.sh
./mach build
```

#### Building without Rust Dafsa (default)

```bash
# Just build normally - the C++ version will be used
./mach build
```

### Testing

```bash
# Run C++ tests (works with either implementation)
./mach gtest Dafsa.*

# Run Rust tests
cd local/rust/firefox_dafsa
cargo test
```

## Maintenance

### Pulling Upstream Changes

The overlay system is designed to survive `git pull` without conflicts:

```bash
# Pull upstream changes normally
git pull upstream master

# If conflicts occur in files you've modified locally, resolve them
# The local/ directory is never modified by upstream

# Reapply overlays if needed
./local/scripts/apply-build-overlays.sh
```

### Adding New Rust Components

1. Create a new crate in `local/rust/`
2. Add it to `local/rust/Cargo.toml` workspace
3. Create a patch file in `local/cargo-patches/`
4. Update `local/scripts/apply-build-overlays.sh` to apply the patch
5. Add build configuration in `local/moz.build`

## Design Principles

1. **Overlay, Don't Modify**: Upstream files remain untouched wherever possible
2. **Additive Configuration**: Use separate override files instead of inline modifications
3. **Idempotent Scripts**: Overlays can be applied multiple times safely
4. **Zero Conflicts**: Structure survives upstream pulls cleanly

## Notes

- The `local/` directory is git-tracked to preserve the overlay infrastructure
- Generated artifacts (like merged Cargo.toml files) are gitignored
- The build system automatically handles switching between C++ and Rust implementations
- No changes to Firefox's existing test infrastructure are required

## Future Work

- Port additional components (e.g., nsAtom, TimeStamp)
- Add performance benchmarks comparing C++ and Rust implementations
- Integrate with Firefox's performance testing infrastructure
- Document porting process for other developers
