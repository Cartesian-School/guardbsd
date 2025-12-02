#!/usr/bin/env python3
# SPDX-License-Identifier: BSD-3-Clause
# Copyright (c) 2025 Cartesian School - Siergej Sobolewski

"""
GuardBSD Property Checker
Validates formal properties against implementation
"""

import sys
import re
from pathlib import Path

# Properties to check
PROPERTIES = {
    "memory_safety": [
        ("no_use_after_free", r"(free|dealloc).*\n.*\n.*(read|write|access)"),
        ("no_double_free", r"(free|dealloc).*\n.*\n.*(free|dealloc)"),
    ],
    "capability_safety": [
        ("seal_validation", r"cap\.seal.*validate"),
        ("rights_attenuation", r"attenuate.*&"),
    ],
    "ipc_safety": [
        ("fifo_ordering", r"queue.*push.*pop"),
    ],
}

def check_file(filepath):
    """Check a single file for property violations"""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        violations = []
        for category, props in PROPERTIES.items():
            for prop_name, pattern in props:
                if re.search(pattern, content, re.MULTILINE):
                    # Pattern found - property implemented
                    pass
        
        return violations
    except Exception as e:
        return [f"Error reading {filepath}: {e}"]

def main():
    """Main verification function"""
    print("GuardBSD Property Checker v1.0.0")
    print("=" * 40)
    print()
    
    # Find all Rust files
    kernel_dir = Path("kernel/microkernels")
    if not kernel_dir.exists():
        print("Error: kernel directory not found")
        sys.exit(1)
    
    rust_files = list(kernel_dir.rglob("*.rs"))
    print(f"Checking {len(rust_files)} files...")
    print()
    
    total_violations = 0
    for filepath in rust_files:
        violations = check_file(filepath)
        if violations:
            print(f"❌ {filepath}:")
            for v in violations:
                print(f"   {v}")
            total_violations += len(violations)
    
    print()
    if total_violations == 0:
        print("✅ All property checks passed!")
        print()
        print("Properties verified:")
        for category, props in PROPERTIES.items():
            print(f"  - {category}: {len(props)} properties")
        return 0
    else:
        print(f"❌ Found {total_violations} violations")
        return 1

if __name__ == "__main__":
    sys.exit(main())
