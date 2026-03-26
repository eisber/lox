#!/usr/bin/env python3
"""lox-validate — Visual validation of Loxone Config state.

Checks the running Loxone Config application via screenshots, OCR,
and FlaUI accessibility inspection.

Usage:
    python3 lox-validate.py                         # basic check
    python3 lox-validate.py --require-connected     # must be connected
    python3 lox-validate.py --tree-contains "text"  # verify tree item
    python3 lox-validate.py --screenshot proof.png  # save screenshot
"""

import argparse
import json
import sys

def main():
    parser = argparse.ArgumentParser(description="Validate Loxone Config UX state")
    parser.add_argument("--require-connected", action="store_true",
                        help="Fail if not connected to Miniserver")
    parser.add_argument("--tree-contains", type=str,
                        help="Verify tree panel contains this text")
    parser.add_argument("--screenshot", type=str,
                        help="Save screenshot to this path")
    parser.add_argument("--json", action="store_true",
                        help="Output JSON result")
    args = parser.parse_args()

    try:
        from loxone_ux import validate_live_ui
    except ImportError:
        print("Error: loxone_ux.py not found. Copy UX tools to this directory.", file=sys.stderr)
        print("Required files: loxone_ux.py, winclick.py, flaui-bridge.py, tree_ocr.py", file=sys.stderr)
        sys.exit(1)

    result = validate_live_ui(
        screenshot_path=args.screenshot,
        require_connected=args.require_connected,
    )

    if args.tree_contains and result.get("validated"):
        tree_items = result.get("tree_items", [])
        tree_text = " ".join(item.get("text", "") for item in tree_items)
        if args.tree_contains.lower() not in tree_text.lower():
            result["validated"] = False
            result["tree_check"] = f"'{args.tree_contains}' not found in tree"

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        status = "✓ PASS" if result.get("validated") else "✗ FAIL"
        print(f"{status}: {result.get('summary', 'no summary')}")
        if not result.get("validated"):
            for check, val in result.get("checks", {}).items():
                if not val:
                    print(f"  ✗ {check}")

    sys.exit(0 if result.get("validated") else 1)

if __name__ == "__main__":
    main()
