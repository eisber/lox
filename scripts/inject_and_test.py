#!/usr/bin/env python3
"""
Inject test circuits into Loxone config, upload, and validate via HTTP API.

Creates: VirtualIn → Block → VirtualState (StateV) circuits.
Uploads via FTP + reboot. Tests via HTTP set/read.

Usage:
  export LOX_PASS="yourpass"
  python3 inject_and_test.py --types And,Or,Not,Xor
  python3 inject_and_test.py --category logic
  python3 inject_and_test.py --all-testable
"""

import argparse
import base64
import json
import os
import re
import subprocess
import sys
import time
import urllib.request
import uuid as uuid_mod
import xml.etree.ElementTree as ET

HOST = os.environ.get("LOX_HOST", "http://192.168.68.72")
USER = os.environ.get("LOX_USER", "admin")
PASS = os.environ.get("LOX_PASS", "")
SETTLE = float(os.environ.get("LOX_SETTLE", "1.5"))
REBOOT_WAIT = int(os.environ.get("LOX_REBOOT_WAIT", "75"))

if not PASS:
    print("ERROR: Set LOX_PASS environment variable", file=sys.stderr)
    sys.exit(1)

# ── Test definitions ──────────────────────────────────────────────────

TESTS = {
    "And": {
        "inputs": {"I1": "digital", "I2": "digital"},
        "outputs": {"Q": "digital"},
        "cases": [
            ({"I1": 0, "I2": 0}, {"Q": 0}),
            ({"I1": 0, "I2": 1}, {"Q": 0}),
            ({"I1": 1, "I2": 0}, {"Q": 0}),
            ({"I1": 1, "I2": 1}, {"Q": 1}),
        ],
    },
    "Or": {
        "inputs": {"I1": "digital", "I2": "digital"},
        "outputs": {"Q": "digital"},
        "cases": [
            ({"I1": 0, "I2": 0}, {"Q": 0}),
            ({"I1": 0, "I2": 1}, {"Q": 1}),
            ({"I1": 1, "I2": 0}, {"Q": 1}),
            ({"I1": 1, "I2": 1}, {"Q": 1}),
        ],
    },
    "Not": {
        "inputs": {"I": "digital"},
        "outputs": {"Q": "digital"},
        "cases": [
            ({"I": 0}, {"Q": 1}),
            ({"I": 1}, {"Q": 0}),
        ],
    },
    "Xor": {
        "inputs": {"I1": "digital", "I2": "digital"},
        "outputs": {"Q": "digital"},
        "cases": [
            ({"I1": 0, "I2": 0}, {"Q": 0}),
            ({"I1": 0, "I2": 1}, {"Q": 1}),
            ({"I1": 1, "I2": 0}, {"Q": 1}),
            ({"I1": 1, "I2": 1}, {"Q": 0}),
        ],
    },
    "Add2": {
        "inputs": {"I1": "analog", "I2": "analog"},
        "outputs": {"Q": "analog"},
        "cases": [
            ({"I1": 3, "I2": 5}, {"Q": 8}),
            ({"I1": -2, "I2": 7}, {"Q": 5}),
            ({"I1": 0, "I2": 0}, {"Q": 0}),
        ],
    },
    "Subtract": {
        "inputs": {"I1": "analog", "I2": "analog"},
        "outputs": {"Q": "analog"},
        "cases": [
            ({"I1": 10, "I2": 3}, {"Q": 7}),
            ({"I1": 5, "I2": 5}, {"Q": 0}),
        ],
    },
    "Multiply": {
        "inputs": {"I1": "analog", "I2": "analog"},
        "outputs": {"Q": "analog"},
        "cases": [
            ({"I1": 3, "I2": 4}, {"Q": 12}),
            ({"I1": 0, "I2": 99}, {"Q": 0}),
        ],
    },
    "Divide": {
        "inputs": {"I1": "analog", "I2": "analog"},
        "outputs": {"Q": "analog"},
        "cases": [
            ({"I1": 12, "I2": 4}, {"Q": 3}),
            ({"I1": 7, "I2": 2}, {"Q": 3.5}),
        ],
    },
    "Comparator": {
        "inputs": {"I1": "analog", "I2": "analog"},
        "outputs": {"GT": "digital", "EQ": "digital", "LT": "digital"},
        "cases": [
            ({"I1": 5, "I2": 3}, {"GT": 1, "EQ": 0, "LT": 0}),
            ({"I1": 3, "I2": 3}, {"GT": 0, "EQ": 1, "LT": 0}),
            ({"I1": 1, "I2": 3}, {"GT": 0, "EQ": 0, "LT": 1}),
        ],
    },
    "FlipFlopRS": {
        "inputs": {"S": "digital", "R": "digital"},
        "outputs": {"Q": "digital"},
        "cases": [
            ({"S": 1, "R": 0}, {"Q": 1}),
            ({"S": 0, "R": 0}, {"Q": 1}),
            ({"S": 0, "R": 1}, {"Q": 0}),
            ({"S": 0, "R": 0}, {"Q": 0}),
        ],
    },
}

