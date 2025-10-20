#!/usr/bin/env python3
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

"""
Generate C++ header for Rust JSONWriter implementation using cbindgen.
This script is called during the build process.
"""

import os
import subprocess
import sys


def main(output, *inputs):
    """Generate header file using cbindgen."""
    
    # Find cbindgen in cargo bin
    cargo_home = os.environ.get('CARGO_HOME', os.path.expanduser('~/.cargo'))
    cbindgen = os.path.join(cargo_home, 'bin', 'cbindgen')
    
    if not os.path.exists(cbindgen):
        # Try without .exe extension on Windows
        cbindgen_exe = cbindgen + '.exe'
        if os.path.exists(cbindgen_exe):
            cbindgen = cbindgen_exe
        else:
            print(f"ERROR: cbindgen not found at {cbindgen}", file=sys.stderr)
            print("Install it with: cargo install cbindgen", file=sys.stderr)
            return 1
    
    # Get the repo root
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(os.path.dirname(script_dir))
    crate_dir = os.path.join(repo_root, 'local', 'rust', 'firefox_jsonwriter')
    
    # Run cbindgen
    try:
        result = subprocess.run(
            [cbindgen, '--config', 'cbindgen.toml', '--crate', 'firefox_jsonwriter', 
             '--output', output],
            cwd=crate_dir,
            check=True,
            capture_output=True,
            text=True
        )
        print(result.stdout)
        return 0
    except subprocess.CalledProcessError as e:
        print(f"ERROR: cbindgen failed: {e}", file=sys.stderr)
        print(e.stdout, file=sys.stderr)
        print(e.stderr, file=sys.stderr)
        return 1
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main(*sys.argv[1:]))
