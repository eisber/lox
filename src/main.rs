use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    host: String,
    user: String,
    pass: String,
    /// Serial number for TLS (e.g. 504f94a26236). Auto-detected if empty.
    #[serde(default)]
    serial: String,
}

impl Config {
    fn dir() -> PathBuf { home_dir().unwrap_or_default().join(".lox") }
    fn path() -> PathBuf { Self::dir().join("config.yaml") }

    fn load() -> Result<Self> {
        let path = Self::path();
        let content = fs::read_to_string(&path)
            .with_context(|| "Config not found. Run: lox config set --host ... --user ... --pass ...")?;
        Ok(serde_yaml::from_str(&content)?)
    }

    fn save(&self) -> Result<()> {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, serde_yaml::to_string(self)?)?;
        println!("✓  Config saved to {:?}", path);
        Ok(())
    }

    /// Build a TLS-valid host URL using dyndns hostname (avoids cert mismatch)
    fn tls_host(&self) -> String {
        if !self.serial.is_empty() {
            // Extract IP from host, build dyndns URL
            let ip_part = self.host
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .replace('.', "-");
            return format!("https://{}.{}.dyndns.loxonecloud.com", ip_part, self.serial.to_lowercase());
        }
        self.host.clone()
    }
}

// ── Scene ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct SceneStep {
    control: String,
    cmd: String,
    #[serde(default)]
    delay_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Scene {
    name: Option<String>,
    description: Option<String>,
    steps: Vec<SceneStep>,
}

impl Scene {
    fn scenes_dir() -> PathBuf { Config::dir().join("scenes") }

    fn load(name: &str) -> Result<Self> {
        let path = Self::scenes_dir().join(format!("{}.yaml", name));
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Scene '{}' not found at {:?}", name, path))?;
        Ok(serde_yaml::from_str(&content)?)
    }

    fn list() -> Result<Vec<String>> {
        let dir = Self::scenes_dir();
        if !dir.exists() { return Ok(vec![]); }
        let mut names = vec![];
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }
}

// ── Loxone API ────────────────────────────────────────────────────────────────

struct LoxClient {
    cfg: Config,
    client: Client,
    structure: Option<Value>,
}

#[derive(Debug, Clone)]
struct Control {
    name: String,
    uuid: String,
    typ: String,
    room: Option<String>,
    states: HashMap<String, Value>,
}

