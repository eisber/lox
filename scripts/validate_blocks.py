#!/usr/bin/env python3
"""
Automated Loxone block validation pipeline.

For each block type with known I/O connectors:
1. Inject VirtualIn → Block → StateV test circuit into config XML
2. Upload via FTP + reboot (~60s)
3. Set inputs via HTTP, read outputs via HTTP
4. Compare against expected truth tables
5. Report PASS/FAIL

Usage:
  python3 validate_blocks.py --category logic
  python3 validate_blocks.py --type And
  python3 validate_blocks.py --all
  python3 validate_blocks.py --structural  (injection only, no upload)
"""

import argparse
import json
import os
import re
import subprocess
import sys
import time
import urllib.request
import uuid as uuid_mod

HOST = os.environ.get("LOX_HOST", "http://192.168.68.72")
USER = os.environ.get("LOX_USER", "admin")
PASS = os.environ.get("LOX_PASS", "")
SETTLE = float(os.environ.get("LOX_SETTLE", "1.0"))
TOLERANCE = float(os.environ.get("LOX_TOLERANCE", "0.01"))

CROSSWALK_PATH = os.path.join(os.path.dirname(__file__), "..", "docs", "schemas", "crosswalk.json")

# ── Test definitions by category ──────────────────────────────────────

TESTS = {
    "logic": {
        "And": {
            "inputs": {"I1": "digital", "I2": "digital"},
            "outputs": {"Q": "digital"},
            "tests": [
                {"set": {"I1": 0, "I2": 0}, "expect": {"Q": 0}},
                {"set": {"I1": 0, "I2": 1}, "expect": {"Q": 0}},
                {"set": {"I1": 1, "I2": 0}, "expect": {"Q": 0}},
                {"set": {"I1": 1, "I2": 1}, "expect": {"Q": 1}},
            ],
        },
        "Or": {
            "inputs": {"I1": "digital", "I2": "digital"},
            "outputs": {"Q": "digital"},
            "tests": [
                {"set": {"I1": 0, "I2": 0}, "expect": {"Q": 0}},
                {"set": {"I1": 0, "I2": 1}, "expect": {"Q": 1}},
                {"set": {"I1": 1, "I2": 0}, "expect": {"Q": 1}},
                {"set": {"I1": 1, "I2": 1}, "expect": {"Q": 1}},
            ],
        },
        "Not": {
            "inputs": {"I": "digital"},
            "outputs": {"Q": "digital"},
            "tests": [
                {"set": {"I": 0}, "expect": {"Q": 1}},
                {"set": {"I": 1}, "expect": {"Q": 0}},
            ],
        },
        "Xor": {
            "inputs": {"I1": "digital", "I2": "digital"},
            "outputs": {"Q": "digital"},
            "tests": [
                {"set": {"I1": 0, "I2": 0}, "expect": {"Q": 0}},
                {"set": {"I1": 0, "I2": 1}, "expect": {"Q": 1}},
                {"set": {"I1": 1, "I2": 0}, "expect": {"Q": 1}},
                {"set": {"I1": 1, "I2": 1}, "expect": {"Q": 0}},
            ],
        },
    },
    "math": {
        "Add2": {
            "inputs": {"I1": "analog", "I2": "analog"},
            "outputs": {"Q": "analog"},
            "tests": [
                {"set": {"I1": 3, "I2": 5}, "expect": {"Q": 8}},
                {"set": {"I1": -2, "I2": 7}, "expect": {"Q": 5}},
                {"set": {"I1": 0, "I2": 0}, "expect": {"Q": 0}},
            ],
        },
        "Subtract": {
            "inputs": {"I1": "analog", "I2": "analog"},
            "outputs": {"Q": "analog"},
            "tests": [
                {"set": {"I1": 10, "I2": 3}, "expect": {"Q": 7}},
                {"set": {"I1": 5, "I2": 5}, "expect": {"Q": 0}},
            ],
        },
        "Multiply": {
            "inputs": {"I1": "analog", "I2": "analog"},
            "outputs": {"Q": "analog"},
            "tests": [
                {"set": {"I1": 3, "I2": 4}, "expect": {"Q": 12}},
                {"set": {"I1": 0, "I2": 99}, "expect": {"Q": 0}},
            ],
        },
        "Divide": {
            "inputs": {"I1": "analog", "I2": "analog"},
            "outputs": {"Q": "analog"},
            "tests": [
                {"set": {"I1": 12, "I2": 4}, "expect": {"Q": 3}},
                {"set": {"I1": 7, "I2": 2}, "expect": {"Q": 3.5}},
            ],
        },
    },
    "comparison": {
        "Comparator": {
            "inputs": {"I1": "analog", "I2": "analog"},
            "outputs": {"GT": "digital", "EQ": "digital", "LT": "digital"},
            "tests": [
                {"set": {"I1": 5, "I2": 3}, "expect": {"GT": 1, "EQ": 0, "LT": 0}},
                {"set": {"I1": 3, "I2": 3}, "expect": {"GT": 0, "EQ": 1, "LT": 0}},
                {"set": {"I1": 1, "I2": 3}, "expect": {"GT": 0, "EQ": 0, "LT": 1}},
            ],
        },
    },
    "stateful": {
        "FlipFlopRS": {
            "inputs": {"R": "digital", "S": "digital"},
            "outputs": {"Q": "digital"},
            "tests": [
                {"set": {"S": 1, "R": 0}, "expect": {"Q": 1}},
                {"set": {"S": 0, "R": 0}, "expect": {"Q": 1}},
                {"set": {"S": 0, "R": 1}, "expect": {"Q": 0}},
                {"set": {"S": 0, "R": 0}, "expect": {"Q": 0}},
            ],
        },
    },
}


