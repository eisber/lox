# lox — Loxone Miniserver CLI

**Fast, scriptable command-line interface for Loxone Miniserver.**  
Single binary. No runtime. No cloud. Works in scripts, cron jobs, and AI agent pipelines.

---

## Why this exists

The Loxone app is great for humans tapping on phones. It's useless for everything else.

`lox` gives you a proper CLI so you can:

- **Script your home** — bash, Python, cron, whatever
- **Connect AI agents** — Claude, GPT, or any LLM tool can control your home via shell commands
- **Build automations** — rule engine with conditions, time windows, edge detection
- **Chain commands** — `lox if "Temperatur" gt 25 && lox blind "Südseite" pos 80`
- **Integrate with anything** — exit codes, JSON output, stdin/stdout

```bash
# Turn off all lights when leaving
lox off "Licht Wohnzimmer Zentral" && lox blind "Südseite" full-up

# AI agent can call these:
lox ls --type LightControllerV2 --json | jq '.[].name'
lox mood "Wohnzimmer" off
lox status --json | jq '.plc_running'

# Disambiguate sensors with the same name using --room or bracket syntax
lox get "Temperatur" --room "Schlafzimmer"
lox get "Temperatur [OG Kinderzimmer Bastian]"

# Quick energy overview
lox status --energy

# Conditionally close blinds
lox if "Temperatur Außen" gt 28 && lox blind "Beschattung Süd" pos 70
```

---

## For AI Agents

This CLI was designed for AI agent integration. Every command:

- Exits `0` on success, non-zero on error
- Has `--json` flag for structured output
- Uses fuzzy name matching — agents don't need UUIDs
- Supports `--room` flag and `[Room]` bracket syntax to resolve ambiguous names
- Returns readable errors with suggestions

**Example: give an LLM a shell tool**
```json
{
  "name": "lox",
  "description": "Control Loxone smart home. Returns JSON on --json flag.",
  "parameters": {
    "command": { "type": "string", "description": "e.g. 'on Wohnzimmer', 'blind Südseite pos 50', 'status --energy'" }
  }
}
```

The agent calls `lox <command>` as a shell tool and reads stdout. That's it.

An agent can discover your home (`lox ls --json`), read sensor values, control devices, and check conditions — all without any custom integration layer.

---

## Install

```bash
git clone https://github.com/discostu105/lox
cd lox
cargo build --release
cp target/release/lox ~/.local/bin/
```

**Requirements:** Rust 1.75+. No OpenSSL. No runtime dependencies.

## Setup

```bash
lox config set --host https://192.168.1.100 --user USER --pass PASS

# With serial for correct TLS hostname (avoids cert warnings)
lox config set --host https://192.168.1.100 --user USER --pass PASS --serial YOUR_SERIAL
```

Config: `~/.lox/config.yaml`

### Aliases

Add short names for frequently-used controls in `~/.lox/config.yaml`:

```yaml
host: https://192.168.1.100
user: admin
pass: secret
aliases:
  wz: "1d8af56e-036e-e9ad-ffffed57184a04d2"   # Lichtsteuerung Wohnzimmer
  kueche: "20236c09-0055-6e94-ffffed57184a04d2" # Licht Küche Insel
```

Then use short names directly: `lox on wz`, `lox off kueche`

---

## Commands