impl LoxClient {
    fn new(cfg: Config) -> Self {
        Self {
            cfg,
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(10))
                .build().unwrap(),
            structure: None,
        }
    }

    fn get(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.cfg.host, path.trim_start_matches('/'));
        let resp = self.client
            .get(&url)
            .basic_auth(&self.cfg.user, Some(&self.cfg.pass))
            .send()?.text()?;
        Ok(resp)
    }

    fn get_json(&self, path: &str) -> Result<Value> {
        let url = format!("{}/{}", self.cfg.host, path.trim_start_matches('/'));
        Ok(self.client
            .get(&url)
            .basic_auth(&self.cfg.user, Some(&self.cfg.pass))
            .send()?.json::<Value>()?)
    }

    fn get_structure(&mut self) -> Result<&Value> {
        if self.structure.is_none() {
            self.structure = Some(self.get_json("/data/LoxApp3.json")?);
        }
        Ok(self.structure.as_ref().unwrap())
    }

    fn send_cmd(&self, uuid: &str, cmd: &str) -> Result<Value> {
        self.get_json(&format!("/jdev/sps/io/{}/{}", uuid, cmd))
    }

    fn get_all(&self, uuid: &str) -> Result<String> {
        self.get(&format!("/dev/sps/io/{}/all", uuid))
    }

    fn list_controls(&mut self, type_filter: Option<&str>, room_filter: Option<&str>) -> Result<Vec<Control>> {
        let structure = self.get_structure()?;

        let mut rooms: HashMap<String, String> = HashMap::new();
        if let Some(map) = structure.get("rooms").and_then(|r| r.as_object()) {
            for (uuid, room) in map {
                if let Some(name) = room.get("name").and_then(|n| n.as_str()) {
                    rooms.insert(uuid.clone(), name.to_string());
                }
            }
        }

        let mut controls = Vec::new();
        if let Some(ctrl_map) = structure.get("controls").and_then(|c| c.as_object()) {
            for (uuid, ctrl) in ctrl_map {
                let name = ctrl.get("name").and_then(|n| n.as_str()).unwrap_or("?").to_string();
                let typ = ctrl.get("type").and_then(|t| t.as_str()).unwrap_or("?").to_string();
                let room_uuid = ctrl.get("room").and_then(|r| r.as_str()).unwrap_or("").to_string();
                let room = rooms.get(&room_uuid).cloned();

                let mut states: HashMap<String, Value> = HashMap::new();
                if let Some(s) = ctrl.get("states").and_then(|s| s.as_object()) {
                    for (k, v) in s { states.insert(k.clone(), v.clone()); }
                }

                if let Some(tf) = type_filter {
                    if !typ.to_lowercase().contains(&tf.to_lowercase()) { continue; }
                }
                if let Some(rf) = room_filter {
                    let room_name = room.as_deref().unwrap_or("");
                    if !room_name.to_lowercase().contains(&rf.to_lowercase()) { continue; }
                }

                controls.push(Control { name, uuid: uuid.clone(), typ, room, states });
            }
        }
        controls.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(controls)
    }

    fn resolve(&mut self, name_or_uuid: &str) -> Result<String> {
        if looks_like_uuid(name_or_uuid) { return Ok(name_or_uuid.to_string()); }
        let controls = self.list_controls(None, None)?;
        let matches: Vec<&Control> = controls.iter()
            .filter(|c| c.name.to_lowercase().contains(&name_or_uuid.to_lowercase()))
            .collect();
        match matches.len() {
            0 => bail!("No control found matching '{}'", name_or_uuid),
            1 => Ok(matches[0].uuid.clone()),
            _ => {
                eprintln!("Multiple matches for '{}', be more specific:", name_or_uuid);
                for c in &matches {
                    eprintln!("  {:40} [{}]  {}", c.name,
                        c.room.as_deref().unwrap_or("-"), c.uuid);
                }
                bail!("Ambiguous name")
            }
        }
    }

    fn find_control(&mut self, name_or_uuid: &str) -> Result<Control> {
        let controls = self.list_controls(None, None)?;
        if looks_like_uuid(name_or_uuid) {
            return controls.into_iter().find(|c| c.uuid == name_or_uuid)
                .context("UUID not found");
        }
        let matches: Vec<Control> = controls.into_iter()
            .filter(|c| c.name.to_lowercase().contains(&name_or_uuid.to_lowercase()))
            .collect();
        match matches.len() {
            0 => bail!("No control found matching '{}'", name_or_uuid),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => {
                eprintln!("Multiple matches for '{}':", name_or_uuid);
                for c in &matches {
                    eprintln!("  {:40} [{}]  {}", c.name,
                        c.room.as_deref().unwrap_or("-"), c.uuid);
                }
                bail!("Ambiguous name")
            }
        }
    }
}

fn looks_like_uuid(s: &str) -> bool {
    s.contains('-') && s.len() > 20
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn xml_attr<'a>(xml: &'a str, attr: &str) -> Option<&'a str> {
    let key = format!("{}=\"", attr);
    let start = xml.find(&key)? + key.len();
    let end = xml[start..].find('"')? + start;
    Some(&xml[start..end])
}

fn print_cmd_response(resp: &Value, json: bool, name: &str, cmd: &str) {
    if json {
        println!("{}", serde_json::to_string_pretty(resp).unwrap());
    } else {
        let val = resp.pointer("/LL/value").and_then(|v| v.as_str()).unwrap_or("?");
        let code = resp.pointer("/LL/Code").and_then(|v| v.as_str()).unwrap_or("?");
        let icon = if code == "200" { "✓" } else { "✗" };
        println!("{icon}  {name} → {cmd} = {val}");
    }
}