def lox_uuid():
    """Generate a Loxone-style UUID."""
    u = uuid_mod.uuid4()
    h = u.hex
    return f"{h[:8]}-{h[8:12]}-{h[12:16]}-ffff000000000000"


def http_get(path):
    """HTTP GET with basic auth."""
    url = f"{HOST}/{path}"
    req = urllib.request.Request(url)
    import base64
    creds = base64.b64encode(f"{USER}:{PASS}".encode()).decode()
    req.add_header("Authorization", f"Basic {creds}")
    try:
        resp = urllib.request.urlopen(req, timeout=10)
        return resp.read().decode("utf-8")
    except Exception as e:
        return f"ERROR: {e}"


def set_input(vi_name, value):
    """Set a VirtualIn value via HTTP API."""
    encoded = vi_name.replace(" ", "%20")
    resp = http_get(f"jdev/sps/io/{encoded}/{value}")
    return "200" in resp


def read_output(uuid):
    """Read a StateV value via HTTP API."""
    resp = http_get(f"jdev/sps/io/{uuid}/state")
    try:
        data = json.loads(resp)
        return float(data["LL"]["value"])
    except Exception:
        return None


def load_crosswalk():
    """Load the crosswalk.json type database."""
    with open(CROSSWALK_PATH) as f:
        return json.load(f)["types"]


def get_connectors(crosswalk, block_type):
    """Get input/output connectors for a block type."""
    info = crosswalk.get(block_type, {})
    conns = info.get("connectors", {})
    inputs = {}
    outputs = {}
    for key, val in conns.items():
        direction = val.get("direction", val.get("dir", ""))
        if direction == "input":
            inputs[key] = val
        elif direction == "output":
            outputs[key] = val
    return inputs, outputs


def structural_test(crosswalk):
    """Test that all 163 types can be injected into a config XML."""
    print("\n=== Structural Validation ===")
    print(f"Testing {len(crosswalk)} block types...\n")

    # Download and extract current config
    result = subprocess.run(
        ["lox", "config", "download", "-q"],
        capture_output=True, text=True
    )
    # Find the downloaded file
    zips = sorted(
        [f for f in os.listdir(".") if f.startswith("sps_") and f.endswith(".zip")],
        key=os.path.getmtime, reverse=True,
    )
    if not zips:
        print("ERROR: No config ZIP found")
        return
    zip_file = zips[0]
    subprocess.run(["lox", "config", "extract", zip_file], capture_output=True)
    loxone_file = zip_file.replace(".zip", ".Loxone")
    if not os.path.exists(loxone_file):
        print(f"ERROR: {loxone_file} not found")
        return

    passed = 0
    failed = 0
    errors = []

    for block_type in sorted(crosswalk.keys()):
        # Try adding this block type
        test_file = f"/tmp/structural_test.Loxone"
        subprocess.run(["cp", loxone_file, test_file], capture_output=True)
        result = subprocess.run(
            ["lox", "config", "add", "--type", block_type, "--title", f"SV_{block_type}", test_file],
            capture_output=True, text=True,
        )
        if result.returncode == 0:
            passed += 1
            print(f"  ✓ {block_type}")
        else:
            failed += 1
            err = result.stderr.strip().split("\n")[-1] if result.stderr else "unknown"
            errors.append((block_type, err))
            print(f"  ✗ {block_type}: {err[:60]}")

    print(f"\nStructural: {passed}/{passed + failed} types can be injected")
    if errors:
        print(f"Failures:")
        for t, e in errors:
            print(f"  {t}: {e}")

    # Cleanup
    os.remove(f"/tmp/structural_test.Loxone") if os.path.exists(f"/tmp/structural_test.Loxone") else None
    return passed, failed


