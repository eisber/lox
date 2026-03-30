# Loxone /wsx Save Protocol

## Overview

The Loxone Miniserver uses a proprietary WebSocket endpoint (`/wsx`) for
config upload and management. This document describes the protocol.

## Connection Flow

1. Authenticate via HTTPS: `getPublicKey` → `getkey2` → `getjwt`
2. Compute `autht = HMAC-SHA256(hex_decode_ascii(key), jwt_token).upper()`
3. TLS connect to port 443
4. HTTP upgrade: `GET /wsx?autht={token}&user={user}`
5. Send hixie-76 text frame: `\x00dev/loxone/start\xff`
6. Send 64-byte binary handshake (see below)
7. Server responds with capabilities (0x01)

## Handshake Packet (64 bytes)

```
Bytes  0-15:  Nonce (timestamp + constants)
Bytes 16-31:  Header (version=1, features=0x20, magic=0xEFBEEDFE)
Bytes 32-63:  RC6-encrypted credentials
```

### Nonce Construction

The nonce embeds the Miniserver's uptime in milliseconds:
```
[0x29, 0x00, 0x00, T3, T0, 0xBE, 0x18, 0x84,
 0x6C, T1, 0xE1, 0x4A, 0xD6, 0x2C, T2, 0xAE]
```
Where T0-T3 are the uptime bytes (T0=low, T3=high).
Constants are from MSVC `rand()` with seed 1.

### Credential Payload

The 32-byte payload is `"user\0password\0locale\0"` zero-padded,
then encrypted with RC6 (16 rounds, single-word key derived from
the uptime timestamp).

## Binary Message Format

```
[cmd:1][flags:1][size:2 LE][seq:4 LE][reserved:4 LE][magic:4]
```
- Client magic: `0xEFBEEDFE`
- Server magic: `0xFEAFEDFE`

### Commands

| Cmd  | Name | Description |
|------|------|-------------|
| 0x01 | Capabilities | Server to client after handshake |
| 0x05 | PostSave | Upload complete, restart SPS |
| 0x14 | Keepalive | Bidirectional keepalive |
| 0x29 | Handshake | 64-byte client handshake |
| 0x3A | PreSave | Opens save window for fsput |
| 0x97 | JSONStatus | Server status (serial, device counts) |

### Save Flow

```
Client: 0x3A (pre-save)
Server: 0x3A ack
Client: HTTP POST /dev/fsput/lx/prog/sps_new.zip?autht={token}&user={user}
Server: HTTP 200
Client: 0x05 (post-save, triggers SPS restart)
```

## Authentication

### Token Computation (autht)

```
key_hex = getkey()  or  getjwt().key
key_ascii = bytes.fromhex(key_hex).decode('ascii')  # double-hex decode
autht = HMAC-SHA256(key_ascii.encode(), jwt_token.encode()).hexdigest().upper()
```

### Deployment Paths

| Method | Speed | Scope |
|--------|-------|-------|
| FTP + `sps/restart` | ~5s | Logic gates |
| FTP + `sys/reboot` | ~60s | All blocks |
| `/wsx` + fsput | ~2s | All blocks |

## WebSocket Framing

The `/wsx` endpoint uses hixie-76 text framing for text commands.
Binary messages are sent as raw bytes (no WebSocket framing).

## SPS Compilation (Key Finding)

The SPS is compiled **ON the Miniserver**, not in LoxoneConfig.exe.
There is NO separate bytecode file — the Miniserver reads the XML config
from `sps0.LoxCC` and compiles it internally during save/reboot.

**However**, XML-injected blocks MUST have the correct attributes to be
recognized by the Miniserver's SPS compiler:

### Required XML Attributes for Blocks

| Attribute | Description | Example |
|-----------|-------------|---------|
| `V` | Config version number | `"175"` |
| `Nio` | Number of I/O connectors | `"3"` |
| `WF` | Wire flags (bitmask) | `"147456"` |
| `Px`, `Py` | Position in UX canvas | `"4416"`, `"5952"` |

### Required for VirtualIn

| Attribute | Description | Example |
|-----------|-------------|---------|
| `IName` | Internal name (NOT VIName) | `"VI1"` |
| `EnVal` | Enable value setting | `"true"` |
| `MaxVal` | Maximum value | `"100"` |
| `<IoData>` | I/O data with room/category refs | Required |

### UUID Format
Loxone UUIDs use a specific format: `{8hex}-{4hex}-{4hex}-{16hex}`
Standard UUID4 format works but the suffix encodes object relationships.

### What Doesn't Work
- Blocks without `V` attribute → ignored by SPS compiler
- VirtualIn without `IoData` + `Cr`/`Pr` refs → not accessible via HTTP API
- Standard UUID4 without Loxone suffix encoding → wiring may not work

## XML Injection: Correct Tree Structure

Blocks MUST be placed in the correct tree container:

```
ControlList
  └─ Document
       ├─ LoxLIVE
       │    ├─ VirtualInCaption → VirtualIn elements HERE
       │    └─ VirtualOutCaption → VirtualOut/StateV elements HERE  
       └─ Program
            └─ Page → Logic/Math/Comparison blocks HERE
```

### VirtualIn Wiring Chain
VirtualIn does NOT have an AQ connector directly. The wiring uses
an InputRef converter element:

```
VirtualIn (Q output)
    → InputRef (AI input ← Q, AQ output)
        → Block input (In Input=InputRef.AQ)
```

The InputRef needs:
- `Type="InputRef"`
- `Ref="{VirtualIn UUID}"`
- `LinkRefType="71"` (VirtualIn reference type)
- `Analog="true"` (for analog VIs)
- Connectors: AI (input from VI.Q), I, AQ (output to block), Q
