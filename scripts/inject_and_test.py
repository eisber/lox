#!/usr/bin/env python3
"""
Inject test circuits into Loxone config and validate via HTTP API.

IMPORTANT: Uses `lox` CLI for all XML manipulation to preserve
BOM, CRLF, and XML declaration formatting. Python ET.tostring()
corrupts these and breaks Miniserver auth.

Pipeline: CLI inject → FTP upload → reboot → HTTP test

Usage:
  export LOX_PASS="yourpass"
  python3 inject_and_test.py --category logic
  python3 inject_and_test.py --types And,Or
  python3 inject_and_test.py --inject-only
"""

import argparse, base64, json, os, re, shutil, subprocess, sys, time, urllib.request

HOST = os.environ.get("LOX_HOST", "http://192.168.68.72")
USER = os.environ.get("LOX_USER", "admin")
PASS = os.environ.get("LOX_PASS", "")
SETTLE = float(os.environ.get("LOX_SETTLE", "1.5"))
REBOOT_WAIT = int(os.environ.get("LOX_REBOOT_WAIT", "75"))

if not PASS:
    print("ERROR: Set LOX_PASS environment variable"); sys.exit(1)

# ── Test definitions ──────────────────────────────────────────────────

TESTS = {
    "And":  {"in": {"I1": "d", "I2": "d"}, "out": {"Q": "d"}, "cases": [
        ({"I1":0,"I2":0}, {"Q":0}), ({"I1":0,"I2":1}, {"Q":0}),
        ({"I1":1,"I2":0}, {"Q":0}), ({"I1":1,"I2":1}, {"Q":1})]},
    "Or":   {"in": {"I1": "d", "I2": "d"}, "out": {"Q": "d"}, "cases": [
        ({"I1":0,"I2":0}, {"Q":0}), ({"I1":0,"I2":1}, {"Q":1}),
        ({"I1":1,"I2":0}, {"Q":1}), ({"I1":1,"I2":1}, {"Q":1})]},
    "Not":  {"in": {"I": "d"}, "out": {"Q": "d"}, "cases": [
        ({"I":0}, {"Q":1}), ({"I":1}, {"Q":0})]},
    "Xor":  {"in": {"I1": "d", "I2": "d"}, "out": {"Q": "d"}, "cases": [
        ({"I1":0,"I2":0}, {"Q":0}), ({"I1":0,"I2":1}, {"Q":1}),
        ({"I1":1,"I2":0}, {"Q":1}), ({"I1":1,"I2":1}, {"Q":0})]},
    "Add2": {"in": {"I1": "a", "I2": "a"}, "out": {"Q": "a"}, "cases": [
        ({"I1":3,"I2":5}, {"Q":8}), ({"I1":-2,"I2":7}, {"Q":5}),
        ({"I1":0,"I2":0}, {"Q":0})]},
    "Subtract": {"in": {"I1": "a", "I2": "a"}, "out": {"Q": "a"}, "cases": [
        ({"I1":10,"I2":3}, {"Q":7}), ({"I1":5,"I2":5}, {"Q":0})]},
    "Multiply": {"in": {"I1": "a", "I2": "a"}, "out": {"Q": "a"}, "cases": [
        ({"I1":3,"I2":4}, {"Q":12}), ({"I1":0,"I2":99}, {"Q":0})]},
    "Divide": {"in": {"I1": "a", "I2": "a"}, "out": {"Q": "a"}, "cases": [
        ({"I1":12,"I2":4}, {"Q":3}), ({"I1":7,"I2":2}, {"Q":3.5})]},
    "Comparator": {"in": {"I1": "a", "I2": "a"}, "out": {"GT":"d","EQ":"d","LT":"d"}, "cases": [
        ({"I1":5,"I2":3}, {"GT":1,"EQ":0,"LT":0}),
        ({"I1":3,"I2":3}, {"GT":0,"EQ":1,"LT":0}),
        ({"I1":1,"I2":3}, {"GT":0,"EQ":0,"LT":1})]},
    "FlipFlopRS": {"in": {"S":"d","R":"d"}, "out": {"Q":"d"}, "cases": [
        ({"S":1,"R":0}, {"Q":1}), ({"S":0,"R":0}, {"Q":1}),
        ({"S":0,"R":1}, {"Q":0}), ({"S":0,"R":0}, {"Q":0})]},
}

