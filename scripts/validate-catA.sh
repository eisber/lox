#!/bin/bash
# validate-catA.sh — Run Cat A logic gate truth tables
#
# Prerequisites: Run validate-setup.sh first to wire VirtualIns.
#
# Uses:
#   lox input set — to set VirtualIn values
#   lox ws-monitor — to read block output connector states

set -euo pipefail

WIRING_FILE="/tmp/validate-wiring.json"
PASS=0
FAIL=0
SKIP=0

if [ ! -f "$WIRING_FILE" ]; then
  echo "ERROR: $WIRING_FILE not found. Run validate-setup.sh first."
  exit 1
fi

# Helper: set a VirtualIn value via HTTP API
set_input() {
  local vi_name="$1" value="$2"
  lox input set "$vi_name" "$value" -q 2>/dev/null || true
}

# Helper: read output value via WebSocket monitor
read_output() {
  local block="$1"
  local q_uuid
  q_uuid=$(python3 -c "import json; d=json.load(open('$WIRING_FILE')); print(d['outputs']['$block'])" 2>/dev/null)
  if [ -z "$q_uuid" ]; then
    echo "ERR"
    return
  fi
  # Monitor for 2 seconds, take the last value
  local val
  val=$(timeout 3 lox ws-monitor "$q_uuid" --timeout 2 2>/dev/null | tail -1 | awk '{print $2}')
  if [ -z "$val" ]; then
    echo "0"  # default: no state change means 0
  else
    echo "$val"
  fi
}

# Helper: test a 2-input gate
test_gate() {
  local block="$1" i1="$2" i2="$3" expected="$4"
  set_input "VI_${block}_I1" "$i1"
  set_input "VI_${block}_I2" "$i2"
  sleep 0.5

  local actual
  actual=$(read_output "$block")
  # Compare as integers (truncate decimal)
  local actual_int expected_int
  actual_int=$(printf "%.0f" "$actual" 2>/dev/null || echo "$actual")
  expected_int=$(printf "%.0f" "$expected" 2>/dev/null || echo "$expected")

  if [ "$actual_int" = "$expected_int" ]; then
    echo "  ✓ ${block}($i1,$i2) = $actual_int"
    ((PASS++))
  else
    echo "  ✗ ${block}($i1,$i2) = $actual_int (expected $expected_int)"
    ((FAIL++))
  fi
}

# Helper: test NOT gate (1 input)
test_not() {
  local block="$1" input="$2" expected="$3"
  set_input "VI_${block}_I" "$input"
  sleep 0.5

  local actual
  actual=$(read_output "$block")
  local actual_int expected_int
  actual_int=$(printf "%.0f" "$actual" 2>/dev/null || echo "$actual")
  expected_int=$(printf "%.0f" "$expected" 2>/dev/null || echo "$expected")

  if [ "$actual_int" = "$expected_int" ]; then
    echo "  ✓ ${block}($input) = $actual_int"
    ((PASS++))
  else
    echo "  ✗ ${block}($input) = $actual_int (expected $expected_int)"
    ((FAIL++))
  fi
}

echo "═══════════════════════════════════════"
echo "  Cat A: Logic Gate Truth Tables"
echo "═══════════════════════════════════════"
echo ""

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
test_not CatA_Not 0 1
test_not CatA_Not 1 0

echo ""
echo "── XOR ──"
test_gate CatA_Xor 0 0 0
test_gate CatA_Xor 0 1 1
test_gate CatA_Xor 1 0 1
test_gate CatA_Xor 1 1 0

echo ""
echo "═══════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo "═══════════════════════════════════════"

exit $FAIL
