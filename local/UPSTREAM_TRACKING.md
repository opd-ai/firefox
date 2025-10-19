# Upstream Tracking Guide

This document explains how the local overlay system maintains a clean tracking relationship with upstream mozilla-central.

## Overview

The overlay system is designed to survive `git pull` operations without merge conflicts. This allows seamless integration of upstream changes while preserving local Rust port overlays.

## Modified Upstream Files

Only **ONE** upstream file has been modified:

### `moz.build` (top-level)

**Change**: Added 3 lines at the end of the file:

```python
# Local overlay for downstream customizations
if os.path.exists(os.path.join(TOPSRCDIR, "local", "local.mozbuild")):
    include("local/local.mozbuild")
```

**Location**: After the last line (`include("build/templates.mozbuild")`)

**Conflict Likelihood**: **VERY LOW**
- This is at the very end of the file
- It's a simple conditional include
- It checks for file existence before including
- Upstream is unlikely to add similar code

**If Conflict Occurs**: 
- The conflict will be trivial to resolve
- Simply ensure these 3 lines are at the end of the file
- The include is safe even if the file doesn't exist

## Unmodified Upstream Files

All other upstream files remain **completely untouched**:

- ✅ `xpcom/ds/Dafsa.cpp` - Original C++ implementation preserved
- ✅ `xpcom/ds/Dafsa.h` - Original header preserved
- ✅ `xpcom/ds/moz.build` - Build configuration preserved
- ✅ `xpcom/tests/gtest/TestDafsa.cpp` - Tests preserved
- ✅ `toolkit/library/rust/shared/Cargo.toml` - Only modified at build time (not committed)

## New Local Files (Never Conflict)

All customizations live in the `local/` directory:

```
local/
├── README.md
├── UPSTREAM_TRACKING.md (this file)
├── mozconfig.rust-dafsa
├── local.mozbuild
├── moz.build
├── rust/
│   ├── Cargo.toml
│   └── firefox_dafsa/
│       ├── Cargo.toml
│       ├── cbindgen.toml
│       └── src/
│           ├── lib.rs
│           └── ffi.rs
├── cargo-patches/
│   └── shared-deps.toml
└── scripts/
    ├── apply-build-overlays.sh
    ├── mach-rust
    ├── generate-dafsa-header.py
    └── test-overlay-system.sh
```

**Upstream Impact**: ZERO - Mozilla will never create files in `local/`

## Pulling from Upstream

### Standard Pull Operation

```bash
# Pull upstream changes
git pull upstream master

# No conflicts expected!
# The local/ directory is never touched by upstream
# Only moz.build might conflict, and it's easily resolved
```

### If moz.build Conflicts

```bash
# After git pull shows conflict in moz.build:

# Option 1: Keep both changes
git checkout --ours moz.build
# Then manually add the 3 lines at the end if they were removed

# Option 2: Use a merge tool
git mergetool

# Option 3: Manual resolution
# Edit moz.build and ensure these lines are at the end:
#   if os.path.exists(os.path.join(TOPSRCDIR, "local", "local.mozbuild")):
#       include("local/local.mozbuild")

# Verify the fix
git diff moz.build

# Continue the merge
git add moz.build
git commit
```

## Build-Time Overlays

Some files are modified **only at build time** (not committed):

### `toolkit/library/rust/shared/Cargo.toml`

This file has dependencies appended when `MOZ_RUST_DAFSA=1` is set.

**Why safe?**
- Changes are only in your working directory
- Never committed to git
- Applied by `apply-build-overlays.sh` script
- Idempotent (can be reapplied safely)

**After upstream pull**:
```bash
# If you had MOZ_RUST_DAFSA=1 enabled, reapply:
source local/mozconfig.rust-dafsa
# or
bash local/scripts/apply-build-overlays.sh
```

## Verifying Clean Tracking

Run this command to verify only expected files are modified:

```bash
git status
```

Expected output:
- Modified: `moz.build` (only if you haven't committed the overlay hook)
- Untracked: `local/` (if first time)
- Modified: `toolkit/library/rust/shared/Cargo.toml` (only if you ran apply-build-overlays.sh)

To see the minimal diff:

```bash
git diff moz.build
```

Should show only the 3-line addition at the end.

## Switching Between C++ and Rust

### Use C++ (Default)

```bash
# Don't source the mozconfig or set MOZ_RUST_DAFSA
./mach build
```

### Use Rust

```bash
# Option 1: Source the mozconfig
source local/mozconfig.rust-dafsa
./mach build

# Option 2: Use the wrapper
./local/scripts/mach-rust build

# Option 3: Manual
export MOZ_RUST_DAFSA=1
bash local/scripts/apply-build-overlays.sh
./mach build
```

## Testing Upstream Compatibility

Run the test script to verify everything is working:

```bash
bash local/scripts/test-overlay-system.sh
```

This checks:
- ✓ Directory structure is correct
- ✓ Rust code compiles
- ✓ Rust tests pass
- ✓ Build system integration is correct
- ✓ Overlay scripts work

## Best Practices

1. **Always pull from upstream with clean working directory**
   ```bash
   git status  # Should show clean or only local/ changes
   git pull upstream master
   ```

2. **Don't commit build-time changes**
   - `toolkit/library/rust/shared/Cargo.toml` should not be committed with overlay additions
   - Use `.gitignore` or manual cleanup

3. **Document all modifications**
   - If you add new overlays, document them here
   - Explain why they're safe from conflicts

4. **Test after upstream pulls**
   ```bash
   bash local/scripts/test-overlay-system.sh
   ```

## Conflict Resolution Checklist

If you encounter conflicts when pulling from upstream:

- [ ] Check which files have conflicts (`git status`)
- [ ] If `moz.build`: Ensure 3-line overlay hook is at the end
- [ ] If `local/*`: This shouldn't happen (upstream won't modify these)
- [ ] If other files: These are unrelated to the overlay system
- [ ] After resolution: Run `bash local/scripts/test-overlay-system.sh`
- [ ] Verify build works: `./mach build` (with and without overlay)

## Summary

**Upstream Modifications**: 1 file, 3 lines (moz.build)
**Conflict Risk**: Very Low
**Maintenance Burden**: Minimal
**Upstream Tracking**: Clean

This design achieves true zero-impact coexistence with upstream mozilla-central.