fn pct_bar(v: f64, max: f64) -> String {
    let pct = (v / max * 20.0) as usize;
    let pct = pct.min(20);
    format!("[{}{}] {:.0}%", "█".repeat(pct), "░".repeat(20 - pct), v / max * 100.0)
}

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "lox", about = "Loxone Miniserver CLI", version)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Configure connection
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Miniserver health & status
    Status,
    /// List controls
    Ls {
        #[arg(long)] r#type: Option<String>,
        #[arg(long)] room: Option<String>,
    },
    /// List rooms
    Rooms,
    /// Get full state of a control
    Get { name_or_uuid: String },
    /// Send a raw command
    Send { name_or_uuid: String, command: String },
    /// Turn on
    On { name_or_uuid: String },
    /// Turn off
    Off { name_or_uuid: String },
    /// Pulse (momentary trigger)
    Pulse { name_or_uuid: String },
    /// Control a blind/jalousie: up | down | open | close | shade | stop
    Blind {
        name_or_uuid: String,
        /// up | down | open | close | shade | stop | full-up | full-down
        action: String,
    },
    /// Watch state changes (polling)
    Watch {
        name_or_uuid: String,
        #[arg(long, default_value = "2")]
        interval: u64,
    },
    /// Check state — exits 0 if condition matches, 1 if not
    If {
        name_or_uuid: String,
        /// eq | ne | gt | lt | ge | le | contains
        op: String,
        value: String,
    },
    /// Run a scene
    Run { scene: String },
    /// Manage scenes
    Scene {
        #[command(subcommand)]
        action: SceneCmd,
    },
    /// Fetch Miniserver system log
    Log {
        #[arg(long, default_value = "50")]
        lines: usize,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    Set {
        #[arg(long)] host: String,
        #[arg(long)] user: String,
        #[arg(long)] pass: String,
        #[arg(long, default_value = "")] serial: String,
    },
    Show,
}

#[derive(Subcommand)]
enum SceneCmd {
    List,
    Show { name: String },
    New { name: String },
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        // ── Config ──────────────────────────────────────────────────────────
        Cmd::Config { action } => match action {
            ConfigCmd::Set { host, user, pass, serial } => {
                Config { host, user, pass, serial }.save()?;
            }
            ConfigCmd::Show => {
                let cfg = Config::load()?;
                println!("host:   {}", cfg.host);
                println!("user:   {}", cfg.user);
                println!("pass:   {}", "*".repeat(cfg.pass.len()));
                if !cfg.serial.is_empty() {
                    println!("serial: {}", cfg.serial);
                    println!("tls:    {}", cfg.tls_host());
                }
            }
        },

        // ── Status ──────────────────────────────────────────────────────────
        Cmd::Status => {
            let lox = LoxClient::new(Config::load()?);

            let version  = lox.get("/dev/cfg/version")?;
            let heap     = lox.get("/dev/sys/heap")?;
            let sps      = lox.get("/dev/sps/state")?;
            let check    = lox.get("/dev/sys/check")?;
            let status   = lox.get("/data/status")?;

            let ver = xml_attr(&version, "value").unwrap_or("?");
            let heap_val = xml_attr(&heap, "value").unwrap_or("?");
            let sps_code = xml_attr(&sps, "value").unwrap_or("?");
            let conn = xml_attr(&check, "value").unwrap_or("?");

            let sps_label = match sps_code {
                "5" => "✓ Running",
                "3" => "Started",
                "7" => "⚠ Error",
                "1" => "Booting",
                "8" => "Updating",
                n   => n,
            };

            // Parse heap: "381236/1016404kB"
            let heap_display = if let Some((used, total)) = heap_val.split_once('/') {
                let total_str = total.trim_end_matches("kB");
                let used_f: f64 = used.parse().unwrap_or(0.0);
                let total_f: f64 = total_str.parse().unwrap_or(1.0);
                format!("{} / {}  {}", used_val_fmt(used_f), used_val_fmt(total_f),
                    pct_bar(used_f, total_f))
            } else { heap_val.to_string() };

            // Parse Miniserver name from status XML
            let ms_name = xml_attr(&status, "Name").unwrap_or("Loxone Miniserver");
            let ms_ip   = xml_attr(&status, "IP").unwrap_or("?");
            let offline = xml_attr(&status, "Offline").unwrap_or("false");
            let online  = if offline == "false" { "✓ Online" } else { "✗ Offline" };

            if cli.json {
                println!("{}", serde_json::json!({
                    "name": ms_name, "ip": ms_ip, "version": ver,
                    "plc": sps_label, "heap": heap_val,
                    "connections": conn, "online": offline == "false"
                }));
            } else {
                println!("┌─ Loxone Miniserver ─────────────────────────────────");
                println!("│  Name:        {}", ms_name);
                println!("│  IP:          {}  ({})", ms_ip, online);
                println!("│  Firmware:    {}", ver);
                println!("│  PLC:         {}", sps_label);
                println!("│  Memory:      {}", heap_display);
                println!("│  Connections: {}", conn);
                println!("└─────────────────────────────────────────────────────");
            }
        },

