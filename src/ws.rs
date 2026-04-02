//! Loxone WebSocket client — used for token auth key exchange

use anyhow::Result;
use rustls::{ClientConfig, crypto::ring};
use std::sync::Arc;
use tokio_tungstenite::Connector;
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::tungstenite::http::Request;

use rand::RngCore as _;

use crate::config::Config;

// ── TLS (accept all) ─────────────────────────────────────────────────────────

pub fn make_tls_config_pub() -> Arc<ClientConfig> {
    make_tls_config()
}

fn make_tls_config() -> Arc<ClientConfig> {
    let _ = ring::default_provider().install_default();
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut cfg = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    cfg.dangerous()
        .set_certificate_verifier(Arc::new(NoCertVerifier));
    cfg.enable_early_data = true;
    Arc::new(cfg)
}

#[derive(Debug)]
struct NoCertVerifier;
impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _: &rustls::pki_types::CertificateDer,
        _: &[rustls::pki_types::CertificateDer],
        _: &rustls::pki_types::ServerName,
        _: &[u8],
        _: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &rustls::pki_types::CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &rustls::pki_types::CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

// ── WS Client ────────────────────────────────────────────────────────────────

pub struct LoxWsClient {
    cfg: Config,
}

impl LoxWsClient {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    fn ws_url(&self) -> String {
        self.cfg
            .host
            .replace("https://", "wss://")
            .replace("http://", "ws://")
            .trim_end_matches('/')
            .to_string()
            + "/ws/rfc6455"
    }

    pub async fn connect_raw(
        &self,
    ) -> Result<(
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    )> {
        let url = self.ws_url();
        let tls_cfg = make_tls_config();
        let basic = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.cfg.user, self.cfg.pass),
        );
        let req = Request::builder()
            .uri(&url)
            .header("Authorization", format!("Basic {}", basic))
            .header(
                "Host",
                url.split("://")
                    .nth(1)
                    .unwrap_or("")
                    .split('/')
                    .next()
                    .unwrap_or(""),
            )
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_ws_key())
            .body(())?;
        connect_async_tls_with_config(req, None, false, Some(Connector::Rustls(tls_cfg)))
            .await
            .map_err(|e| anyhow::anyhow!("WS connect: {}", e))
    }
}

fn generate_ws_key() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

// ── Fast reload via /wsx ────────────────────────────────────────────────────

