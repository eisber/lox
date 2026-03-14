//! OpenTelemetry exporter — pushes Loxone metrics via OTLP to any backend.
//!
//! Supports both continuous daemon mode (`lox otel serve`) and one-shot push
//! (`lox otel push`). Uses WebSocket streaming for real-time control state
//! updates, plus HTTP polling for system/network diagnostics.

use anyhow::{bail, Result};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::Resource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::client::LoxClient;
use crate::config::Config;
use crate::stream::{self, StateEvent, StateUuidInfo};

// ── Interval parsing ────────────────────────────────────────────────────────

/// Parse a human-friendly duration string (e.g. "30s", "5m", "1h").
pub fn parse_interval(s: &str) -> Result<Duration> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('s') {
        Ok(Duration::from_secs(n.parse()?))
    } else if let Some(n) = s.strip_suffix('m') {
        Ok(Duration::from_secs(n.parse::<u64>()? * 60))
    } else if let Some(n) = s.strip_suffix('h') {
        Ok(Duration::from_secs(n.parse::<u64>()? * 3600))
    } else {
        // Assume seconds if no suffix
        Ok(Duration::from_secs(s.parse()?))
    }
}

// ── OTLP exporter setup ────────────────────────────────────────────────────

/// Parse header strings like "Authorization=Bearer xxx" into (key, value) pairs.
fn parse_headers(headers: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for h in headers {
        if let Some((k, v)) = h.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        } else {
            bail!("Invalid header format '{}'. Expected 'Key=Value'.", h);
        }
    }
    Ok(map)
}

/// Build an OTLP metric exporter with the given endpoint and headers.
fn build_exporter(endpoint: &str, headers: &[String]) -> Result<MetricExporter> {
    let header_map = parse_headers(headers)?;
    let mut builder = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint);
    if !header_map.is_empty() {
        builder = builder.with_headers(header_map);
    }
    builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build OTLP exporter: {}", e))
}

/// Build the OTel resource with Miniserver metadata.
fn build_resource(cfg: &Config) -> Resource {
    let mut attrs = vec![
        KeyValue::new("service.name", "loxone-miniserver"),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        KeyValue::new("host.name", cfg.host.clone()),
    ];
    if !cfg.serial.is_empty() {
        attrs.push(KeyValue::new("device.id", cfg.serial.clone()));
    }
    Resource::builder().with_attributes(attrs).build()
}

// ── Metric recording ────────────────────────────────────────────────────────

/// Shared state for the latest values from WebSocket streaming.
type StateStore = Arc<Mutex<HashMap<String, (StateUuidInfo, f64)>>>;

/// Record system diagnostics (CPU, heap, tasks, etc.) as metrics.
fn record_system_metrics(lox: &LoxClient, meter: &opentelemetry::metrics::Meter) -> Result<()> {
    // Heap
    if let Ok(text) = lox.get_text("/dev/sys/heap") {
        if let Some(val) = extract_lox_value(&text) {
            let gauge = meter.f64_gauge("loxone.system.heap_bytes").build();
            gauge.record(val, &[]);
        }
    }

    // CPU (admin only — may fail)
    if let Ok(text) = lox.get_text("/jdev/sys/lastcpu") {
        if let Some(val) = extract_lox_value(&text) {
            let gauge = meter.f64_gauge("loxone.system.cpu_percent").build();
            gauge.record(val, &[]);
        }
    }

    // Tasks
    if let Ok(text) = lox.get_text("/jdev/sys/numtasks") {
        if let Some(val) = extract_lox_value(&text) {
            let gauge = meter.f64_gauge("loxone.system.tasks_count").build();
            gauge.record(val, &[]);
        }
    }

    // Context switches
    if let Ok(text) = lox.get_text("/jdev/sys/contextswitches") {
        if let Some(val) = extract_lox_value(&text) {
            let gauge = meter.f64_gauge("loxone.system.context_switches").build();
            gauge.record(val, &[]);
        }
    }

    Ok(())
}