        // ── Ls ──────────────────────────────────────────────────────────────
        Cmd::Ls { r#type, room } => {
            let mut lox = LoxClient::new(Config::load()?);
            let controls = lox.list_controls(r#type.as_deref(), room.as_deref())?;
            if cli.json {
                let out: Vec<Value> = controls.iter().map(|c| serde_json::json!({
                    "name": c.name, "uuid": c.uuid, "type": c.typ, "room": c.room
                })).collect();
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("{:<40} {:<24} {:<22} {}", "NAME", "ROOM", "TYPE", "UUID");
                println!("{}", "─".repeat(120));
                for c in &controls {
                    println!("{:<40} {:<24} {:<22} {}",
                        c.name, c.room.as_deref().unwrap_or("─"), c.typ, c.uuid);
                }
                println!("\n{} controls", controls.len());
            }
        },

        // ── Rooms ───────────────────────────────────────────────────────────
        Cmd::Rooms => {
            let mut lox = LoxClient::new(Config::load()?);
            let structure = lox.get_structure()?;
            if let Some(rooms) = structure.get("rooms").and_then(|r| r.as_object()) {
                let mut names: Vec<&str> = rooms.values()
                    .filter_map(|r| r.get("name").and_then(|n| n.as_str()))
                    .collect();
                names.sort();
                for name in names { println!("{}", name); }
            }
        },

        // ── Get ─────────────────────────────────────────────────────────────
        Cmd::Get { name_or_uuid } => {
            let mut lox = LoxClient::new(Config::load()?);
            let ctrl = lox.find_control(&name_or_uuid)?;
            let xml = lox.get_all(&ctrl.uuid)?;

            if cli.json {
                // Parse XML attributes into a map
                let mut result = serde_json::json!({
                    "name": ctrl.name,
                    "uuid": ctrl.uuid,
                    "type": ctrl.typ,
                    "room": ctrl.room,
                    "value": xml_attr(&xml, "value"),
                });
                // Extract known state attrs
                for attr in &["StateUp","StateDown","StatePos","StateShade","StateAutoShade","StateSafety"] {
                    if let Some(v) = xml_attr(&xml, attr) {
                        result[attr] = Value::String(v.to_string());
                    }
                }
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let val = xml_attr(&xml, "value").unwrap_or("?");
                let code = xml_attr(&xml, "Code").unwrap_or("?");
                println!("Control:  {} ({})", ctrl.name, ctrl.uuid);
                println!("Type:     {}   Room: {}", ctrl.typ,
                    ctrl.room.as_deref().unwrap_or("─"));
                println!("Value:    {}  [Code {}]", val, code);

                // Jalousie extra states
                for attr in &["StateUp","StateDown","StatePos","StateShade","StateAutoShade"] {
                    if let Some(v) = xml_attr(&xml, attr) {
                        println!("{:10}{}", attr.trim_start_matches("State"), v);
                    }
                }

                // Output list
                let mut out_num = 1;
                loop {
                    let name_attr = format!("n{}", out_num);
                    let val_attr  = format!("v{}", out_num);
                    let Some(oname) = xml_attr(&xml, &name_attr) else { break; };
                    let oval = xml_attr(&xml, &val_attr).unwrap_or("?");
                    if !oname.is_empty() {
                        println!("  {:30} = {}", oname, oval);
                    }
                    out_num += 1;
                }
            }
        },

        // ── Send / On / Off / Pulse ──────────────────────────────────────────
        Cmd::Send { name_or_uuid, command } => {
            let mut lox = LoxClient::new(Config::load()?);
            let uuid = lox.resolve(&name_or_uuid)?;
            let resp = lox.send_cmd(&uuid, &command)?;
            print_cmd_response(&resp, cli.json, &name_or_uuid, &command);
        },
        Cmd::On { name_or_uuid } => {
            let mut lox = LoxClient::new(Config::load()?);
            let uuid = lox.resolve(&name_or_uuid)?;
            let resp = lox.send_cmd(&uuid, "on")?;
            print_cmd_response(&resp, cli.json, &name_or_uuid, "on");
        },
        Cmd::Off { name_or_uuid } => {
            let mut lox = LoxClient::new(Config::load()?);
            let uuid = lox.resolve(&name_or_uuid)?;
            let resp = lox.send_cmd(&uuid, "off")?;
            print_cmd_response(&resp, cli.json, &name_or_uuid, "off");
        },
        Cmd::Pulse { name_or_uuid } => {
            let mut lox = LoxClient::new(Config::load()?);
            let uuid = lox.resolve(&name_or_uuid)?;
            let resp = lox.send_cmd(&uuid, "pulse")?;
            print_cmd_response(&resp, cli.json, &name_or_uuid, "pulse");
        },

        // ── Blind ───────────────────────────────────────────────────────────
        Cmd::Blind { name_or_uuid, action } => {
            let mut lox = LoxClient::new(Config::load()?);
            let ctrl = lox.find_control(&name_or_uuid)?;

            if !matches!(ctrl.typ.as_str(), "Jalousie" | "CentralJalousie") {
                bail!("'{}' is type '{}', not a Jalousie", ctrl.name, ctrl.typ);
            }

            let cmd = match action.to_lowercase().as_str() {
                "up"       | "open"  => "PulseUp",
                "down"     | "close" => "PulseDown",
                "stop"               => "off",
                "shade"              => "AutomaticDown",
                "full-up"            => "FullUp",
                "full-down"          => "FullDown",
                "auto"               => "AutomaticDown",
                other => bail!("Unknown blind action '{}'. Use: up down stop shade full-up full-down auto", other),
            };

            let resp = lox.send_cmd(&ctrl.uuid, cmd)?;
            print_cmd_response(&resp, cli.json, &ctrl.name, cmd);

            // Show position after command
            if !cli.json {
                thread::sleep(Duration::from_millis(300));
                let xml = lox.get_all(&ctrl.uuid)?;
                if let Some(pos) = xml_attr(&xml, "StatePos") {
                    let pos_f: f64 = pos.parse().unwrap_or(0.0);
                    println!("   Position: {:.0}%  {}", pos_f * 100.0,
                        pct_bar(pos_f, 1.0));
                }
            }
        },

        // ── Watch ───────────────────────────────────────────────────────────
        Cmd::Watch { name_or_uuid, interval } => {
            let mut lox = LoxClient::new(Config::load()?);
            let ctrl = lox.find_control(&name_or_uuid)?;
            println!("Watching '{}' every {}s  (Ctrl+C to stop)", ctrl.name, interval);

            let mut last = String::new();
            loop {
                match lox.get_all(&ctrl.uuid) {
                    Ok(xml) => {
                        let val = xml_attr(&xml, "value").unwrap_or("?").to_string();
                        if val != last {
                            let ts = now_hms();
                            if cli.json {
                                println!("{}", serde_json::json!({
                                    "time": ts, "control": ctrl.name, "value": val
                                }));
                            } else {
                                println!("[{}]  {}  =  {}", ts, ctrl.name, val);
                            }
                            last = val;
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
                thread::sleep(Duration::from_secs(interval));
            }
        },

        // ── If ──────────────────────────────────────────────────────────────
        Cmd::If { name_or_uuid, op, value } => {
            let mut lox = LoxClient::new(Config::load()?);
            let ctrl = lox.find_control(&name_or_uuid)?;
            let xml = lox.get_all(&ctrl.uuid)?;
            let current = xml_attr(&xml, "value").unwrap_or("").to_string();

            let matches = eval_op(&current, &op, &value)?;
            if !cli.json {
                println!("{} = {}  →  {} {} {}  →  {}",
                    ctrl.name, current, current, op, value,
                    if matches { "✓ true" } else { "✗ false" });
            } else {
                println!("{}", serde_json::json!({
                    "control": ctrl.name, "current": current,
                    "op": op, "target": value, "result": matches
                }));
            }
            std::process::exit(if matches { 0 } else { 1 });
        },

        // ── Run ─────────────────────────────────────────────────────────────
        Cmd::Run { scene } => {
            let s = Scene::load(&scene)?;
            let mut lox = LoxClient::new(Config::load()?);
            println!("▶  {}", s.name.as_deref().unwrap_or(&scene));
            if let Some(desc) = &s.description { println!("   {}", desc); }
            println!();

            for (i, step) in s.steps.iter().enumerate() {
                let uuid = match lox.resolve(&step.control) {
                    Ok(u) => u,
                    Err(e) => { eprintln!("Step {}: {}", i + 1, e); continue; }
                };
                let resp = lox.send_cmd(&uuid, &step.cmd)?;
                print_cmd_response(&resp, cli.json, &step.control, &step.cmd);
                if step.delay_ms > 0 {
                    thread::sleep(Duration::from_millis(step.delay_ms));
                }
            }
        },

        // ── Scene ───────────────────────────────────────────────────────────
        Cmd::Scene { action } => match action {
            SceneCmd::List => {
                let names = Scene::list()?;
                if names.is_empty() {
                    println!("No scenes. Create one: lox scene new <name>");
                } else {
                    for name in &names {
                        let desc = Scene::load(name).ok()
                            .and_then(|s| s.description)
                            .unwrap_or_default();
                        println!("  {:<20}  {}", name, desc);
                    }
                }
            },
            SceneCmd::Show { name } => {
                let path = Scene::scenes_dir().join(format!("{}.yaml", name));
                println!("{}", fs::read_to_string(&path)
                    .with_context(|| format!("Scene '{}' not found", name))?);
            },
            SceneCmd::New { name } => {
                let dir = Scene::scenes_dir();
                fs::create_dir_all(&dir)?;
                let path = dir.join(format!("{}.yaml", name));
                if path.exists() { bail!("Scene '{}' already exists", name); }
                fs::write(&path, format!(
r#"name: "{name}"
description: "Describe your scene"
steps:
  - control: "Control Name or UUID"
    cmd: "on"
  - control: "Another Control"
    cmd: "off"
    delay_ms: 500
"#))?;
                println!("✓  Created {:?}", path);
                println!("   Edit it and run: lox run {}", name);
            },
        },

        // ── Log ─────────────────────────────────────────────────────────────
        Cmd::Log { lines } => {
            let lox = LoxClient::new(Config::load()?);
            let log = lox.get("/dev/fsget/log/def.log")?;
            let all: Vec<&str> = log.lines().collect();
            let start = all.len().saturating_sub(lines);
            for line in &all[start..] {
                println!("{}", line);
            }
        },
    }

    Ok(())
}

fn eval_op(current: &str, op: &str, target: &str) -> Result<bool> {
    Ok(match op {
        "eq" | "==" => current == target,
        "ne" | "!=" => current != target,
        "contains"  => current.contains(target),
        "gt" | ">"  => parse_f(current)? > parse_f(target)?,
        "lt" | "<"  => parse_f(current)? < parse_f(target)?,
        "ge" | ">=" => parse_f(current)? >= parse_f(target)?,
        "le" | "<=" => parse_f(current)? <= parse_f(target)?,
        _ => bail!("Unknown op '{}'. Use: eq ne gt lt ge le contains", op),
    })
}

fn parse_f(s: &str) -> Result<f64> {
    s.parse::<f64>().with_context(|| format!("Cannot parse '{}' as number", s))
}

fn now_hms() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    format!("{:02}:{:02}:{:02}", (s % 86400) / 3600, (s % 3600) / 60, s % 60)
}

fn used_val_fmt(kb: f64) -> String {
    if kb > 1024.0 { format!("{:.0} MB", kb / 1024.0) }
    else { format!("{:.0} kB", kb) }
}