CATEGORIES = {
    "logic": ["And", "Or", "Not", "Xor"],
    "math": ["Add2", "Subtract", "Multiply", "Divide"],
    "comparison": ["Comparator"],
    "stateful": ["FlipFlopRS"],
}


def lox(*args):
    """Run lox CLI command, return (stdout, success)."""
    r = subprocess.run(["lox"] + list(args), capture_output=True, text=True)
    return r.stdout.strip(), r.returncode == 0


def http_get(path):
    """HTTP GET with basic auth. Returns parsed JSON or None."""
    url = f"{HOST}/{path}"
    req = urllib.request.Request(url)
    creds = base64.b64encode(f"{USER}:{PASS}".encode()).decode()
    req.add_header("Authorization", f"Basic {creds}")
    try:
        resp = urllib.request.urlopen(req, timeout=10)
        data = json.loads(resp.read())
        return data
    except urllib.error.HTTPError as e:
        if e.code == 403:
            body = e.read().decode()
            m = re.search(r'remaining.*?(\d+)', body)
            if m:
                print(f"\n⚠ AUTH LOCKED OUT — {m.group(1)}s remaining. ABORTING.", file=sys.stderr)
                sys.exit(2)
        return None
    except Exception:
        return None


def check_auth():
    """Verify auth works with ONE request. Abort on lockout."""
    r = http_get("jdev/cfg/api")
    if r and r.get("LL", {}).get("Code") == "200":
        return True
    print("⚠ Cannot reach Miniserver or auth failed", file=sys.stderr)
    return False


def set_vi(name, value):
    r = http_get(f"jdev/sps/io/{name.replace(' ', '%20')}/{value}")
    return r and r.get("LL", {}).get("Code") == "200"


def read_sv(uuid):
    r = http_get(f"jdev/sps/io/{uuid}/state")
    try:
        return float(r["LL"]["value"])
    except:
        return None


def inject_circuit(config_file, block_type, spec):
    """Inject VirtualIn→Block→VirtualState using CLI. Returns (vi_names, sv_uuids)."""
    prefix = f"T_{block_type}"
    vi_names = {}
    sv_uuids = {}

    # Add block
    out, ok = lox("config", "add", "--type", block_type, "--title", f"{prefix}_Blk", config_file)
    if not ok:
        print(f"  ✗ Add {block_type}: {out}"); return None, None

    # Add VirtualIns for each input
    for key, kind in spec["in"].items():
        vi_name = f"{prefix}_{key}"
        vi_names[key] = vi_name
        cmd = ["config", "add-virtual-in", config_file, vi_name]
        if kind == "a":
            cmd.append("--analog")
        out, ok = lox(*cmd)
        if not ok:
            print(f"  ✗ Add VI {vi_name}: {out}"); continue
        # Extract connector UUID and wire
        m = re.search(r'UUID:\s*([a-f0-9-]+)', out)
        if m:
            vi_uuid = m.group(1)
            w_out, w_ok = lox("config", "wire-connector", config_file, f"{prefix}_Blk.{key}", vi_uuid)
            if not w_ok:
                print(f"  ⚠ Wire {vi_name}→{key}: {w_out}")

    # Add VirtualState for each output
    for key, kind in spec["out"].items():
        sv_name = f"{prefix}_{key}_SV"
        out, ok = lox("config", "add", "--type", "VirtualState", "--title", sv_name, config_file)
        if ok:
            m = re.search(r'UUID:\s*([a-f0-9-]+)', out)
            if m:
                sv_uuids[key] = m.group(1)

    return vi_names, sv_uuids


def verify_xml_integrity(filepath):
    """Check that XML preserves BOM, CRLF, double-quoted declaration."""
    with open(filepath, "rb") as f:
        data = f.read()
    issues = []
    if not data.startswith(b'\xef\xbb\xbf'):
        issues.append("Missing BOM")
    if b'\r\n' not in data:
        issues.append("No CRLF line endings")
    if b"version='1.0'" in data[:100]:
        issues.append("Single-quoted XML declaration")
    bare_lf = data.count(b'\n') - data.count(b'\r\n')
    if bare_lf > 0:
        issues.append(f"{bare_lf} bare LF line endings")
    return issues


