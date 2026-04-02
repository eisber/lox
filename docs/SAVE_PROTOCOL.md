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

## Definitive Finding: SPS Pre-Compiled by LoxoneConfig.exe

**CONFIRMED**: The Miniserver does NOT compile new blocks from XML.

### Evidence
1. A CLONED working And block (identical XML, new UUID) → 404 after reboot
2. VirtualIn with correct tree placement, attributes, InputRef → 404
3. XML with all UX attributes (V, Nio, WF, IoData) → 404
4. No compiled SPS bytecode found on filesystem (/prog, /sys, /backup)

### Conclusion
LoxoneConfig.exe compiles the SPS program and uploads it as part of
the /wsx fsput save flow. The compiled SPS is stored in Miniserver
RAM or internal flash (not accessible via FTP). On reboot, the
Miniserver reloads the LAST compiled SPS — it does not recompile
from the XML config.

### Implication
Adding new block types REQUIRES one save from LoxoneConfig.exe UX.
After that initial save, FTP + reboot can modify wiring and settings
of EXISTING block types.

### Workaround
For automated testing, use the 14 block types already compiled on
the Miniserver (And, Or, Not, Xor, Add, Add4, Memory, LightController2,
PresenceDetector, etc.). New block types require UX intervention.

## Save Flow Clarification (Definitive)

### What 0x3A → 0x05 Does
- **Reloads** the existing compiled SPS program from memory/flash
- Does NOT recompile from XML config on disk
- Equivalent to `sps/restart` but via /wsx binary protocol

### What fsput Does
- Uploads new config AND triggers full SPS recompilation
- This is what creates NEW block types
- Auth: requires specific autht that we haven't cracked for fsput

### What LoxoneConfig UX Does
1. Compiles SPS locally (creates the internal SPS program)
2. Uploads compiled config via fsput (on the SAME /wsx TLS socket)
3. Sends 0x3A → 0x05 to activate

### Current State
- /wsx connect + handshake: ✅ WORKING (separate sends key)
- 0x3A → 0x05 save reload: ✅ WORKING  
- fsput upload: ❌ BLOCKED (auth returns 404 in save window)
- FTP upload + reboot: ✅ for existing blocks only

## Final Conclusion: SPS Compilation Architecture

### Confirmed Flow (from comprehensive Frida + FTP analysis)

**LoxoneConfig UX Save:**
1. UX compiles SPS program locally (in LoxoneConfig.exe)
2. UX connects to /wsx (upgrade + start + handshake)
3. UX sends `0x3A` pre-save → `0x05` post-save (32 bytes total)
4. Miniserver receives 0x05, reloads SPS from its cached compiled program
5. Miniserver saves a new versioned ZIP to /prog/sps_XXXX.zip

**No fsput upload occurs.** The UX sends zero config data during save.
The compiled SPS is transferred during an earlier phase (initial connection
or config sync) that we haven't captured yet.

### What We CAN Do From Linux
- FTP upload modified config → reboot → existing blocks reload ✅
- /wsx connect + handshake (RC6) → 0x3A/0x05 → SPS reload ✅
- Create new block types → ❌ requires UX compilation

### What Needs UX Intervention
Adding NEW block types (Comparator, Counter, etc.) requires one
LoxoneConfig UX save to compile the SPS. After that, the blocks
persist across reboots and can be modified (rewired, renamed) via FTP.

### Reliable /wsx Connection Pattern
Three SEPARATE SSL_write calls (not bundled):
1. HTTP upgrade request
2. `\x00dev/loxone/start\xff` (after 300ms delay)
3. RC6 handshake (after 100ms delay)
This consistently produces 0x01 Capabilities (binary mode).

## sendfile Endpoint (Key Discovery)

The Miniserver has a `jdev/sys/sendfile` endpoint for file transfer:
```
jdev/sys/sendfile/?json={"file":"%s","destination":"%s","address":"%s","crc":%d,"auth":"%s","sk":"%s"}
```

Parameters:
- `file`: source filename (e.g., "sps_new.zip")
- `destination`: target path (e.g., "/prog/")
- `address`: client IP address (Miniserver connects BACK to client to download)
- `crc`: CRC32 of file data
- `auth`: JWT token
- `sk`: session key (from getjwt response)