CATEGORIES = {
    "logic": ["And", "Or", "Not", "Xor"],
    "math": ["Add2", "Subtract", "Multiply", "Divide"],
    "comparison": ["Comparator"],
    "stateful": ["FlipFlopRS"],
}


def lox_uuid():
    h = uuid_mod.uuid4().hex
    return f"{h[:8]}-{h[8:12]}-{h[12:16]}-ffff000000000000"


def http_get(path):
    url = f"{HOST}/{path}"
    req = urllib.request.Request(url)
    creds = base64.b64encode(f"{USER}:{PASS}".encode()).decode()
    req.add_header("Authorization", f"Basic {creds}")
    try:
        resp = urllib.request.urlopen(req, timeout=10)
        return json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        return {"error": str(e)}


def set_vi(name, value):
    encoded = name.replace(" ", "%20")
    r = http_get(f"jdev/sps/io/{encoded}/{value}")
    return r.get("LL", {}).get("Code", "") == "200"


def read_sv(uuid):
    r = http_get(f"jdev/sps/io/{uuid}/state")
    try:
        return float(r["LL"]["value"])
    except Exception:
        return None


def wait_for_miniserver():
    """Wait for Miniserver to come back online after reboot."""
    print(f"  Waiting {REBOOT_WAIT}s for reboot...", end="", flush=True)
    for i in range(REBOOT_WAIT):
        time.sleep(1)
        if i % 10 == 9:
            print(".", end="", flush=True)
    print(" checking...", flush=True)
    for _ in range(30):
        try:
            r = urllib.request.urlopen(f"{HOST}/dev/sys/check", timeout=3)
            if r.status == 200:
                print("  Miniserver online!")
                time.sleep(3)  # Extra settle time
                return True
        except Exception:
            pass
        time.sleep(2)
    print("  WARNING: Miniserver not responding")
    return False