def main():
    parser = argparse.ArgumentParser(description="Inject + test Loxone blocks")
    parser.add_argument("--types", help="Comma-separated types")
    parser.add_argument("--category", choices=list(CATEGORIES.keys()))
    parser.add_argument("--all-testable", action="store_true")
    parser.add_argument("--inject-only", action="store_true")
    parser.add_argument("--list", action="store_true")
    args = parser.parse_args()

    if args.list:
        for cat, types in CATEGORIES.items():
            n = sum(len(TESTS[t]["cases"]) for t in types)
            print(f"  {cat}: {', '.join(types)} ({n} tests)")
        return

    types = []
    if args.all_testable: types = list(TESTS.keys())
    elif args.category: types = CATEGORIES[args.category]
    elif args.types: types = [t.strip() for t in args.types.split(",")]
    types = [t for t in types if t in TESTS]
    if not types:
        parser.print_help(); return

    print(f"=== Testing {len(types)} types: {', '.join(types)} ===\n")

    # Step 1: Download config
    print("1. Downloading config...")
    lox("config", "download", "-q")
    zips = sorted([f for f in os.listdir(".") if f.startswith("sps_") and f.endswith(".zip")],
                  key=os.path.getmtime, reverse=True)
    if not zips:
        print("ERROR: No config ZIP"); return
    zip_file = zips[0]
    lox("config", "extract", zip_file)
    config = zip_file.replace(".zip", ".Loxone")
    work = f"/tmp/inject_test.Loxone"
    shutil.copy(config, work)
    print(f"   Using: {config}")

    # Step 2: Inject using CLI
    print("\n2. Injecting test circuits (via CLI)...")
    all_vi = {}; all_sv = {}
    for t in types:
        vi, sv = inject_circuit(work, t, TESTS[t])
        if vi is not None:
            all_vi[t] = vi; all_sv[t] = sv
            n_in = len(TESTS[t]["in"]); n_out = len(TESTS[t]["out"])
            print(f"  ✓ {t}: {n_in}in, {n_out}out")

    # Verify XML integrity
    issues = verify_xml_integrity(work)
    if issues:
        print(f"\n  ⚠ XML INTEGRITY ISSUES: {', '.join(issues)}")
        print("  This will likely corrupt auth. ABORTING.")
        return
    print(f"  ✓ XML integrity OK (BOM, CRLF, declaration)")

    # Save mapping
    json.dump({"vi": all_vi, "sv": all_sv}, open("/tmp/test_mapping.json", "w"), indent=2)

    if args.inject_only:
        print(f"\n[inject-only] Config saved to {work}")
        return

    # Step 3: Upload
    print("\n3. Uploading config...")
    out, ok = lox("config", "push", work, "--reboot", "--force")
    print(f"   {out}")
    if not ok:
        print("   Upload failed, aborting"); return

    # Step 4: Wait for reboot
    print(f"\n4. Waiting {REBOOT_WAIT}s for reboot...")
    time.sleep(REBOOT_WAIT)
    for _ in range(30):
        try:
            urllib.request.urlopen(f"{HOST}/dev/sys/check", timeout=3)
            print("   Online!")
            time.sleep(3)
            break
        except:
            time.sleep(2)

    # Step 5: Test
    print("\n5. Running tests...")
    if not check_auth():
        return

    total_p = total_f = 0
    for t in types:
        if t not in all_vi:
            continue
        vi = all_vi[t]; sv = all_sv[t]
        p = f = 0
        for inputs, expected in TESTS[t]["cases"]:
            for k, v in inputs.items():
                set_vi(vi[k], v)
            time.sleep(SETTLE)
            ok = True
            for k, exp in expected.items():
                if k not in sv:
                    ok = False; continue
                actual = read_sv(sv[k])
                if actual is None or abs(actual - exp) > 0.01:
                    ok = False
                    print(f"  {t} FAIL: {k} exp={exp} got={actual}")
            if ok: p += 1
            else: f += 1
        status = "✓ PASS" if f == 0 else "✗ FAIL"
        print(f"  {t}: {status} ({p}/{p+f})")
        total_p += p; total_f += f

    print(f"\nTOTAL: {total_p} PASS, {total_f} FAIL")


if __name__ == "__main__":
    main()