```bash
# ── System ────────────────────────────────────────────────────────
lox status                              # Miniserver health: firmware, PLC, memory
lox status --energy                     # + live energy panel (PV, grid, battery, consumption)
lox status --json

# ── Discovery ─────────────────────────────────────────────────────
lox ls                                  # All controls
lox ls --type Jalousie                  # Filter by type
lox ls --room "Wohnzimmer"              # Filter by room
lox ls --values                         # Include current values (slower)
lox ls --type LightControllerV2 --json  # JSON for agents/scripts
lox rooms                               # List all rooms
lox get "Lichtsteuerung Wohnzimmer"     # Full state of one control

# ── Resolving ambiguous names ─────────────────────────────────────
# When multiple controls share the same name (e.g. "Temperatur"):
lox get "Temperatur" --room "Schlafzimmer"          # --room flag
lox get "Temperatur [OG Schlafzimmer]"              # bracket syntax
lox off "Lichtsteuerung" --room "Büro Christoph"    # works on all commands

# ── Lights ────────────────────────────────────────────────────────
lox on  "Lichtsteuerung Wohnzimmer"
lox off "Lichtsteuerung Wohnzimmer"
lox mood "Lichtsteuerung Wohnzimmer" plus     # Next mood
lox mood "Lichtsteuerung Wohnzimmer" minus    # Previous mood
lox mood "Lichtsteuerung Wohnzimmer" off      # Turn off (mood 778)
lox mood "Lichtsteuerung Wohnzimmer" 704      # Set by numeric mood ID

# ── Blinds ────────────────────────────────────────────────────────
lox blind "Beschattung Süd" up
lox blind "Beschattung Süd" down
lox blind "Beschattung Süd" stop
lox blind "Beschattung Süd" pos 50      # Position 0-100%
lox blind "Beschattung Süd" full-up
lox blind "Beschattung Süd" full-down
lox blind "Beschattung Süd" shade       # Automatic shading

# ── Conditions & Logic ────────────────────────────────────────────
lox if "Temperatur Außen" gt 25         # Exit 0=true, 1=false
lox if "Schalter" eq 1 && lox on "Licht"

# ── Analog / Virtual Inputs ───────────────────────────────────────
lox set "Sollwert Heizung" 21.5
lox pulse "Taster"

# ── Scenes ────────────────────────────────────────────────────────
lox run abend
lox scene list
lox scene show abend
lox scene new abend

# ── Automation Daemon ─────────────────────────────────────────────
lox daemon                              # WebSocket (needs Monitor rights)
lox daemon --poll                       # HTTP polling fallback
lox automation list

# ── Cache ─────────────────────────────────────────────────────────
lox cache info
lox cache refresh
lox cache clear

# ── Token Auth ────────────────────────────────────────────────────
lox token fetch                         # Fetch & save token (valid 20 days)
lox token info
lox token clear

# ── Raw ───────────────────────────────────────────────────────────
lox send <uuid> <command>
lox watch "Temperatur Außen"
lox log                                 # Miniserver log (needs admin)
```

---

## Scenes

YAML files in `~/.lox/scenes/`:

```yaml
# ~/.lox/scenes/abend.yaml
name: Abend
steps:
  - control: "Lichtsteuerung Wohnzimmer"
    cmd: on
  - control: "Beschattung Südseite"
    cmd: "pos 70"
  - control: "LED Küche"
    cmd: off
    delay_ms: 500
```

---

## Automation Rules

`~/.lox/automations.yaml` — evaluated by the daemon:

```yaml
rules:
  - name: "Sonnenschutz bei Hitze"
    when:
      control: "Temperatur Außen"
      op: gt
      value: 28
    also:
      - control: "Windgeschwindigkeit"
        op: lt
        value: 10
    only_between: "10:00-18:00"
    then:
      - control: "Beschattung Süd"
        command: "pos 80"
```

**Operators:** `eq`, `ne`, `gt`, `lt`, `gte`, `lte`, `changes`  
**Conditions:** `also` (AND), `only_between` (time window)

---

## Performance

Structure cache at `~/.lox/cache/structure.json` (24h TTL):

| Operation | Cold | Warm |
|-----------|------|------|
| `lox on "Licht"` | ~1.2s | ~80ms |
| `lox ls` | ~1.2s | ~80ms |
| `lox ls --values` | ~1.2s + N×req | slower (one HTTP request per control) |
| `lox status` | ~120ms | ~120ms |

---

## Supported Control Types

| Type | Commands |
|------|----------|
| `LightControllerV2` | `on`, `off`, `mood plus/minus/off/<id>` |
| `Jalousie` / `CentralJalousie` | `up`, `down`, `stop`, `pos <0-100>`, `shade`, `full-up`, `full-down` |
| `Switch` | `on`, `off`, `pulse` |
| `Dimmer` | `on`, `off`, `set <0-100>` |
| `InfoOnlyAnalog` / `Meter` | `get` (read-only) |
| Any | `send <raw-command>` |

---

## Architecture

```
~/.lox/
  config.yaml          # Host, credentials, serial, aliases
  cache/
    structure.json     # LoxApp3.json (24h TTL, ~150KB)
  token.json           # Token auth (optional)
  scenes/*.yaml        # Your scenes
  automations.yaml     # Automation rules
```

Single static Rust binary ~4MB. TLS via rustls (no OpenSSL). Self-signed certs accepted.

---

## Systemd Service

```bash
lox service install    # Install automation daemon as systemd user service
lox service status
lox service logs
lox service uninstall
```

---

## Requirements

- Loxone Miniserver Gen 1/2, firmware 12.0+
- Local network access (or DynDNS)
- For `lox daemon` (WebSocket): Monitor rights enabled in Loxone Config
- For `lox log`: Admin user

---

## License

MIT