The transfer is **pull-based**: the Miniserver initiates an HTTP GET to the
client's address to download the file.

### fsput Authentication
LoxoneConfig uses `Authorization: Bearer <jwt>` (not `?autht=` query param).
Additional headers: `X-Checksum`, `Content-Type: application/octet-stream`.

### SPS Restart After Upload
After file transfer: `/dev/sps/restartclearu/{serial}` triggers SPS recompilation.
The `/jdev/sps/restartclear` endpoint also works (returns 200).

## sendfile Parameter Details (Ongoing Investigation)

The `sendfile` endpoint is confirmed working but parameters need refinement:

### What Works
- `jdev/sys/sendfile/?json={...}` — endpoint responds (not 404)
- Without `sk`: returns `{"value": "transfer failed", "Code": "500"}`  
- With wrong `sk` format: returns `{"value": "missing parameters", "Code": "400"}`

### Parameter Format (from binary strings)
```
jdev/sys/sendfile/?json={"file":"%s","destination":"%s","address":"%s","crc":%d,"auth":"%s","sk":"%s","https":true}
```

### Key Questions
1. What is `sk`? Likely the RSA-encrypted AES session key from `enc/` auth flow
2. What is `address`? Client IP where Miniserver pulls the file (pull-based)
3. Does the Miniserver use HTTP or HTTPS to pull? (`"https":true` flag exists)
4. What port does it connect to? (Binary doesn't show port in format string)

### Alternative: restartclear
`jdev/sps/restartclear` returns 200 and triggers SPS restart.
`/dev/sps/restartclearu/{serial}` triggers restart with UUID reference.
Neither recompiles from XML — they reload the cached compiled SPS.

## sendfile sk Parameter (Investigation Status)

Tested sk formats:
- No sk → "missing parameters" (400)
- JWT-only auth (no sk) → "transfer failed" (500)
- RSA-encrypted session key (684 chars) → "missing parameters" (400) — too long
- getjwt key ASCII (40 chars) → server closes connection — processes but fails
- getjwt key hex (80 chars) → CONNECTION RESET — crashes server!

The sk is likely 40-char ASCII (decoded getjwt key) but the transfer 
still fails because the HTTP server wasn't accessible or the pull 
protocol has additional requirements.

### Next Steps
1. Hook CHTTPClientQt::SendFile via Frida during UX save to capture exact sk
2. Or: serve the file on the Windows host and try with a correct sk
3. Monitor Windows HTTP server logs to see if Miniserver connects back

## Save Flow (MITM2 Capture — March 31, 2026)

Definitive connection-tagged MITM capture reveals the complete save architecture:

### Connections
- **C3**: Plain HTTP (port 80) — health checks (`dev/sys/check`)
- **C4**: HTTPS (port 443) — enc/ authentication: `getPublicKey` → `enc/getkey2?sk=RSA_ENC` → `enc/getjwt` → `getkey`
- **C5**: HTTPS (port 443) — `/wsx` WebSocket: handshake, keepalives, 0x71, 0x3A, 0x05
- **C6**: HTTPS (port 443) — file transfer: fsget, fsput

### Save Sequence
```
C4: enc/getkey2?sk={RSA_ENC(aes_key:aes_iv)}  → key, salt
C4: enc/getjwt/{sig}/admin/8/{uuid}/Loxone Config → JWT
C4: getkey → one-time key (purpose unknown, connection closes after)
C5: /wsx?autht={HMAC-SHA256(jwt_key, jwt)} → 101 upgrade
C5: dev/loxone/start + handshake → 0x01 capabilities
C6: GET /dev/fsget/lx/prog/sps.LoxCC  [Referrer: {TOKEN}]  → 200 (downloads current config)
C6: GET /jdev/sys/getkey → fresh key
C6: GET /dev/fsget/temp/custom?autht={HMAC(key,jwt)} → 200
C5: 0x71 SAVE-MANIFEST (196B: 16B header + 11 UUIDs)
C5: 0x3A PRE-SAVE → 0x3A ACK
C6: GET /jdev/sys/getkey → fresh key
C6: POST /dev/fsput/lx/prog/sps_new.zip?autht={HMAC(key,jwt)}&user=admin → 200 OK
C5: 0x05 POST-SAVE → 0x05 ACK (triggers SPS restart)
```

### Critical: Referrer Header
The **first request on C6** uses a custom `Referrer` HTTP header (NOT `?autht=`):
```
GET /dev/fsget/lx/prog/sps.LoxCC HTTP/1.1
Content-Type: application/octet-stream
Referrer: 74094882455171152761
```

This Referrer value:
- Is a ~20 digit decimal number (~67 bits, exceeds u64 range)
- Is CLIENT-GENERATED (not returned by any server API)
- Is SESSION-SPECIFIC (expired values return 401)
- Links the HTTP connection (C6) to the enc/ AES session (C4)
- **Without the correct Referrer, fsput returns 404 on ANY separate connection**
- The Referrer computation is in `CHTTPClientQt::SendFile` (binary offset 0x10FB6D0)
- The exact derivation from AES session material remains unknown

### What Works Without Referrer
- fsget with `?autht=` on any HTTPS connection (200 OK)
- fsget on the enc/ connection (same conn as `enc/getkey2?sk=`) with `?autht=` (200 OK)
- /wsx binary protocol: handshake, 0x3A, 0x05 (all work)

### What Requires Referrer
- fsput on ANY connection: returns 404 without prior Referrer-authenticated request on that connection

## Final Investigation Status (April 1, 2026)

### What Works Programmatically
- enc/ AES session establishment (getPublicKey → RSA encrypt → enc/getkey2?sk=) ✓
- enc/getjwt with NON-enc getkey2 key (sig from plain getkey2, sent via enc/) ✓
- /wsx handshake (RC6) + 0x3A save window + 0x05 post-save ✓
- fsget with autht on enc/ connection (200 OK) ✓
- fsget on separate plain HTTPS connection (200 OK) ✓

### What Fails
- fsput returns 404 on ALL connections, despite:
  - Correct autht (HMAC-SHA256 of fresh getkey + JWT)
  - Active enc/ AES session on same or separate connection
  - Open save window (0x3A ACK on /wsx)
  - Prior fsget on same connection
  - 3-connection UX-matched flow (C4=enc/, C5=/wsx, C6=fsget+fsput)
  - JWT with tokenRights=265 (same as UX)

### SslEncryptPacket Capture of Successful UX Save
Confirmed via Frida hook: the UX sends fsput through ncrypt.dll SslEncryptPacket (Schannel).
The request is identical to our programmatic attempt (same headers, same autht format).
The UX's connection was established with the full enc/ setup flow, and subsequent saves
reuse the same connection without re-authentication.

### Remaining Unknown
The Miniserver differentiates between the UX's HTTPS connection and our programmatic HTTPS
connection at a level below the HTTP protocol. Possible mechanisms:
- TLS session parameters (cipher suite, session ticket, client certificate)
- Internal server-side session state tied to TLS connection identity
- Undiscovered header or connection setup step

## Comprehensive Investigation (April 2, 2026)

### TLS Stack Elimination
Tested fsput with EVERY available TLS implementation:
- Python OpenSSL (TLS 1.2 and 1.3) → 404
- Python OpenSSL with ALPN h2,http/1.1 → 404
- Python OpenSSL with Schannel-like cipher suites → 404
- Windows .NET WebRequest (Schannel TLS 1.2) → 404
- Windows curl.exe (OpenSSL) → 404
- Full C# program with .NET connection pooling → 404

**Conclusion: TLS stack is NOT the differentiating factor.**

### fenc/ Discovery
The `fenc/` endpoint (vs `enc/`) returns AES-encrypted responses:
- `enc/` = encrypt request, plain response
- `fenc/` = encrypt request, encrypted response (AES-CBC, same session key)

The key from `enc/getkey2` is stored in a SEPARATE namespace and cannot be used for getjwt (returns 401).
The key from `fenc/getkey2` CAN be used for `fenc/getjwt` (returns 200 with valid JWT).
Both JWTs have identical tokenRights=265, but neither enables fsput.

### ALPN Investigation
The Miniserver does NOT support ALPN negotiation (returns None for all offered protocols).
This eliminates ALPN as a differentiating factor.

### Exact UX Sequence Replication
Replicated the EXACT UX startup sequence on ONE connection:
1. dev/sys/check → 200
2. dev/cfg/api → 200
3. getPublicKey → 200
4. enc/getkey2?sk= → 200 (AES session established)
5. getjwt (via non-enc key) → 200 (JWT obtained)
6. getkey → fsget sps.LoxCC → 200
7. getkey → fsget temp/custom → 200
8. /wsx 0x3A → ACK
9. getkey → POST fsput → **404**

**The fsput endpoint remains inaccessible despite matching every known aspect of the UX's behavior.**

### Remaining Hypothesis
The Miniserver's HTTP server may track connections using an internal identifier derived from
the specific enc/ AES session. The UX's `enc/getjwt` succeeds with the `enc/` key (not
`fenc/` key) because the UX's AES padding or command format produces a correctly decryptable
message. Our AES encryption produces valid enc/getkey2 responses but invalid enc/getjwt
commands. The 401 from enc/getjwt with enc/-derived keys suggests the decrypted command
is malformed (wrong padding, wrong salt format, or missing null terminator).