def inject_test_circuit(xml_bytes, block_type, spec):
    """Inject VirtualIn → Block → VirtualState test circuit into XML.
    
    Returns: (modified_xml_bytes, vi_names_dict, sv_uuids_dict)
    """
    if xml_bytes[:3] == b'\xef\xbb\xbf':
        bom = xml_bytes[:3]
        xml_str = xml_bytes[3:].decode('utf-8')
    else:
        bom = b''
        xml_str = xml_bytes.decode('utf-8')

    root = ET.fromstring(xml_str)
    inputs = spec["inputs"]
    outputs = spec["outputs"]

    # Find the page to add elements to
    page = None
    for elem in root.iter('C'):
        if elem.get('Type') == 'Page':
            page = elem
            break
    if page is None:
        # Create a page
        page = ET.SubElement(root, 'C', Type='Page', Title='TestPage', U=lox_uuid(), V='175')

    # Track what we create
    vi_names = {}   # input_key → VirtualIn name
    sv_uuids = {}   # output_key → StateV UUID

    prefix = f"T_{block_type}"

    # Create the block under test
    block_uuid = lox_uuid()
    block_elem = ET.SubElement(page, 'C', Type=block_type, Title=f"{prefix}_Block", U=block_uuid, V='175')
    io_elem = ET.SubElement(block_elem, 'Io')

    # Create connectors for the block
    block_conn_uuids = {}
    for key in list(inputs.keys()) + list(outputs.keys()):
        co_uuid = lox_uuid()
        is_analog = spec.get("inputs", {}).get(key, spec.get("outputs", {}).get(key, "")) == "analog"
        co_type = "AI" if (key in inputs and is_analog) else \
                  "DI" if key in inputs else \
                  "AQ" if (key in outputs and is_analog) else "Q"
        co = ET.SubElement(io_elem, 'Co', K=key, U=co_uuid, Type=co_type)
        block_conn_uuids[key] = co_uuid

    # For each INPUT: create a VirtualIn + wire to block input
    for key, kind in inputs.items():
        vi_uuid = lox_uuid()
        vi_name = f"{prefix}_{key}"
        vi_names[key] = vi_name
        is_analog = (kind == "analog")

        vi_elem = ET.SubElement(page, 'C', Type='VirtualIn', Title=vi_name, U=vi_uuid, V='175')
        vi_io = ET.SubElement(vi_elem, 'Io')

        # VirtualIn has an output connector (AQ for analog, Q for digital)
        vi_out_key = "AQ" if is_analog else "Q"
        vi_out_uuid = lox_uuid()
        vi_co = ET.SubElement(vi_io, 'Co', K=vi_out_key, U=vi_out_uuid,
                               Type="AQ" if is_analog else "Q")

        # Also need an InputRef converter inside the VirtualIn
        conv_uuid = lox_uuid()
        conv_aq_uuid = lox_uuid()
        conv = ET.SubElement(vi_elem, 'C', Type='InputRef', U=conv_uuid, V='175')
        conv_io = ET.SubElement(conv, 'Io')
        ET.SubElement(conv_io, 'Co', K='AQ', U=conv_aq_uuid, Type='AQ')

        # Wire: block input ← converter AQ output
        block_co = io_elem.find(f".//Co[@K='{key}']")
        if block_co is not None:
            in_elem = ET.SubElement(block_co, 'In', Input=conv_aq_uuid)

    # For each OUTPUT: create a VirtualState (StateV) ← block output
    for key, kind in outputs.items():
        sv_uuid = lox_uuid()
        sv_uuids[key] = sv_uuid

        sv_elem = ET.SubElement(page, 'C', Type='VirtualState', Title=f"{prefix}_{key}_SV", U=sv_uuid, V='175')
        sv_io = ET.SubElement(sv_elem, 'Io')

        is_analog = (kind == "analog")
        sv_in_key = "AI" if is_analog else "I"
        sv_co_uuid = lox_uuid()
        sv_co = ET.SubElement(sv_io, 'Co', K=sv_in_key, U=sv_co_uuid,
                               Type="AI" if is_analog else "I")

        # Wire: StateV input ← block output
        in_elem = ET.SubElement(sv_co, 'In', Input=block_conn_uuids[key])

    # Serialize back
    result = ET.tostring(root, encoding='unicode', xml_declaration=True)
    return bom + result.encode('utf-8'), vi_names, sv_uuids


def run_tests(block_type, spec, vi_names, sv_uuids):
    """Run test cases against the live Miniserver."""
    cases = spec["cases"]
    passed = 0
    failed = 0
    errors = []

    for i, (inputs, expected) in enumerate(cases):
        # Set all inputs
        for key, value in inputs.items():
            vi_name = vi_names[key]
            if not set_vi(vi_name, value):
                errors.append(f"[{i}] Could not set {vi_name}={value}")
                failed += 1
                continue

        time.sleep(SETTLE)

        # Read all outputs
        case_ok = True
        for key, exp_val in expected.items():
            sv_uuid = sv_uuids[key]
            actual = read_sv(sv_uuid)
            if actual is None:
                errors.append(f"[{i}] Could not read {key} (UUID: {sv_uuid[:16]}...)")
                case_ok = False
            elif abs(actual - exp_val) > 0.01:
                errors.append(f"[{i}] {key}: expected={exp_val} actual={actual}")
                case_ok = False

        if case_ok:
            passed += 1
        else:
            failed += 1

    return passed, failed, errors


