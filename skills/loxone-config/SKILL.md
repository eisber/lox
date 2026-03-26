# Loxone Config Editing Skill

## Description
Edit Loxone Miniserver configuration headlessly using the `lox` CLI.
Supports room management, control wiring, MQTT setup, and deployment —
all without the Loxone Config desktop application.

## When to use
- User asks to configure, add, move, or wire Loxone smart home controls
- User wants to set up MQTT broker connection on a Miniserver
- User needs to deploy configuration changes to a live Miniserver
- User asks about Loxone room/control/device structure

## Prerequisites
- `lox` CLI installed and configured (`lox setup set --host ... --user ... --pass ...`)
- Network access to the Loxone Miniserver (HTTP + FTP on port 21)

## Workflow

### 1. Download current config
```bash
lox config download --extract --save-as config.Loxone
```

### 2. Inspect
```bash
lox config rooms config.Loxone              # rooms with item counts
lox config controls config.Loxone           # all controls
lox config controls config.Loxone -t WeatherData  # filter by type
lox config control describe config.Loxone "Kitchen Light"
lox config control wires config.Loxone "Kitchen Light"  # connections
lox config mqtt topics config.Loxone        # MQTT bindings
lox config validate config.Loxone           # integrity check
```

### 3. Edit
```bash
# Rooms
lox config room add config.Loxone "Zentral"
lox config room rename config.Loxone "Old" "New"

# Controls
lox config control move config.Loxone --type WeatherData --to-room "Zentral"
lox config control rename config.Loxone "Old Name" "New Name"
lox config add --type light --title "Kitchen Light" --room "Kitchen" config.Loxone

# Wiring
lox config control wire config.Loxone "Sensor.Q" "Light.On"
lox config control unwire config.Loxone "Light.AQ1"

# MQTT
lox config mqtt setup config.Loxone --broker 192.168.1.100 --user loxberry --password secret
lox config add --type mqtt-sub --title "Temp" --topic "weather/temp" --room "Zentral" config.Loxone

# Low-level XML
lox config xml set-property config.Loxone "gid:Mqtt" mqtt_broker_port "1883" --type 7
```

### 4. Deploy
```bash
lox config push --file config.Loxone --reboot --force
```

### 5. Verify
```bash
lox status                    # Miniserver online?
lox config download --extract # re-download and compare
```

## Element selector syntax
- `"Kitchen Light"` — match by Title (case-insensitive substring)
- `"Type:WeatherData"` — match all elements of a type
- `"uuid:abc-123"` — match by UUID
- `"gid:Mqtt"` — match by gid attribute

## Control type shortcuts for `lox config add`
| `--type` | Creates |
|----------|---------|
| `light` | LightController2 |
| `switch` | Switch |
| `presence` | PresenceDetector |
| `alarm-clock` | AlarmClock |
| `memory` | Memory |
| `timer` | SwitchingTimer |
| `mqtt-sub` | GenTSensor (MQTT subscription) |
| `mqtt-pub` | GenTActor (MQTT publish) |
| `calendar` | Calendar |
| `autopilot` | AutoPilot |

## Password handling
- Use `t="11"` (plaintext) for password fields — the Miniserver accepts these directly
- `t="15"` is encrypted by Miniserver firmware and cannot be created headlessly
- Example: `lox config mqtt setup ... --password "broker-pass"` stores as `t="11"`

## Key technical details
- Config files are `.Loxone` XML inside `.zip` archives compressed with LoxCC (custom LZ4)
- CRC32 in the LoxCC header (offset 12) must be correct for password fields to work
- The `lox config push` command handles recompression + CRC32 automatically
- Room assignments stored in `<IoData Pr="room-uuid"/>` child elements
- Connectors stored in `<Co K="connector-name" U="target-uuid"/>` child elements
