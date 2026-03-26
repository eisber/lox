#!/bin/bash
# validate-catA.sh — Run Cat A logic gate truth tables
#
# Prerequisites:
#   1. Cat A blocks (CatA_And, CatA_Or, CatA_Not, CatA_Xor) on Miniserver
#   2. VirtualIns (Eingang VI1-VI7) wired to block inputs
#   3. Config saved to Miniserver via Loxone Config UX (SPS compiled)
#
# Uses:
#   HTTP API /sps/io/ — to set VirtualIn values
#   HTTP API /sps/state/ — to read block output connector states
#
# Block output Q UUIDs (from XML config):
#   CatA_And.Q  = 206966b7-01f6-4cec-ffff6437a6164046
#   CatA_Or.Q   = 206966b7-01f6-4cf3-ffff6437a6164046
#   CatA_Not.Q  = 206966b7-01f6-4cf8-ffff6437a6164046
#   CatA_Xor.Q  = 206966b7-01f6-4cff-ffff6437a6164046

set -euo pipefail

HOST="${LOX_HOST:-http://192.168.68.72}"
CREDS="${LOX_CREDS:-admin:admin}"
PASS=0
FAIL=0
SETTLE_MS=300  # milliseconds to wait for SPS to settle

# Q output connector UUIDs
declare -A Q_UUID=(
  [CatA_And]="206966b7-01f6-4cec-ffff6437a6164046"
  [CatA_Or]="206966b7-01f6-4cf3-ffff6437a6164046"
  [CatA_Not]="206966b7-01f6-4cf8-ffff6437a6164046"
  [CatA_Xor]="206966b7-01f6-4cff-ffff6437a6164046"
)

# VirtualIn names (VI1-VI7 mapped to block inputs)
declare -A VI_MAP=(
  [CatA_And_I1]="Eingang VI1"
  [CatA_And_I2]="Eingang VI2"
  [CatA_Or_I1]="Eingang VI3"
  [CatA_Or_I2]="Eingang VI4"
  [CatA_Not_I]="Eingang VI5"
  [CatA_Xor_I1]="Eingang VI6"
  [CatA_Xor_I2]="Eingang VI7"
)

# URL-encode a string
urlencode() { python3 -c "import urllib.parse; print(urllib.parse.quote('$1'))"; }

# Set a VirtualIn value
set_vi() {
  local name="$1" value="$2"
  local encoded
  encoded=$(urlencode "$name")
  curl -sf "${HOST}/jdev/sps/io/${encoded}/${value}" \
    -u "$CREDS" > /dev/null 2>&1
}

# Read block output Q state
read_q() {
  local block="$1"
  local uuid="${Q_UUID[$block]}"
  local resp
  resp=$(curl -sf "${HOST}/jdev/sps/state/${uuid}" -u "$CREDS" 2>/dev/null)
  echo "$resp" | python3 -c "import json,sys; print(json.load(sys.stdin)['LL']['value'])" 2>/dev/null
}

# Pre-flight check: verify Miniserver is reachable and VIs work
preflight() {
  echo "Pre-flight checks..."
  local code
  code=$(curl -sf -o /dev/null -w "%{http_code}" "${HOST}/jdev/cfg/api" -u "$CREDS" 2>/dev/null)
  if [ "$code" != "200" ]; then
    echo "ERROR: Miniserver not reachable at ${HOST}"
    exit 1
  fi
  echo "  ✓ Miniserver reachable"

  # Check VIs are accessible
  for key in CatA_And_I1 CatA_Not_I; do
    local vi_name="${VI_MAP[$key]}"
    local encoded
    encoded=$(urlencode "$vi_name")
    code=$(curl -sf -o /dev/null -w "%{http_code}" \
      "${HOST}/jdev/sps/io/${encoded}/0" -u "$CREDS" 2>/dev/null)
    if [ "$code" != "200" ]; then
      echo "ERROR: ${vi_name} not accessible (HTTP ${code}). Did you save from UX?"
      exit 1
    fi
  done
  echo "  ✓ VirtualIns accessible"

  # Check if block outputs return something other than "5" (= not compiled)
  local q_val
  q_val=$(read_q CatA_And)
  if [ "$q_val" = "5" ]; then
    echo "  ⚠ CatA_And.Q returns '5' — wiring may not be compiled in SPS"
    echo "    → Save config to Miniserver from Loxone Config UX to compile"
    echo "    → See docs/WIRING_GUIDE.md"
    exit 1
  fi
  echo "  ✓ Block outputs responding (Q=${q_val})"
  echo ""
}

# Test a 2-input gate
test_gate() {
  local block="$1" i1_val="$2" i2_val="$3" expected="$4"
  set_vi "${VI_MAP[${block}_I1]}" "$i1_val"
  set_vi "${VI_MAP[${block}_I2]}" "$i2_val"
  sleep "0.${SETTLE_MS}"

  local actual
  actual=$(read_q "$block")
  local actual_int
  actual_int=$(printf "%.0f" "$actual" 2>/dev/null || echo "$actual")

  if [ "$actual_int" = "$expected" ]; then
    echo "  ✓ ${block}(${i1_val},${i2_val}) = ${actual_int}"
    ((PASS++))
  else
    echo "  ✗ ${block}(${i1_val},${i2_val}) = ${actual_int} (expected ${expected})"
    ((FAIL++))
  fi
}

# Test NOT gate (1 input)
test_not() {
  local input="$1" expected="$2"
  set_vi "${VI_MAP[CatA_Not_I]}" "$input"
  sleep "0.${SETTLE_MS}"

  local actual
  actual=$(read_q "CatA_Not")
  local actual_int
  actual_int=$(printf "%.0f" "$actual" 2>/dev/null || echo "$actual")

  if [ "$actual_int" = "$expected" ]; then
    echo "  ✓ NOT(${input}) = ${actual_int}"
    ((PASS++))
  else
    echo "  ✗ NOT(${input}) = ${actual_int} (expected ${expected})"
    ((FAIL++))
  fi
}

echo "═══════════════════════════════════════"
echo "  Cat A: Logic Gate Truth Tables"
echo "═══════════════════════════════════════"
echo ""

preflight

echo "── AND ──"
test_gate CatA_And 0 0 0
test_gate CatA_And 0 1 0
test_gate CatA_And 1 0 0
test_gate CatA_And 1 1 1

echo ""
echo "── OR ──"
test_gate CatA_Or 0 0 0
test_gate CatA_Or 0 1 1
test_gate CatA_Or 1 0 1
test_gate CatA_Or 1 1 1

echo ""
echo "── NOT ──"
test_not 0 1
test_not 1 0

echo ""
echo "── XOR ──"
test_gate CatA_Xor 0 0 0
test_gate CatA_Xor 0 1 1
test_gate CatA_Xor 1 0 1
test_gate CatA_Xor 1 1 0

echo ""
echo "═══════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════"

[ "$FAIL" -eq 0 ] && exit 0 || exit 1
