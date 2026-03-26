# Schema UX Validation Report

## Summary

| Type | Schema | XML Connectors | UX Validated | Issues Found |
|------|--------|---------------|--------------|--------------|
| AlarmClock | alarm-clock.json | 26 | ✅ Yes | Fixed: added missing Ca input |
| LightController2 | lighting-controller.json | 41 | ✅ Yes | Fixed: I/O misclassification, expanded ranges |
| Memory | analogue-memory.json | 3 | ⬜ Partial | XML has 1 input, schema has 2 |
| AutoPilot | automatic-rule.json | 2 | ⬜ Partial | Simple 1-in/1-out confirmed |

## Key Discovery: Three Naming Layers

Loxone uses three different naming conventions for the same connector:

| Layer | Example (AlarmClock) | Used In |
|-------|---------------------|---------|
| **XML key** | `Acknowledge` | Config XML files, API |
| **UX abbreviation** | `Ca` | Loxone Config UI, block labels |
| **KB description** | `Confirm alarm` | Documentation, KB articles |

Our schemas use UX abbreviations (matching what users see in Loxone Config).
The `crosswalk.json` maps between all three layers.

## AlarmClock Validation

### Issue Found: Missing Input
- **Ca** (Confirm alarm) was missing from the KB schema
- UX shows 7 inputs: Ca, S, DisA, Tg, Set, Rtd, Off
- KB article only listed 6 inputs (omitted Ca)
- **Fixed**: Added Ca to alarm-clock.json

### Connector Verification (from Anschluss anzeigen popup)
- Inputs (7): Ca, S, DisA, Tg, Set, Rtd, Off ✅
- Remanence: Aus ✅
- Parameters (3 visible): MaxA (120s), Pat (180s), Sd (300s) ✅
- Outputs (visible on block): Buzzer, Pa ✅

## LightController2 Validation

### Issue Found: Input/Output Misclassification
- KB article listed On, Alarm, Buzzer, Br, DisPc, P, Rtd, MBr as **inputs**
- UX shows these as **outputs** (right side of block)
- **Fixed**: Moved 8 connectors from inputs to outputs

### Issue Found: Abbreviated Ranges
- KB used "Lc1-4" and "T5/1-8" as single entries
- **Fixed**: Expanded to individual entries (Lc1, Lc2, Lc3, Lc4 and T5/1-T5/8)

### Issue Found: Missing Mood Input
- UX shows "Mood" input for selecting mood by ID
- Not in KB article
- **Fixed**: Added Mood input

### Connector Verification
- Inputs (18): Lc1-4, M+, M-, Mood, Off, T5/1-8, DisP, Mo ✅
- Outputs (visible): On, Alarm, Buzzer, Br, P ✅
- Parameters: Brt (30,000 lux), Moet (300,000s) ✅

## XML Key ↔ UX Abbreviation Crosswalk

See `docs/schemas/crosswalk.json` for the complete mapping.

### AlarmClock Example
| XML Key | UX Abbrev | Direction | Description |
|---------|-----------|-----------|-------------|
| Acknowledge | Ca | input | Confirm alarm |
| Snooze | S | input | Start snooze timer |
| Qa | Buzzer | output | Buzzer |
| QTa | A | output | Alarm active |
| AlarmTime | MaxA | parameter | Max alarm duration |

## Methodology

1. Opened function blocks on DG Schlafzimmer page in Loxone Config
2. Clicked green "+" buttons to open "Anschluss anzeigen" popups
3. Compared popup connector lists with KB schema definitions
4. Cross-referenced with XML config connector keys from templates.json
5. Built three-layer crosswalk (XML key ↔ UX abbreviation ↔ KB description)

## Files Updated

- `docs/schemas/alarm-clock.json` — Added Ca input, xml_key mappings
- `docs/schemas/lighting-controller.json` — Fixed I/O classification, expanded ranges, added Mood
- `docs/schemas/analogue-memory.json` — Added xml_key mappings
- `docs/schemas/automatic-rule.json` — Added xml_key mappings
- `docs/schemas/crosswalk.json` — NEW: XML↔UX connector mapping for 4 types