Cracking the enc/ key → getjwt flow would likely solve the fsput 404 issue, as the
enc/-authenticated JWT may carry an internal flag that enables file write operations.

## WORKING SOLUTION: FTP + restartclear (April 2, 2026)

### Bypassing fsput entirely!

The `fsput` endpoint requires an undiscovered connection-level authentication
mechanism. However, the same result can be achieved with a simpler approach:

### Flow
```
1. JWT auth: getkey2 → getjwt → compute autht
2. FTP upload: STOR /prog/sps_new.zip (the config ZIP with sps0.LoxCC)
3. HTTP GET: /dev/sps/restartclear?autht={HMAC-SHA256(getkey_ascii, jwt)}&user=admin → 200
4. Wait ~5 seconds for SPS restart
5. Verify: sps_new.zip is consumed (deleted), backup sps_VERS_TIMESTAMP.zip created
```

### Details
- FTP uses plain credentials (admin:password) on port 21
- `restartclear` uses JWT-based autht (same as fsget)
- The Miniserver loads config in priority order: Emergency.LoxCC → sps_new.zip → ...
- After loading, sps_new.zip is deleted and a backup is created
- SPS restarts automatically (~2-5 seconds)
- No /wsx connection or 0x3A save window needed!

### Verified
- FTP upload + restartclear: ✓ (config loaded, SPS running)
- sps_new.zip consumed: ✓ (deleted after loading)
- Backup created: ✓ (sps_0267_TIMESTAMP.zip)
- SPS status: Running 100/sec ✓