/// Record CAN bus and LAN network metrics.
fn record_network_metrics(lox: &LoxClient, meter: &opentelemetry::metrics::Meter) -> Result<()> {
    let can_metrics = [
        ("/jdev/bus/packetssent", "loxone.can.packets_sent"),
        ("/jdev/bus/packetsreceived", "loxone.can.packets_received"),
        ("/jdev/bus/receiveerrors", "loxone.can.receive_errors"),
        ("/jdev/bus/frameerrors", "loxone.can.frame_errors"),
        ("/jdev/bus/overruns", "loxone.can.overruns"),
    ];
    for (path, name) in &can_metrics {
        if let Ok(text) = lox.get_text(path) {
            if let Some(val) = extract_lox_value(&text) {
                let gauge = meter.f64_gauge(*name).build();
                gauge.record(val, &[]);
            }
        }
    }

    let lan_metrics = [
        ("/jdev/lan/txp", "loxone.lan.tx_packets"),
        ("/jdev/lan/txe", "loxone.lan.tx_errors"),
        ("/jdev/lan/txc", "loxone.lan.tx_collisions"),
        ("/jdev/lan/rxp", "loxone.lan.rx_packets"),
        ("/jdev/lan/rxo", "loxone.lan.rx_overflow"),
        ("/jdev/lan/eof", "loxone.lan.eof_errors"),
    ];
    for (path, name) in &lan_metrics {
        if let Ok(text) = lox.get_text(path) {
            if let Some(val) = extract_lox_value(&text) {
                let gauge = meter.f64_gauge(*name).build();
                gauge.record(val, &[]);
            }
        }
    }

    Ok(())
}

/// Record the latest control state values as OTel gauge metrics.
fn record_control_metrics(
    store: &StateStore,
    meter: &opentelemetry::metrics::Meter,
    type_filter: Option<&str>,
    room_filter: Option<&str>,
) {
    let state = store.lock().unwrap();
    let gauge = meter.f64_gauge("loxone.control.value").build();

    for (uuid, (info, value)) in state.iter() {
        if let Some(tf) = type_filter {
            if !info
                .control_type
                .to_lowercase()
                .contains(&tf.to_lowercase())
            {
                continue;
            }
        }
        if let Some(rf) = room_filter {
            if !info
                .room
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&rf.to_lowercase())
            {
                continue;
            }
        }

        let attrs = [
            KeyValue::new("control.name", info.control_name.clone()),
            KeyValue::new("control.type", info.control_type.clone()),
            KeyValue::new("control.uuid", uuid.clone()),
            KeyValue::new("state.name", info.state_name.clone()),
            KeyValue::new("control.room", info.room.clone().unwrap_or_default()),
            KeyValue::new(
                "control.category",
                info.category.clone().unwrap_or_default(),
            ),
        ];
        gauge.record(*value, &attrs);
    }
}

/// Extract a numeric value from Loxone XML/JSON response.
/// Handles both JSON `{"LL":{"value":"42"}}` and plain XML `value="42"` formats.
fn extract_lox_value(text: &str) -> Option<f64> {
    // Try JSON first
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(val) = v.pointer("/LL/value").and_then(|v| v.as_str()) {
            return val.parse().ok();
        }
    }
    // Try XML attribute
    let key = "value=\"";
    if let Some(start) = text.find(key) {
        let rest = &text[start + key.len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].parse().ok();
        }
    }
    None
}

// ── Serve (continuous daemon) ───────────────────────────────────────────────