/// Connect to /wsx, perform handshake, and send 0x3A → 0x05 to trigger
/// a fast SPS reload (~4 seconds). Used after FTP-uploading sps_new.zip.
pub fn trigger_fast_reload(cfg: &Config) -> Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let host = cfg.host
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap_or(&cfg.host)
        .trim_end_matches('/');

    // Get uptime for handshake timestamp
    let client = crate::client::LoxClient::new(cfg.clone())?;
    let uptime_resp = client.get_text(
        "jdev/sps/io/20123f74-0222-3d2f-ffff234d69b98eb1/state",
    )?;
    let secs: f64 = serde_json::from_str::<serde_json::Value>(&uptime_resp)?["LL"]["value"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);
    let uptime_ms = (secs * 1000.0) as u32;

    // Compute autht for /wsx upgrade
    let gk2_resp = client.get_text(&format!("jdev/sys/getkey2/{}", cfg.user))?;
    let gk2: serde_json::Value = serde_json::from_str(&gk2_resp)?;
    let gk2_val = &gk2["LL"]["value"];
    let key_hex = gk2_val["key"].as_str().ok_or_else(|| anyhow::anyhow!("missing key"))?;
    let salt = gk2_val["salt"].as_str().ok_or_else(|| anyhow::anyhow!("missing salt"))?;

    let pw_hash = {
        use sha2::{Sha256, Digest};
        let h = Sha256::digest(format!("{}:{}", cfg.pass, salt).as_bytes());
        hex::encode_upper(h)
    };
    let sig = {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let key_bytes = hex::decode(key_hex)?;
        let mut mac = HmacSha256::new_from_slice(&key_bytes)?;
        mac.update(format!("{}:{}", cfg.user, pw_hash).as_bytes());
        hex::encode(mac.finalize().into_bytes())
    };
    let jwt_resp = client.get_text(&format!(
        "jdev/sys/getjwt/{}/{}/8/{}/lox-cli",
        sig, cfg.user, uuid::Uuid::new_v4()
    ))?;
    let jwt_val: serde_json::Value = serde_json::from_str(&jwt_resp)?;
    let jwt_token = jwt_val["LL"]["value"]["token"].as_str().ok_or_else(|| anyhow::anyhow!("no JWT token"))?;
    let jwt_key_hex = jwt_val["LL"]["value"]["key"].as_str().ok_or_else(|| anyhow::anyhow!("no JWT key"))?;
    let jwt_key_ascii = String::from_utf8(hex::decode(jwt_key_hex)?)?;

    let autht = {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut mac = HmacSha256::new_from_slice(jwt_key_ascii.as_bytes())?;
        mac.update(jwt_token.as_bytes());
        hex::encode_upper(mac.finalize().into_bytes())
    };

    // TLS connection
    let tls_cfg = make_tls_config();
    let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
        .unwrap_or_else(|_| rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap());
    let conn = rustls::ClientConnection::new(tls_cfg, server_name)?;
    let tcp = TcpStream::connect(format!("{}:443", host))?;
    tcp.set_nodelay(true)?;
    tcp.set_read_timeout(Some(Duration::from_secs(10)))?;

    let mut tls = rustls::StreamOwned::new(conn, tcp);

    // HTTP upgrade (use basic auth — simpler than JWT for /wsx)
    let upgrade = format!(
        "GET /wsx?autht={}&user={} HTTP/1.1\r\n\
         Host: {}:443\r\n\
         Upgrade: WebSocket\r\n\
         Connection: Upgrade\r\n\
         \r\n",
        autht, cfg.user, host,
    );
    tls.write_all(upgrade.as_bytes())?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(300));

    let mut resp = [0u8; 4096];
    let n = tls.read(&mut resp)?;
    let resp_str = String::from_utf8_lossy(&resp[..n]);
    if !resp_str.contains("101") {
        anyhow::bail!("/wsx upgrade failed: {}", resp_str.lines().next().unwrap_or(""));
    }

    // Send start frame
    tls.write_all(b"\x00dev/loxone/start\xff")?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(100));

    // Build and send handshake
    let handshake = crate::wsx::build_handshake_with_ts(
        &cfg.user, &cfg.pass, "DEU", uptime_ms,
    );
    tls.write_all(&handshake)?;
    tls.flush()?;
    std::thread::sleep(Duration::from_millis(2000));

    // Read capabilities response
    let mut cap = [0u8; 4096];
    let n = tls.read(&mut cap)?;
    if n == 0 || cap[0] != 0x01 {
        anyhow::bail!("/wsx handshake rejected (got 0x{:02X})", if n > 0 { cap[0] } else { 0 });
    }

    // Drain any pending messages
    // tcp_ref.set_read_timeout(Some(Duration::from_millis(500)))?;
    loop {
        match tls.read(&mut cap) {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => continue,
        }
    }
    // tcp_ref.set_read_timeout(Some(Duration::from_secs(10)))?;

    // 0x3A PreSave
    let magic = 0xEFBE_EDFEu32;
    let mut presave = [0u8; 16];
    presave[0] = 0x3A;
    presave[12..16].copy_from_slice(&magic.to_le_bytes());
    tls.write_all(&presave)?;
    tls.flush()?;

    let mut ack = [0u8; 256];
    let n = tls.read(&mut ack)?;
    if n == 0 || ack[0] != 0x3A {
        anyhow::bail!("0x3A PreSave not acknowledged (got 0x{:02X})", if n > 0 { ack[0] } else { 0 });
    }

    // Drain
    // tcp_ref.set_read_timeout(Some(Duration::from_millis(500)))?;
    loop {
        match tls.read(&mut ack) {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => continue,
        }
    }
    // tcp_ref.set_read_timeout(Some(Duration::from_secs(10)))?;

    // 0x05 PostSave → triggers fast SPS reload
    let mut postsave = [0u8; 16];
    postsave[0] = 0x05;
    postsave[12..16].copy_from_slice(&magic.to_le_bytes());
    tls.write_all(&postsave)?;
    tls.flush()?;

    // Read response (may be 0x05 ack or 0x34 status)
    let _ = tls.read(&mut ack);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ws_key_length() {
        let key = generate_ws_key();
        assert_eq!(key.len(), 24);
    }

    #[test]
    fn test_generate_ws_key_unique() {
        let k1 = generate_ws_key();
        let k2 = generate_ws_key();
        assert_ne!(k1, k2);
    }
}
