# Open Source Readiness Plan

This document tracks everything needed to make `lox` a high-quality, reusable open source project.
Items are ordered by impact and dependency.

---

## 1. Bug Fixes (Block Release)

These are correctness bugs that affect any user — fix before publishing.

### 1.1 `lox config set` clobbers aliases
- [ ] Load existing config first, patch only supplied fields, then save — so running `lox config set` to update a password doesn't wipe the user's alias list.

`ConfigCmd::Set` constructs a fresh `Config { aliases: Default::default() }`, wiping
whatever aliases the user had configured.

**Fix:**
```rust
// Instead of:
Config { host, user, pass, serial, aliases: Default::default() }.save()?;

// Do:
let mut cfg = Config::load().unwrap_or_default();
cfg.host = host; cfg.user = user; cfg.pass = pass;
if !serial.is_empty() { cfg.serial = serial; }
cfg.save()?;
```

### 1.2 Token auth acquired but never used
- [ ] Add `LoxClient::auth()` helper that applies token auth if a valid token is saved, falling back to Basic Auth.

`lox token fetch` acquires and persists a token via RSA+AES handshake, but every HTTP
request in `LoxClient` still uses Basic Auth. The token subsystem is inert.

### 1.3 Hardcoded energy meter UUIDs
- [ ] Auto-discover energy meters from structure by type (`Meter`, `EnergyManager`) instead of hardcoding UUIDs.
- [ ] If no energy meters exist in the structure, print a clear message rather than zeros.

`lox status --energy` hardcodes four UUIDs specific to one Miniserver installation.
No other user's Miniserver has these UUIDs.

Option A (preferred): auto-discover from structure by control type.
Option B: let users configure energy UUIDs in `config.yaml` under an `energy:` key.

### 1.4 `lox blind` and `lox mood` missing `--room`
- [ ] Add `#[arg(long)] room: Option<String>` to `Blind` in the clap enum and pass it to `resolve_with_room()`.
- [ ] Same for `Mood`.
- [ ] Fold the type-check into the resolution path so `find_control()` is no longer called separately.

Both commands use `find_control()` instead of `resolve_with_room()`. The README says
`--room` "works on all commands" — this is false for `blind` and `mood`.

### 1.5 `lox set` sends value unencoded in URL path
- [ ] Percent-encode the value segment before constructing the URL (at minimum encode `/` → `%2F`).

`format!("{}/jdev/sps/io/{}/{}", host, uuid, value)` — values containing `/`, `%`,
or spaces produce a malformed request.

### 1.6 Debug `eprintln!` left in `token.rs`
- [ ] Remove the debug line at `token.rs:84`: `eprintln!("[debug] normalized pem[..80]: ...")`.

### 1.7 Weak WebSocket nonce entropy (token auth)
- [ ] Replace `generate_ws_key()` with a proper 16-byte random nonce:

```rust
fn generate_ws_key() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}
```

---

## 2. Code Organisation

### 2.1 Break up `main.rs`
- [ ] Extract `LoxClient`, `Control`, and all resolution/fetch helpers into `src/client.rs`.
- [ ] Extract `Scene` and `SceneStep` into `src/scene.rs`.
- [ ] Leave `main.rs` as a thin dispatcher (`match cli.cmd { ... }`).

### 2.2 Remove dead code in `ws.rs`
- [ ] Delete `hmac_auth()` (never called; superseded by Basic Auth over the Upgrade header), or wire it up as an explicit fallback.

---

## 3. Testing

Currently zero tests. A published project needs a baseline.

### 3.1 Unit tests (no network required)
- [ ] `main.rs` / `client.rs` — `eval_op`, fuzzy name matching with and without room qualifier.
- [ ] `token.rs` — `TokenStore::is_valid()`.

### 3.2 Mock HTTP integration tests
- [ ] Add `wiremock` or `httpmock` dev-dependency.
- [ ] Test `list_controls` against a synthetic `LoxApp3.json` fixture.
- [ ] Test `send_cmd` round-trip.
- [ ] Test cache read/write lifecycle.
- [ ] Test `lox if` exit code correctness.

### 3.3 CI test gate
- [ ] Confirm no test requires a live Miniserver — all tests pass with `cargo test` in CI.

---

## 4. CI / CD

### 4.1 GitHub Actions — `ci.yml`
- [ ] Create `.github/workflows/ci.yml` running on every push and PR.
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo clippy -- -D warnings`.
- [ ] Run `cargo build --release`.
- [ ] Run `cargo test`.
- [ ] Matrix: `ubuntu-latest` + `macos-latest`.

### 4.2 Release workflow — `release.yml`
- [ ] Create `.github/workflows/release.yml` triggered on `v*` tags.
- [ ] Build `x86_64-unknown-linux-musl` → `lox-linux-x86_64`.
- [ ] Build `aarch64-unknown-linux-musl` → `lox-linux-aarch64`.
- [ ] Build `x86_64-apple-darwin` → `lox-macos-x86_64`.
- [ ] Build `aarch64-apple-darwin` → `lox-macos-aarch64`.
- [ ] Attach `sha256` checksums alongside each binary on the GitHub Release.
- [ ] Update README `## Install` with a one-liner `curl` install command.