def behavioral_test(category, tests_dict, dry_run=False):
    """Run behavioral tests for a category of blocks."""
    print(f"\n=== Behavioral Validation: {category} ===")

    crosswalk = load_crosswalk()
    total_pass = 0
    total_fail = 0

    for block_type, spec in tests_dict.items():
        print(f"\n--- {block_type} ---")
        inputs_spec = spec["inputs"]
        outputs_spec = spec["outputs"]
        test_cases = spec["tests"]

        # Check if block already exists in config (from previous setup)
        # For now, just run tests against pre-wired blocks
        print(f"  Inputs: {list(inputs_spec.keys())}")
        print(f"  Outputs: {list(outputs_spec.keys())}")
        print(f"  Test cases: {len(test_cases)}")

        if dry_run:
            print(f"  [DRY RUN] Would run {len(test_cases)} tests")
            continue

        for i, tc in enumerate(test_cases):
            # Set inputs
            for conn_key, value in tc["set"].items():
                vi_name = f"VI_{block_type}_{conn_key}"
                if not set_input(vi_name, value):
                    print(f"  [{i}] SKIP: Could not set {vi_name}={value}")
                    continue

            time.sleep(SETTLE)

            # Read outputs
            all_match = True
            for conn_key, expected in tc["expect"].items():
                sv_uuid = f"SV_{block_type}_{conn_key}"
                # For now, we'd need the actual UUID — placeholder
                actual = read_output(sv_uuid)
                if actual is None:
                    print(f"  [{i}] SKIP: Could not read {sv_uuid}")
                    all_match = False
                    continue
                if abs(actual - expected) > TOLERANCE:
                    print(f"  [{i}] FAIL: {conn_key} expected={expected} actual={actual}")
                    all_match = False
                else:
                    pass

            if all_match:
                total_pass += 1
            else:
                total_fail += 1

        input_str = ", ".join(f"{k}={v}" for tc in test_cases for k, v in tc["set"].items())

    print(f"\n{category}: {total_pass} PASS, {total_fail} FAIL")
    return total_pass, total_fail


def main():
    parser = argparse.ArgumentParser(description="Loxone block validation")
    parser.add_argument("--category", choices=list(TESTS.keys()) + ["all"],
                        help="Test category")
    parser.add_argument("--type", dest="block_type", help="Specific block type")
    parser.add_argument("--structural", action="store_true",
                        help="Structural validation only (no upload)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Show what would be tested without running")
    parser.add_argument("--list", action="store_true",
                        help="List available test categories and types")
    args = parser.parse_args()

    if args.list:
        print("Available test categories:")
        for cat, types in TESTS.items():
            n_tests = sum(len(t["tests"]) for t in types.values())
            print(f"  {cat}: {len(types)} types, {n_tests} test cases")
            for t in types:
                print(f"    - {t}")
        return

    if args.structural:
        crosswalk = load_crosswalk()
        structural_test(crosswalk)
        return

    if args.block_type:
        # Find the type in any category
        for cat, types in TESTS.items():
            if args.block_type in types:
                behavioral_test(cat, {args.block_type: types[args.block_type]}, args.dry_run)
                return
        print(f"No tests defined for '{args.block_type}'")
        print(f"Available: {', '.join(t for types in TESTS.values() for t in types)}")
        return

    if args.category == "all":
        for cat, types in TESTS.items():
            behavioral_test(cat, types, args.dry_run)
    elif args.category:
        behavioral_test(args.category, TESTS[args.category], args.dry_run)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
