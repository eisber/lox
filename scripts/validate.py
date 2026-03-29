#!/usr/bin/env python3
"""
Generalized Loxone block validation harness.

Usage:
    python3 scripts/validate.py tests/vectors/comparison.json
    python3 scripts/validate.py tests/vectors/*.json

Each JSON file contains test definitions:
{
  "category": "Comparison",
  "blocks": {
    "Comparator": {
      "inputs": {"I1": "analog", "I2": "analog"},
      "outputs": {"GT": "digital", "EQ": "digital", "LT": "digital"},
      "tests": [
        {"set": {"I1": 5.0, "I2": 3.0}, "expect": {"GT": 1, "EQ": 0, "LT": 0}},
        {"set": {"I1": 3.0, "I2": 5.0}, "expect": {"GT": 0, "EQ": 0, "LT": 1}},
        {"set": {"I1": 4.0, "I2": 4.0}, "expect": {"GT": 0, "EQ": 1, "LT": 0}}
      ]
    }
  }
}
"""

import json, sys, os, time, urllib.request, urllib.error

HOST = os.environ.get("LOX_HOST", "http://192.168.68.72")
USER = os.environ.get("LOX_USER", "admin")
PASS = os.environ.get("LOX_PASS", "")
SETTLE_TIME = float(os.environ.get("LOX_SETTLE", "0.5"))
TOLERANCE = float(os.environ.get("LOX_TOLERANCE", "0.01"))

def http_get(path):
    """GET request with basic auth."""
    url = f"{HOST}/{path}"
    req = urllib.request.Request(url)
    import base64
    creds = base64.b64encode(f"{USER}:{PASS}".encode()).decode()
    req.add_header("Authorization", f"Basic {creds}")
    try:
        resp = urllib.request.urlopen(req, timeout=5)
        return json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        print(f"  HTTP {e.code} for {path}")
        return None

def set_input(name, value):
    """Set a VirtualIn value."""
    encoded = urllib.parse.quote(name)
    return http_get(f"jdev/sps/io/{encoded}/{value}")

def read_output(uuid):
    """Read a StateV value by UUID."""
    resp = http_get(f"jdev/sps/io/{uuid}/state")
    if resp and "LL" in resp:
        return float(resp["LL"]["value"])
    return None

def find_block_uuid(config_xml, block_title, conn_key):
    """Find the UUID of a connector in the config XML."""
    import xml.etree.ElementTree as ET
    root = ET.parse(config_xml).getroot()
    for c in root.iter('C'):
        if c.get('Title') == block_title:
            for co in c.iter('Co'):
                if co.get('K') == conn_key:
                    return co.get('U')
    return None

def run_tests(vectors_file):
    """Run all tests from a vectors JSON file."""
    with open(vectors_file) as f:
        spec = json.load(f)
    
    category = spec.get("category", "Unknown")
    wiring = spec.get("wiring", {})  # UUID mapping
    blocks = spec.get("blocks", {})
    
    total = passed = failed = skipped = 0
    
    print(f"\n{'='*60}")
    print(f"Category: {category} ({len(blocks)} block types)")
    print(f"{'='*60}")
    
    for block_name, block_spec in blocks.items():
        inputs = block_spec.get("inputs", {})
        outputs = block_spec.get("outputs", {})
        tests = block_spec.get("tests", [])
        vi_prefix = block_spec.get("vi_prefix", f"VI_{block_name}")
        
        # Check wiring exists
        block_wiring = wiring.get(block_name, {})
        if not block_wiring:
            print(f"\n  {block_name}: SKIPPED (no wiring configured)")
            skipped += len(tests)
            continue
        
        print(f"\n  {block_name} ({len(tests)} tests)")
        
        for i, test in enumerate(tests):
            total += 1
            set_vals = test.get("set", {})
            expected = test.get("expect", {})
            desc = test.get("desc", f"test {i+1}")
            wait = test.get("wait", SETTLE_TIME)
            
            # Set all inputs
            for inp_key, value in set_vals.items():
                vi_name = block_wiring.get(f"vi_{inp_key}", f"{vi_prefix}_{inp_key}")
                result = set_input(vi_name, value)
                if result is None:
                    print(f"    {desc}: FAIL (cannot set {vi_name}={value})")
                    failed += 1
                    continue
            
            time.sleep(wait)
            
            # Read and verify all outputs
            all_ok = True
            details = []
            for out_key, exp_value in expected.items():
                out_uuid = block_wiring.get(f"out_{out_key}")
                if not out_uuid:
                    details.append(f"{out_key}=NO_UUID")
                    all_ok = False
                    continue
                
                actual = read_output(out_uuid)
                if actual is None:
                    details.append(f"{out_key}=READ_FAIL")
                    all_ok = False
                    continue
                
                # Compare with tolerance
                if isinstance(exp_value, (int, float)):
                    if abs(actual - exp_value) > TOLERANCE:
                        details.append(f"{out_key}={actual}≠{exp_value}")
                        all_ok = False
                    else:
                        details.append(f"{out_key}={actual}✓")
                else:
                    details.append(f"{out_key}={actual}")
            
            status = "PASS" if all_ok else "FAIL"
            if all_ok:
                passed += 1
            else:
                failed += 1
            
            inputs_str = ", ".join(f"{k}={v}" for k, v in set_vals.items())
            details_str = ", ".join(details)
            print(f"    {status} {desc}: ({inputs_str}) → {details_str}")
    
    print(f"\n{'='*60}")
    print(f"Results: {passed} PASS, {failed} FAIL, {skipped} SKIP / {total+skipped} total")
    print(f"{'='*60}")
    
    return failed == 0

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 scripts/validate.py <vectors.json> [...]")
        sys.exit(1)
    
    all_ok = True
    for f in sys.argv[1:]:
        if not run_tests(f):
            all_ok = False
    
    sys.exit(0 if all_ok else 1)
