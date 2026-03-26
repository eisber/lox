#!/bin/bash
# validate-catA.sh — Cat A logic gate truth tables (14 tests)
#
# Architecture:
#   VirtualIn (set via HTTP) → Block input → Block Q output → StateV (read via HTTP)
#
# Prerequisites:
#   - Cat A blocks wired: VI1,VI2→And, VI3,VI4→Or, VI5→Not, VI6,VI7→Xor
#   - Block Q outputs wired to Virtual Status (StateV) elements
#   - Config saved to Miniserver via Loxone Config UX

set -euo pipefail

HOST="${LOX_HOST:-http://192.168.68.72}"
CREDS="${LOX_CREDS:-admin:admin}"
PASS=0; FAIL=0

# StateV UUIDs (block Q → Virtual Status, readable via /sps/io/{uuid}/state)
AND_VS="${AND_VS:-20699539-0224-4b37-ffff234d69b98eb1}"
OR_VS="${OR_VS:-2069954d-01d1-62dd-ffff234d69b98eb1}"
NOT_VS="${NOT_VS:-20699556-020e-6eb5-ffff234d69b98eb1}"
XOR_VS="${XOR_VS:-2069955f-038b-7a99-ffff234d69b98eb1}"

set_vi() { curl -sf "$HOST/jdev/sps/io/Eingang%20VI$1/$2" -u "$CREDS" > /dev/null; }
read_vs() { curl -sf "$HOST/jdev/sps/io/$1/state" -u "$CREDS" | python3 -c "import json,sys; print(int(float(json.load(sys.stdin)['LL']['value'])))" 2>/dev/null; }

preflight() {
    echo "Pre-flight checks..."
    local code
    code=$(curl -sf -o /dev/null -w "%{http_code}" "$HOST/jdev/cfg/api" -u "$CREDS" 2>/dev/null)
    [ "$code" = "200" ] && echo "  ✓ Miniserver reachable" || { echo "  ✗ Miniserver unreachable"; exit 1; }

    local val
    val=$(read_vs "$AND_VS")
    [ -n "$val" ] && echo "  ✓ StateV readable (And=$val)" || { echo "  ✗ StateV not readable"; exit 1; }
    echo ""
}

test_gate() {
    local name=$1 vs=$2 vi1=$3 vi2=$4 v1=$5 v2=$6 expected=$7
    set_vi "$vi1" "$v1"; set_vi "$vi2" "$v2"
    sleep 0.3
    local actual
    actual=$(read_vs "$vs")
    if [ "$actual" = "$expected" ]; then
        echo "  ✓ $name($v1,$v2) = $actual"
        PASS=$((PASS+1))
    else
        echo "  ✗ $name($v1,$v2) = $actual (expected $expected)"
        FAIL=$((FAIL+1))
    fi
}

test_not() {
    local v=$1 expected=$2
    set_vi 5 "$v"
    sleep 0.3
    local actual
    actual=$(read_vs "$NOT_VS")
    if [ "$actual" = "$expected" ]; then
        echo "  ✓ NOT($v) = $actual"
        PASS=$((PASS+1))
    else
        echo "  ✗ NOT($v) = $actual (expected $expected)"
        FAIL=$((FAIL+1))
    fi
}

echo "═══════════════════════════════════════"
echo "  Cat A: Logic Gate Truth Tables"
echo "═══════════════════════════════════════"
echo ""

preflight

echo "── AND ──"
test_gate AND "$AND_VS" 1 2 0 0 0
test_gate AND "$AND_VS" 1 2 0 1 0
test_gate AND "$AND_VS" 1 2 1 0 0
test_gate AND "$AND_VS" 1 2 1 1 1

echo ""
echo "── OR ──"
test_gate OR "$OR_VS" 3 4 0 0 0
test_gate OR "$OR_VS" 3 4 0 1 1
test_gate OR "$OR_VS" 3 4 1 0 1
test_gate OR "$OR_VS" 3 4 1 1 1

echo ""
echo "── NOT ──"
test_not 0 1
test_not 1 0

echo ""
echo "── XOR ──"
test_gate XOR "$XOR_VS" 6 7 0 0 0
test_gate XOR "$XOR_VS" 6 7 0 1 1
test_gate XOR "$XOR_VS" 6 7 1 0 1
test_gate XOR "$XOR_VS" 6 7 1 1 0

echo ""
echo "═══════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
