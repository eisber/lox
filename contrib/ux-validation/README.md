# UX Validation Companion

Visual validation of Loxone Config changes using Windows UI automation.
Complements the `lox` CLI for headless config editing workflows.

## What it does

After deploying config changes via `lox config push`, these tools can verify
the changes took effect by inspecting the actual Loxone Config application:

- **Screenshot capture** — full desktop or specific regions
- **OCR** — read tree items, properties panel, connection status
- **Accessibility inspection** — FlaUI-based window/control tree
- **Automated clicks** — navigate the UI via SendInput or AutoHotkey

## Workflow

```bash
# 1. Edit config via lox CLI
lox config download --extract --save-as config.Loxone
lox config control move config.Loxone --type WeatherData --to-room "Zentral"
lox config push --file config.Loxone --reboot --force

# 2. Wait for reboot
sleep 60 && lox status

# 3. Validate via UX (requires Loxone Config running on Windows host)
python3 lox-validate.py --require-connected --tree-contains "Temperatur (Zentral"
```

## Requirements

- **Windows host** with Loxone Config running
- **WSL** or similar environment to run Python scripts
- **FlaUI-MCP bridge** (`flaui-bridge.py`) running on port 8585
- **Tesseract OCR** installed
- **AutoHotkey v2** (optional, for click automation)

## Files

| File | Purpose |
|------|---------|
| `loxone_ux.py` | Core validation: focus, screenshots, OCR, connection check |
| `winclick.py` | Mouse click automation (SendInput + AHK) |
| `flaui-bridge.py` | HTTP bridge to FlaUI Windows accessibility framework |
| `tree_ocr.py` | Specialized OCR for the Loxone Config tree panel |
| `loxvision.py` | Screenshot capture + region-based OCR |
| `loxone_properties.py` | OCR for the right-hand properties panel |
| `lox-validate.py` | CLI wrapper for validation checks |

## Note

These tools are Windows-specific and require a running Loxone Config instance.
They are provided as a companion to the cross-platform `lox` CLI, not as a
replacement. The `lox` CLI handles all config editing and deployment; these
tools add visual verification.
