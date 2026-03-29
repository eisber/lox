#!/usr/bin/env python3
"""
Structural validation: verify crosswalk.json connector definitions
against the actual XML config downloaded from the Miniserver.

Checks:
- Connector keys in crosswalk match those in XML
- Connector count matches
- Type names are valid

Usage:
    python3 scripts/validate_structural.py [config.xml]
"""

import json, sys, os
import xml.etree.ElementTree as ET

def load_crosswalk():
    """Load crosswalk.json types."""
    path = os.path.join(os.path.dirname(__file__), '..', 'docs', 'schemas', 'crosswalk.json')
    with open(path) as f:
        return json.load(f)['types']

def load_config_xml(path):
    """Load and parse a Loxone config XML."""
    tree = ET.parse(path)
    root = tree.getroot()
    blocks = {}
    for c in root.iter('C'):
        t = c.get('Type', '')
        if not t:
            continue
        connectors = {}
        for co in c.iter('Co'):
            k = co.get('K', '')
            if k:
                connectors[k] = {
                    'uuid': co.get('U', ''),
                    'type': co.get('Type', ''),
                }
        if t not in blocks:
            blocks[t] = {'title': c.get('Title', ''), 'connectors': connectors}
    return blocks

def validate(crosswalk, config_blocks):
    """Compare crosswalk against config."""
    results = {'matched': 0, 'mismatched': 0, 'missing_in_config': 0, 'extra_in_config': 0}
    
    crosswalk_types = set(crosswalk.keys())
    config_types = set(config_blocks.keys())
    
    # Types in crosswalk but not in config
    for t in sorted(crosswalk_types - config_types):
        results['missing_in_config'] += 1
    
    # Types in config but not in crosswalk
    for t in sorted(config_types - crosswalk_types):
        results['extra_in_config'] += 1
    
    # Types in both — compare connectors
    common = crosswalk_types & config_types
    print(f"\nValidating {len(common)} types present in both crosswalk and config:")
    
    for t in sorted(common):
        cw_conns = set(crosswalk[t].get('connectors', {}).keys())
        cfg_conns = set(config_blocks[t].get('connectors', {}).keys())
        
        if cw_conns == cfg_conns:
            results['matched'] += 1
        else:
            results['mismatched'] += 1
            missing = cw_conns - cfg_conns
            extra = cfg_conns - cw_conns
            if missing or extra:
                print(f"  {t}: ", end='')
                if missing:
                    print(f"missing={missing} ", end='')
                if extra:
                    print(f"extra={extra}", end='')
                print()
    
    return results

def main():
    crosswalk = load_crosswalk()
    print(f"Crosswalk: {len(crosswalk)} types")
    
    if len(sys.argv) > 1:
        config_xml = sys.argv[1]
    else:
        # Try to find a config XML in common locations
        candidates = ['/tmp/lox_config/sps0.xml', 'sps0.xml']
        config_xml = None
        for c in candidates:
            if os.path.exists(c):
                config_xml = c
                break
        if not config_xml:
            print("No config XML found. Download with: lox config download")
            print("Then decompress with the LZ4 extractor.")
            sys.exit(1)
    
    print(f"Config: {config_xml}")
    config_blocks = load_config_xml(config_xml)
    print(f"Config blocks: {len(config_blocks)} types")
    
    results = validate(crosswalk, config_blocks)
    
    print(f"\n{'='*40}")
    print(f"Matched:            {results['matched']}")
    print(f"Connector mismatch: {results['mismatched']}")
    print(f"In crosswalk only:  {results['missing_in_config']}")
    print(f"In config only:     {results['extra_in_config']}")
    print(f"{'='*40}")
    
    return results['mismatched'] == 0

if __name__ == '__main__':
    ok = main()
    sys.exit(0 if ok else 1)