/// Run the OTel exporter in continuous mode:
/// - Connect to Miniserver WebSocket for real-time state changes
/// - Push metrics via OTLP at the configured interval
/// - Also fetch system/network diagnostics on each interval
pub fn serve(
    cfg: &Config,
    endpoint: &str,
    interval: Duration,
    type_filter: Option<&str>,
    room_filter: Option<&str>,
    headers: &[String],
    quiet: bool,
) -> Result<()> {
    let exporter = build_exporter(endpoint, headers)?;
    let reader = PeriodicReader::builder(exporter)
        .with_interval(interval)
        .build();
    let provider = SdkMeterProvider::builder()
        .with_resource(build_resource(cfg))
        .with_reader(reader)
        .build();

    // Load structure for UUID mapping
    let mut lox = LoxClient::new(cfg.clone());
    let structure = lox.get_structure()?.clone();
    let state_map = stream::build_state_uuid_map(&structure);

    // Shared state store updated by WebSocket, read by metric callbacks
    let store: StateStore = Arc::new(Mutex::new(HashMap::new()));
    let store_ws = Arc::clone(&store);

    if !quiet {
        eprintln!(
            "Pushing metrics to {} every {}s (Ctrl+C to stop)...",
            endpoint,
            interval.as_secs()
        );
    }

    let tf = type_filter.map(|s| s.to_string());
    let rf = room_filter.map(|s| s.to_string());

    // Spawn system metrics polling on a separate thread
    let cfg_sys = cfg.clone();
    let meter_sys = provider.meter("loxone");
    let tf_sys = tf.clone();
    let rf_sys = rf.clone();
    let store_sys = Arc::clone(&store);
    std::thread::spawn(move || {
        let lox = LoxClient::new(cfg_sys);
        loop {
            // Record system diagnostics
            let _ = record_system_metrics(&lox, &meter_sys);
            let _ = record_network_metrics(&lox, &meter_sys);

            // Record control state gauges from the shared store
            record_control_metrics(&store_sys, &meter_sys, tf_sys.as_deref(), rf_sys.as_deref());

            std::thread::sleep(interval);
        }
    });

    // WebSocket streaming on the main async runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(stream::stream_events(cfg, |events| {
        let mut state = store_ws.lock().unwrap();
        for event in &events {
            if let StateEvent::ValueState { uuid, value } = event {
                if let Some(info) = state_map.get(uuid) {
                    state.insert(uuid.clone(), (info.clone(), *value));
                }
            }
        }
        Ok(())
    }))?;

    // Graceful shutdown
    provider
        .shutdown()
        .map_err(|e| anyhow::anyhow!("OTel shutdown: {}", e))?;

    Ok(())
}

// ── Push (one-shot) ─────────────────────────────────────────────────────────