## FAST SAVE: FTP + /wsx 0x3A→0x05 (SOLVED!)

### 1-second save cycle — bypasses fsput entirely!

```
1. FTP STOR /prog/sps_new.zip         (~0.5s, upload config ZIP)
2. /wsx 0x3A pre-save → ACK           (~0.2s, open save window)
3. /wsx 0x05 post-save → SPS reload   (~0.3s, trigger fast reload)
Total: ~1.0s (SPS fully running in ~4s)
```

### Why this works
- The Miniserver loads config in priority order: Emergency.LoxCC → **sps_new.zip** → ...
- FTP places the file directly on the filesystem
- 0x3A opens the save window (same as UX's pre-save)
- 0x05 triggers a FAST SPS reload (NOT a full reboot)
- The Miniserver reads sps_new.zip, loads it, creates backup, deletes sps_new.zip

### Comparison
| Method | Time | Mechanism |
|--------|------|-----------|
| UX fsput + 0x05 | ~4s | HTTP POST + /wsx (requires undiscovered session auth) |
| **FTP + 0x3A/0x05** | **~1s + 4s** | FTP upload + /wsx (WORKING!) |
| FTP + restartclear | ~100s | FTP upload + full SPS restart (slow) |
| FTP + reboot | ~60s | FTP upload + full system reboot (slowest) |

### Verified (April 2, 2026)
- sps_new.zip consumed: ✓
- Backup created: ✓ (sps_0267_TIMESTAMP.zip)
- SPS Running 100/sec: ✓
- Save cycle time: 1.0s ✓
