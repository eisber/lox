#!/bin/bash
# validate-catB.sh — Run Cat B math block tests
#
# Prerequisites: Run validate-setup.sh first to wire VirtualIns.

set -euo pipefail

WIRING_FILE="/tmp/validate-wiring.json"
PASS=0
FAIL=0

if [ ! -f "$WIRING_FILE" ]; then
  echo "ERROR: $WIRING_FILE not found. Run validate-setup.sh first."
  exit 1
fi

set_input() {
  local vi_name="$1" value="$2"
  lox input set "$vi_name" "$value" -q 2>/dev/null || true
}

read_output() {
  local block="$1"
  local q_uuid
  q_uuid=$(python3 -c "import json; d=json.load(open('$WIRING_FILE')); print(d['outputs']['$block'])" 2>/dev/null)
  if [ -z "$q_uuid" ]; then echo "ERR"; return; fi
  local val
  val=$(timeout 3 lox ws-monitor "$q_uuid" --timeout 2 2>/dev/null | tail -1 | awk '{print $2}')
  echo "${val:-0}"
}

# Test a 2-input math block with tolerance
test_math() {
  local block="$1" i1="$2" i2="$3" expected="$4" tolerance="${5:-0.01}"
  set_input "VI_${block}_I1" "$i1"
  set_input "VI_${block}_I2" "$i2"
  sleep 0.5

  local actual
  actual=$(read_output "$block")

  # Compare with tolerance using Python
  local result
  result=$(python3 -c "
a, e, t = float('$actual'), float('$expected'), float('$tolerance')
print('PASS' if abs(a - e) <= t else 'FAIL')
print(f'{a:.4f}')
" 2>/dev/null)
  local verdict=$(echo "$result" | head -1)
  local actual_fmt=$(echo "$result" | tail -1)

  if [ "$verdict" = "PASS" ]; then
    echo "  ✓ ${block}($i1, $i2) = $actual_fmt"
    ((PASS++))
  else
    echo "  ✗ ${block}($i1, $i2) = $actual_fmt (expected $expected)"
    ((FAIL++))
  fi
}

# Test a 1-input math block
test_math1() {
  local block="$1" input="$2" expected="$3" tolerance="${4:-0.01}"
  set_input "VI_${block}_I" "$input"
  sleep 0.5

  local actual
  actual=$(read_output "$block")
  local result
  result=$(python3 -c "
a, e, t = float('$actual'), float('$expected'), float('$tolerance')
print('PASS' if abs(a - e) <= t else 'FAIL')
print(f'{a:.4f}')
" 2>/dev/null)
  local verdict=$(echo "$result" | head -1)
  local actual_fmt=$(echo "$result" | tail -1)

  if [ "$verdict" = "PASS" ]; then
    echo "  ✓ ${block}($input) = $actual_fmt"
    ((PASS++))
  else
    echo "  ✗ ${block}($input) = $actual_fmt (expected $expected)"
    ((FAIL++))
  fi
}

echo "═══════════════════════════════════════"
echo "  Cat B: Math Block Tests"
echo "═══════════════════════════════════════"
echo ""

echo "── Add2 ──"
test_math CatB_Add2 3 5 8
test_math CatB_Add2 -1 1 0
test_math CatB_Add2 0 0 0

echo ""
echo "── Subtract ──"
test_math CatB_Subtract 10 3 7
test_math CatB_Subtract 0 5 -5
test_math CatB_Subtract 3 3 0

echo ""
echo "── Multiply ──"
test_math CatB_Multiply 4 5 20
test_math CatB_Multiply 0 99 0
test_math CatB_Multiply -3 4 -12

echo ""
echo "── Divide ──"
test_math CatB_Divide 10 2 5
test_math CatB_Divide 7 3 2.333 0.01
test_math CatB_Divide 0 5 0

echo ""
echo "── Modulo ──"
test_math CatB_Modulo 10 3 1
test_math CatB_Modulo 9 3 0
test_math CatB_Modulo 7 4 3

echo ""
echo "── Integer (truncate) ──"
test_math1 CatB_Integer 3.7 3
test_math1 CatB_Integer -2.3 -2
test_math1 CatB_Integer 0.9 0

echo ""
echo "═══════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════"

exit $FAIL