def main():
    parser = argparse.ArgumentParser(description="Inject blocks, upload, and test against Miniserver")
    parser.add_argument("--types", help="Comma-separated block types (e.g. And,Or,Not)")
    parser.add_argument("--category", choices=list(CATEGORIES.keys()),
                        help="Test category (logic, math, comparison, stateful)")
    parser.add_argument("--all-testable", action="store_true", help="Test all defined types")
    parser.add_argument("--inject-only", action="store_true", help="Inject but don't upload/test")
    parser.add_argument("--output", default="/tmp/test_config.Loxone", help="Output XML file")
    args = parser.parse_args()

    # Determine which types to test
    if args.all_testable:
        test_types = list(TESTS.keys())
    elif args.category:
        test_types = CATEGORIES[args.category]
    elif args.types:
        test_types = [t.strip() for t in args.types.split(",")]
    else:
        parser.print_help()
        return

    # Filter to types with test definitions
    test_types = [t for t in test_types if t in TESTS]
    if not test_types:
        print("No test definitions for specified types")
        return

    print(f"=== Testing {len(test_types)} block types: {', '.join(test_types)} ===\n")

    # Step 1: Download current config
    print("Step 1: Downloading current config...")
    result = subprocess.run(["lox", "config", "download", "-q"], capture_output=True, text=True)
    zips = sorted(
        [f for f in os.listdir(".") if f.startswith("sps_") and f.endswith(".zip")],
        key=lambda f: os.path.getmtime(f), reverse=True,
    )
    if not zips:
        print("ERROR: No config ZIP found")
        return
    zip_file = zips[0]
    print(f"  Using: {zip_file}")

    # Extract
    subprocess.run(["lox", "config", "extract", zip_file], capture_output=True)
    loxone_file = zip_file.replace(".zip", ".Loxone")
    with open(loxone_file, "rb") as f:
        xml_bytes = f.read()
    print(f"  Config: {len(xml_bytes)} bytes")

    # Step 2: Inject all test circuits
    print("\nStep 2: Injecting test circuits...")
    all_vi_names = {}
    all_sv_uuids = {}
    modified = xml_bytes

    for block_type in test_types:
        spec = TESTS[block_type]
        modified, vi_names, sv_uuids = inject_test_circuit(modified, block_type, spec)
        all_vi_names[block_type] = vi_names
        all_sv_uuids[block_type] = sv_uuids
        n_in = len(spec["inputs"])
        n_out = len(spec["outputs"])
        n_cases = len(spec["cases"])
        print(f"  ✓ {block_type}: {n_in} inputs, {n_out} outputs, {n_cases} test cases")

    # Save modified config
    with open(args.output, "wb") as f:
        f.write(modified)
    print(f"  Saved: {args.output} ({len(modified)} bytes)")

    # Save the UUID mapping for reference
    mapping = {"vi_names": all_vi_names, "sv_uuids": all_sv_uuids}
    with open("/tmp/test_mapping.json", "w") as f:
        json.dump(mapping, f, indent=2)

    if args.inject_only:
        print("\n[inject-only] Skipping upload and test.")
        return

    # Step 3: Upload via lox config push
    print("\nStep 3: Uploading config...")
    result = subprocess.run(
        ["lox", "config", "push", args.output, "--reboot", "--force"],
        capture_output=True, text=True,
    )
    print(f"  {result.stdout.strip()}")
    if result.returncode != 0:
        print(f"  ERROR: {result.stderr.strip()}")
        # Try manual FTP upload + reboot
        print("  Falling back to manual FTP upload...")
        # First recompress
        subprocess.run(["lox", "config", "compress", args.output], capture_output=True)
        compressed = args.output.replace(".Loxone", ".zip")
        if not os.path.exists(compressed):
            compressed = args.output + ".zip"
        if os.path.exists(compressed):
            subprocess.run(
                ["lox", "config", "upload", compressed, "--force"],
                capture_output=True, text=True,
            )
            # Trigger reboot
            http_get("jdev/sys/reboot")
            print("  Reboot triggered")
        else:
            print("  ERROR: Could not compress config")
            return

    # Step 4: Wait for reboot
    print("\nStep 4: Waiting for Miniserver reboot...")
    if not wait_for_miniserver():
        print("ERROR: Miniserver did not come back online")
        return

    # Step 5: Run tests
    print("\nStep 5: Running behavioral tests...\n")
    total_pass = 0
    total_fail = 0
    results = {}

    for block_type in test_types:
        spec = TESTS[block_type]
        vi_names = all_vi_names[block_type]
        sv_uuids = all_sv_uuids[block_type]

        passed, failed, errors = run_tests(block_type, spec, vi_names, sv_uuids)
        total_pass += passed
        total_fail += failed
        results[block_type] = {"passed": passed, "failed": failed, "errors": errors}

        status = "✓ PASS" if failed == 0 else "✗ FAIL"
        print(f"  {block_type}: {status} ({passed}/{passed + failed})")
        for err in errors:
            print(f"    {err}")

    # Summary
    print(f"\n{'='*50}")
    print(f"TOTAL: {total_pass} PASS, {total_fail} FAIL ({total_pass + total_fail} tests)")
    print(f"{'='*50}")

    # Save results
    with open("/tmp/test_results.json", "w") as f:
        json.dump(results, f, indent=2)

    return total_fail == 0


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