### 4.3 Publish to crates.io
- [ ] Add `description`, `repository`, `homepage`, `keywords`, `categories`, `license` to `Cargo.toml`.
- [ ] Check crates.io for name conflicts (`lox`).
- [ ] Run `cargo publish` on first tagged release.

---

## 5. Distribution Packaging

### 5.1 Homebrew tap
- [ ] Create `homebrew-lox` tap repository.
- [ ] Write formula that downloads the release binary and verifies the sha256 checksum.

### 5.2 Nix flake
- [ ] Add `flake.nix` using `rustPlatform.buildRustPackage` or `naersk`.

### 5.3 AUR (Arch Linux)
- [ ] Publish `lox-bin` PKGBUILD pointing at the GitHub release binary.

---

## 6. Open Source Hygiene

### 6.1 `CONTRIBUTING.md`
- [ ] Document development setup (`cargo build`, `cargo test`, no live Miniserver needed).
- [ ] Document how to test manually (what's required: a Miniserver, or a mock).
- [ ] State PR process: one feature/fix per PR, tests expected.

### 6.2 `CHANGELOG.md`
- [ ] Create `CHANGELOG.md` starting at v0.1.0 with the current feature set, following [Keep a Changelog](https://keepachangelog.com).
- [ ] Wire the release workflow to update or validate the changelog on tag.

### 6.3 Issue templates
- [ ] Create `.github/ISSUE_TEMPLATE/bug_report.md` — fields: lox version, firmware version, command run, full output, `lox config show` output.
- [ ] Create `.github/ISSUE_TEMPLATE/feature_request.md` — fields: use case, what you tried, what you'd expect.

### 6.4 `LICENSE` file
- [ ] Confirm a `LICENSE` file exists with the MIT text (not just a mention in README). Add it if missing.

### 6.5 `--version` output
- [ ] Set a proper semver version in `Cargo.toml` (e.g. `0.1.0`).
- [ ] Consider embedding build target and git commit hash: `lox 0.1.0 (x86_64-unknown-linux-musl, git:abc1234)`.

---

## 7. Features Required for General Usability

These aren't bugs but the tool doesn't work well for new users without them.

### 7.1 CLI alias management
- [ ] Implement `lox alias add <name> <uuid>`.
- [ ] Implement `lox alias remove <name>`.
- [ ] Implement `lox alias list`.

### 7.2 `lox config set` should not require all fields
- [ ] Make `--host`, `--user`, `--pass` optional in `config set` so individual fields can be updated.
- [ ] Or add `lox config edit` that opens `~/.lox/config.yaml` in `$EDITOR`.

### 7.3 Better `lox status --energy`
- [ ] Implement auto-discovery of energy meters from structure (covered in §1.3).

---

## 8. Documentation

### 8.1 README improvements
- [ ] Add install options: pre-built binary, `cargo install`, Homebrew, Nix.
- [ ] Add badge row: CI status, crates.io version, license.
- [ ] Add "Tested against" section (firmware versions, Gen 1 / Gen 2 Miniserver).
- [ ] Link to `CONTRIBUTING.md` and the issue tracker.

### 8.2 `lox --help` completeness
- [ ] Add or improve description strings for `Rooms`, `Watch`, and `Log` in the clap derive macros.

---

## 9. Security / Credential Hygiene

### 9.1 Password visibility in process table
- [ ] Support `LOX_PASS` env var as an alternative to `--pass` on `config set`.
- [ ] Or support `--pass -` to read from stdin.

### 9.2 Config file permissions
- [ ] After writing `~/.lox/config.yaml`, set permissions to `0600`:

```rust
use std::os::unix::fs::PermissionsExt;
fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
```

---

## Priority Order

| # | Item | Why |
|---|------|-----|
| 1 | §1.3 Energy UUIDs | Breaks for every user except author |
| 2 | §1.1 Config clobbers aliases | Silently destroys user data |
| 3 | §1.2 Token auth not wired up | Feature appears to work but doesn't |
| 4 | §1.4 Blind/Mood missing --room | README says it works; it doesn't |
| 5 | §1.7 Debug eprintln | Unprofessional in release |
| 6 | §3 Tests + §4.1 CI | Required before public promotion |
| 7 | §4.2 Release binary builds | Required for non-Rust users |
| 8 | §6 OSS hygiene | CONTRIBUTING, CHANGELOG, issue templates |
| 10 | §1.5 URL encoding for set | Edge case but correctness |
| 11 | §7 General usability features | Quality of life |
| 12 | §2 Code organisation | Maintainability |
| 13 | §5 Packaging | Reach |
| 14 | §9 Credential hygiene | Good practice |