/// Push current state once and exit. Connects to WebSocket, collects the
/// initial state dump, fetches system metrics, pushes everything, and exits.
pub fn push(
    cfg: &Config,
    endpoint: &str,
    type_filter: Option<&str>,
    room_filter: Option<&str>,
    headers: &[String],
    quiet: bool,
) -> Result<()> {
    let exporter = build_exporter(endpoint, headers)?;
    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(1))
        .build();
    let provider = SdkMeterProvider::builder()
        .with_resource(build_resource(cfg))
        .with_reader(reader)
        .build();
    let meter = provider.meter("loxone");

    // Load structure for UUID mapping
    let mut lox = LoxClient::new(cfg.clone());
    let structure = lox.get_structure()?.clone();
    let state_map = stream::build_state_uuid_map(&structure);

    if !quiet {
        eprintln!("Collecting current state...");
    }

    // Collect initial state dump via WebSocket with a timeout.
    // After subscribing, the Miniserver sends a full dump of all states,
    // then switches to incremental updates. We collect for a few seconds
    // to capture the full dump, then disconnect.
    let store: StateStore = Arc::new(Mutex::new(HashMap::new()));
    let store_ws = Arc::clone(&store);

    let rt = tokio::runtime::Runtime::new()?;
    let collect_result = rt.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(10),
            stream::stream_events(cfg, |events| {
                let mut state = store_ws.lock().unwrap();
                let mut collected = false;
                for event in &events {
                    if let StateEvent::ValueState { uuid, value } = event {
                        if let Some(info) = state_map.get(uuid) {
                            state.insert(uuid.clone(), (info.clone(), *value));
                            collected = true;
                        }
                    }
                }
                // After we've collected the initial dump (first batch of value states),
                // check if we already have enough. The initial dump is complete when
                // we've received at least some states.
                if collected && !state.is_empty() {
                    // Signal to stop by returning an error (stream_events will propagate it)
                    // We use a custom error type to distinguish "done collecting" from real errors
                    return Err(anyhow::anyhow!("__done_collecting__"));
                }
                Ok(())
            }),
        )
        .await
    });

    // The "done collecting" error is expected — it means we got the initial dump.
    // WebSocket auth failures are non-fatal: we still push system metrics via HTTP.
    match collect_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) if e.to_string().contains("__done_collecting__") => {}
        Ok(Err(e)) => {
            if !quiet {
                eprintln!("Warning: WebSocket streaming failed: {}", e);
                eprintln!("Falling back to system metrics only (no control states).");
            }
        }
        Err(_) => {} // Timeout — collected what we could
    }

    // Record system metrics
    let _ = record_system_metrics(&lox, &meter);
    let _ = record_network_metrics(&lox, &meter);

    // Record control metrics from collected state
    record_control_metrics(&store, &meter, type_filter, room_filter);

    if !quiet {
        let count = store.lock().unwrap().len();
        eprintln!(
            "Pushing {} control states + system metrics to {}...",
            count, endpoint
        );
    }

    // Wait for the periodic reader to complete at least one export cycle,
    // then shut down (which does a final flush).
    std::thread::sleep(Duration::from_secs(2));

    // Shutdown may fail if no metrics were recorded (empty flush) — non-fatal
    let _ = provider.shutdown();

    if !quiet {
        eprintln!("Done.");
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval_seconds() {
        assert_eq!(parse_interval("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_interval("1s").unwrap(), Duration::from_secs(1));
    }

    #[test]
    fn test_parse_interval_minutes() {
        assert_eq!(parse_interval("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_interval("1m").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_parse_interval_hours() {
        assert_eq!(parse_interval("2h").unwrap(), Duration::from_secs(7200));
    }

    #[test]
    fn test_parse_interval_no_suffix() {
        assert_eq!(parse_interval("60").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_parse_interval_whitespace() {
        assert_eq!(parse_interval(" 30s ").unwrap(), Duration::from_secs(30));
    }

    #[test]
    fn test_parse_interval_invalid() {
        assert!(parse_interval("abc").is_err());
        assert!(parse_interval("").is_err());
    }

    #[test]
    fn test_parse_headers() {
        let headers = vec![
            "Authorization=Bearer token123".to_string(),
            "X-Custom=value".to_string(),
        ];
        let map = parse_headers(&headers).unwrap();
        assert_eq!(map.get("Authorization").unwrap(), "Bearer token123");
        assert_eq!(map.get("X-Custom").unwrap(), "value");
    }

    #[test]
    fn test_parse_headers_invalid() {
        let headers = vec!["no-equals-sign".to_string()];
        assert!(parse_headers(&headers).is_err());
    }

    #[test]
    fn test_extract_lox_value_json() {
        let text = r#"{"LL":{"value":"42.5","Code":"200"}}"#;
        assert_eq!(extract_lox_value(text), Some(42.5));
    }

    #[test]
    fn test_extract_lox_value_xml() {
        let text = r#"<LL value="21.5" />"#;
        assert_eq!(extract_lox_value(text), Some(21.5));
    }

    #[test]
    fn test_extract_lox_value_none() {
        assert_eq!(extract_lox_value("no value here"), None);
    }

    #[test]
    fn test_build_resource() {
        let cfg = Config {
            host: "https://192.168.1.100".to_string(),
            serial: "SERIAL123".to_string(),
            ..Default::default()
        };
        let resource = build_resource(&cfg);
        // Resource is opaque, but we can verify it was created without panic
        let _ = format!("{:?}", resource);
    }
}
