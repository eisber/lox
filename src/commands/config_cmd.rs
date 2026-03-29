use anyhow::{Context, Result, bail};
use clap::CommandFactory;
use clap_complete::generate;
use reqwest::blocking::Client;
use std::fs;
use std::time::Duration;

use crate::client::{LOXONE_EPOCH_SECS, LoxClient, USER_AGENT};
use crate::commands::RunContext;
use crate::config::Config;
use crate::config_edit::ConfigEditor;
use crate::scene::Scene;
use crate::token;
use crate::{
    AliasCmd, CacheCmd, Cli, ConfigCmd, ControlCmd, MqttConfigCmd, RoomCmd, SceneCmd, SetupCmd,
    TokenCmd, XmlEditCmd, build_schema, detect_shell, ftp, gitops, install_completions,
    json_val_str, load_config_xml, loxcc, loxone_xml,
};

pub fn cmd_setup(ctx: &RunContext, action: SetupCmd) -> Result<()> {
    match action {
        SetupCmd::Set {
            host,
            user,
            pass,
            serial,
            verify_ssl,
            no_verify_ssl,
        } => {
            let mut cfg = Config::load().unwrap_or_default();
            if let Some(h) = host {
                cfg.host = if h.starts_with("http://") || h.starts_with("https://") {
                    h
                } else {
                    format!("https://{}", h)
                };
            }
            if let Some(u) = user {
                cfg.user = u;
            }
            if let Some(p) = pass {
                cfg.pass = p;
            }
            if let Some(s) = serial {
                cfg.serial = s;
            }
            if verify_ssl {
                cfg.verify_ssl = Some(true);
            } else if no_verify_ssl {
                cfg.verify_ssl = Some(false);
            }
            let path = cfg.save()?;
            if !ctx.quiet {
                println!("✓  Config saved to {:?}", path);
            }
        }
        SetupCmd::Show => {
            let cfg = Config::load()?;
            println!("host:   {}", cfg.host);
            println!("user:   {}", cfg.user);
            println!("pass:   {}", "*".repeat(cfg.pass.len()));
            if !cfg.serial.is_empty() {
                println!("serial: {}", cfg.serial);
            }
            if !cfg.aliases.is_empty() {
                println!("aliases:");
                let mut aliases: Vec<_> = cfg.aliases.iter().collect();
                aliases.sort_by_key(|(k, _)| k.as_str());
                for (name, uuid) in aliases {
                    println!("  {}: {}", name, uuid);
                }
            }
        }
    }
    Ok(())
}

pub fn cmd_alias(ctx: &RunContext, action: AliasCmd) -> Result<()> {
    let mut cfg = Config::load()?;
    match action {
        AliasCmd::Add { name, uuid } => {
            cfg.aliases.insert(name.clone(), uuid.clone());
            cfg.save()?;
            if !ctx.quiet {
                println!("✓  alias '{}' → {}", name, uuid);
            }
        }
        AliasCmd::Remove { name } => {
            if cfg.aliases.remove(&name).is_some() {
                cfg.save()?;
                if !ctx.quiet {
                    println!("✓  removed alias '{}'", name);
                }
            } else {
                println!("No alias named '{}'", name);
            }
        }
        AliasCmd::Ls => {
            if cfg.aliases.is_empty() {
                println!("No aliases. Add with: lox alias add <name> <uuid>");
            } else {
                let mut aliases: Vec<_> = cfg.aliases.iter().collect();
                aliases.sort_by_key(|(k, _)| k.as_str());
                if !ctx.no_header {
                    println!("{:<24} UUID", "ALIAS");
                    println!("{}", "─".repeat(60));
                }
                for (name, uuid) in aliases {
                    println!("{:<24} {}", name, uuid);
                }
            }
        }
    }
    Ok(())
}

pub fn cmd_scene(_ctx: &RunContext, action: SceneCmd) -> Result<()> {
    let cfg = Config::load().unwrap_or_default();
    match action {
        SceneCmd::Ls => {
            let names = Scene::list_with_config(&cfg)?;
            if names.is_empty() {
                println!("No scenes. Create: lox scene new <name>");
            } else {
                for name in &names {
                    let desc = Scene::load_with_config(name, &cfg)
                        .ok()
                        .and_then(|s| s.description)
                        .unwrap_or_default();
                    println!("  {:<20}  {}", name, desc);
                }
            }
        }
        SceneCmd::Show { name } => {
            println!(
                "{}",
                fs::read_to_string(Scene::scenes_dir_for(&cfg).join(format!("{}.yaml", name)))
                    .with_context(|| format!("Scene '{}' not found", name))?
            );
        }
        SceneCmd::New { name } => {
            let dir = Scene::scenes_dir_for(&cfg);
            fs::create_dir_all(&dir)?;
            let path = dir.join(format!("{}.yaml", name));
            if path.exists() {
                bail!("Already exists");
            }
            fs::write(
                &path,
                format!(
                    "name: \"{name}\"\ndescription: \"\"\nsteps:\n  - control: \"\"\n    cmd: \"on\"\n"
                ),
            )?;
            println!("✓  {:?}", path);
        }
    }
    Ok(())
}

pub fn cmd_cache(ctx: &RunContext, action: CacheCmd) -> Result<()> {
    let cfg = Config::load()?;
    let cache = LoxClient::cache_path(&cfg);
    match action {
        CacheCmd::Info => {
            if cache.exists() {
                let meta = cache.metadata()?;
                let age = std::time::SystemTime::now()
                    .duration_since(meta.modified()?)
                    .unwrap_or_default();
                let size = meta.len();
                println!("Cache: {:?}", cache);
                println!("Size:  {:.1} KB", size as f64 / 1024.0);
                println!("Age:   {}m {}s", age.as_secs() / 60, age.as_secs() % 60);
                if age.as_secs() < 86400 {
                    println!("Status: ✓ valid ({} until refresh)", {
                        let remaining = 86400u64.saturating_sub(age.as_secs());
                        format!("{}h {}m", remaining / 3600, (remaining % 3600) / 60)
                    });
                } else {
                    println!("Status: ⚠ stale (will refresh on next command)");
                }
            } else {
                println!("No cache. Will be created on first command.");
            }
        }
        CacheCmd::Clear => {
            if cache.exists() {
                fs::remove_file(&cache)?;
                println!("✓ Cache cleared");
            } else {
                println!("No cache to clear");
            }
        }
        CacheCmd::Check => {
            let lox = LoxClient::new(cfg)?;
            let resp = lox.get_json("/jdev/sps/LoxAPPversion3")?;
            let remote_ver = resp
                .pointer("/LL/value")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "remote_version": remote_ver,
                        "cache_exists": cache.exists(),
                    })
                );
            } else {
                println!("Remote structure version: {}", remote_ver);
                if cache.exists() {
                    println!("Cache: exists at {:?}", cache);
                } else {
                    println!("Cache: not present");
                }
            }
        }
        CacheCmd::Refresh => {
            let client = Client::builder()
                .user_agent(USER_AGENT)
                .danger_accept_invalid_certs(true)
                .redirect(LoxClient::same_origin_redirect_policy(&cfg.host))
                .timeout(Duration::from_secs(15))
                .build()?;
            if cache.exists() {
                let _ = fs::remove_file(&cache);
            }
            LoxClient::load_or_fetch_structure(&cfg, &client)?;
            println!("✓ Structure cache refreshed ({:?})", cache);
        }
    }
    Ok(())
}

pub fn cmd_token(ctx: &RunContext, action: TokenCmd) -> Result<()> {
    let cfg = Config::load()?;
    match action {
        TokenCmd::Fetch => {
            println!("Fetching token from Miniserver...");
            let rt = tokio::runtime::Runtime::new()?;
            match rt.block_on(token::acquire_token(&cfg)) {
                Ok(ts) => {
                    let _exp =
                        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.valid_until);
                    let days = ts.valid_until.saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ) / 86400;
                    println!("✓ Token saved to {:?}", token::TokenStore::path_for(&cfg));
                    println!("  Valid for: {} days", days);
                }
                Err(e) => bail!("Token fetch failed: {}", e),
            }
        }
        TokenCmd::Info => match token::TokenStore::load_for(&cfg) {
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let days_left = ts.valid_until.saturating_sub(now) / 86400;
                if ts.token.len() >= 12 {
                    println!(
                        "Token: {}...{}",
                        &ts.token[..8],
                        &ts.token[ts.token.len() - 4..]
                    );
                } else {
                    println!("Token: {}", ts.token);
                }
                if ts.is_valid() {
                    println!("Status: ✓ valid ({} days remaining)", days_left);
                } else {
                    println!("Status: ⚠ expired — run: lox token fetch");
                }
            }
            None => println!("No token saved. Using Basic Auth. Run: lox token fetch"),
        },
        TokenCmd::Clear => {
            let path = token::TokenStore::path_for(&cfg);
            if path.exists() {
                fs::remove_file(&path)?;
                println!("✓ Token cleared (reverting to Basic Auth)");
            } else {
                println!("No token to clear");
            }
        }
        TokenCmd::Check => {
            let ts = token::TokenStore::load_for(&cfg)
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run: lox token fetch"))?;
            let lox = LoxClient::new(cfg.clone())?;
            // Hash the token with the key for the check endpoint
            let hash = token::hash_token(&ts.token, &ts.key);
            let resp = lox.get_json(&format!("/jdev/sys/checktoken/{}/{}", hash, cfg.user))?;
            let code = resp
                .pointer("/LL/Code")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "valid": code == "200",
                        "code": code,
                    })
                );
            } else if code == "200" {
                println!("✓ Token is valid on the Miniserver");
            } else {
                println!("✗ Token is not valid (code {})", code);
            }
        }
        TokenCmd::Refresh => {
            let ts = token::TokenStore::load_for(&cfg)
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run: lox token fetch"))?;
            let lox = LoxClient::new(cfg.clone())?;
            let hash = token::hash_token(&ts.token, &ts.key);
            let resp = lox.get_json(&format!("/jdev/sys/refreshtoken/{}/{}", hash, cfg.user))?;
            let code = resp
                .pointer("/LL/Code")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if code == "200" {
                // Update the valid_until in our local store
                let valid_until = resp
                    .pointer("/LL/value")
                    .and_then(|v| v.get("validUntil"))
                    .and_then(|v| v.as_u64());
                if let Some(vu) = valid_until {
                    let unix_until = if vu > 1_500_000_000 {
                        vu
                    } else {
                        LOXONE_EPOCH_SECS.saturating_add(vu)
                    };
                    let new_ts = token::TokenStore {
                        token: ts.token,
                        key: ts.key,
                        valid_until: unix_until,
                    };
                    new_ts.save_for(&cfg)?;
                    let days = unix_until.saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ) / 86400;
                    println!("✓ Token refreshed ({} days remaining)", days);
                } else {
                    println!("✓ Token refreshed");
                }
            } else {
                bail!("Token refresh failed (code {})", code);
            }
        }
        TokenCmd::Revoke => {
            let ts = token::TokenStore::load_for(&cfg)
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run: lox token fetch"))?;
            let lox = LoxClient::new(cfg.clone())?;
            let hash = token::hash_token(&ts.token, &ts.key);
            let resp = lox.get_json(&format!("/jdev/sys/killtoken/{}/{}", hash, cfg.user))?;
            let code = resp
                .pointer("/LL/Code")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if code == "200" {
                // Remove local token
                let path = token::TokenStore::path_for(&cfg);
                if path.exists() {
                    fs::remove_file(&path)?;
                }
                println!("✓ Token revoked and cleared");
            } else {
                bail!("Token revoke failed (code {})", code);
            }
        }
    }
    Ok(())
}

pub fn cmd_config(ctx: &RunContext, action: ConfigCmd) -> Result<()> {
    match action {
        ConfigCmd::Ls => {
            let cfg = Config::load()?;
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                println!("No configs found on the Miniserver.");
            } else if ctx.json {
                let arr: Vec<serde_json::Value> = backups
                    .iter()
                    .map(|b| {
                        serde_json::json!({
                            "filename": b.filename,
                            "version": b.version,
                            "date": b.formatted_date(),
                            "size": b.size,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr)?);
            } else {
                println!("  {:<4} {:<8} {:<22} Size", "#", "Version", "Date");
                for (i, b) in backups.iter().enumerate() {
                    println!(
                        "  {:<4} {:<8} {:<22} {} KB{}",
                        i + 1,
                        b.version,
                        b.formatted_date(),
                        b.size / 1024,
                        if i == 0 { "  (latest)" } else { "" }
                    );
                }
            }
        }
        ConfigCmd::Download { save_as, extract } => {
            let cfg = Config::load()?;
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                bail!("No configs found on the Miniserver.");
            }
            let newest = &backups[0];
            eprintln!(
                "Downloading {} ({} KB)...",
                newest.filename,
                newest.size / 1024
            );
            let data = ftp::download_backup(&cfg, &newest.filename)?;
            let out_path = save_as.unwrap_or_else(|| newest.filename.clone());
            fs::write(&out_path, &data)?;
            println!("Saved to {}", out_path);

            if extract {
                eprintln!("Extracting sps0.LoxCC...");
                let xml = loxcc::extract_and_decompress(&data)?;
                let xml_path = out_path
                    .strip_suffix(".zip")
                    .unwrap_or(&out_path)
                    .to_string()
                    + ".Loxone";
                fs::write(&xml_path, &xml)?;
                println!(
                    "Decompressed {} KB → {} KB → {}",
                    data.len() / 1024,
                    xml.len() / 1024,
                    xml_path
                );
            }
        }
        ConfigCmd::Extract { file, save_as } => {
            let zip_data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            eprintln!("Extracting sps0.LoxCC...");
            let xml = loxcc::extract_and_decompress(&zip_data)?;
            let xml_path = save_as.unwrap_or_else(|| {
                file.strip_suffix(".zip").unwrap_or(&file).to_string() + ".Loxone"
            });
            fs::write(&xml_path, &xml)?;
            println!(
                "Decompressed {} KB → {} KB → {}",
                zip_data.len() / 1024,
                xml.len() / 1024,
                xml_path
            );
        }
        ConfigCmd::Upload { file, force } => {
            let cfg = Config::load()?;
            if !force {
                eprintln!(
                    "⚠  WARNING: Uploading a config will replace the current Miniserver\n\
                     \x20  programming. A bad configuration can require physical SD card\n\
                     \x20  access to recover.\n\
                     \n\
                     \x20  Config file: {}\n\
                     \n\
                     \x20  Use --force to proceed.",
                    file
                );
                std::process::exit(1);
            }
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let filename = std::path::Path::new(&file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file);
            eprintln!("Uploading {} ({} KB)...", filename, data.len() / 1024);
            ftp::upload_backup(&cfg, filename, &data)?;
            println!("Upload complete.");
            println!("Reboot the Miniserver to apply: lox reboot");
        }
        ConfigCmd::Users { file } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let users = loxone_xml::parse_users(&xml)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&users)?);
            } else {
                let nfc_count = users.iter().filter(|u| u.nfc).count();
                println!("  {:<26} {:<6} Description", "Name", "NFC");
                for u in &users {
                    println!(
                        "  {:<26} {:<6} {}",
                        u.name,
                        if u.nfc { "yes" } else { "-" },
                        u.description,
                    );
                }
                println!("\n{} users ({} with NFC)", users.len(), nfc_count);
            }
        }
        ConfigCmd::Devices { file } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let devices = loxone_xml::parse_devices(&xml)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            } else {
                let tree: Vec<_> = devices
                    .iter()
                    .filter(|d| d.bus == loxone_xml::DeviceBus::Tree)
                    .collect();
                let air: Vec<_> = devices
                    .iter()
                    .filter(|d| d.bus == loxone_xml::DeviceBus::Air)
                    .collect();
                let net: Vec<_> = devices
                    .iter()
                    .filter(|d| d.bus == loxone_xml::DeviceBus::Network)
                    .collect();

                if !tree.is_empty() {
                    println!("  Tree devices ({})", tree.len());
                    println!("  {:<30} {:<12} Type", "Name", "Serial");
                    for d in &tree {
                        println!(
                            "  {:<30} {:<12} {}",
                            d.name,
                            d.serial.as_deref().unwrap_or("-"),
                            d.type_label
                        );
                    }
                }
                if !air.is_empty() {
                    if !tree.is_empty() {
                        println!();
                    }
                    println!("  LoxAIR devices ({})", air.len());
                    println!("  {:<30} Type", "Name");
                    for d in &air {
                        println!("  {:<30} {}", d.name, d.type_label);
                    }
                }
                if !net.is_empty() {
                    if !tree.is_empty() || !air.is_empty() {
                        println!();
                    }
                    println!("  Network devices ({})", net.len());
                    println!("  {:<30} {:<18} MAC", "Name", "Address");
                    for d in &net {
                        println!(
                            "  {:<30} {:<18} {}",
                            d.name,
                            d.address.as_deref().unwrap_or("-"),
                            d.mac.as_deref().unwrap_or("-")
                        );
                    }
                }
                println!("\n{} devices total", devices.len());
            }
        }
        ConfigCmd::Diff { file1, file2 } => {
            let xml1 = load_config_xml(&file1)?;
            let xml2 = load_config_xml(&file2)?;
            let s1 = loxone_xml::parse_config_summary(&xml1)?;
            let s2 = loxone_xml::parse_config_summary(&xml2)?;
            let diff = loxone_xml::diff_configs(&s1, &s2);

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&diff)?);
            } else {
                println!(
                    "Config version: {} → {}",
                    diff.version_old, diff.version_new
                );
                println!("Modified: {} → {}", diff.date_old, diff.date_new);

                if !diff.controls_added.is_empty()
                    || !diff.controls_removed.is_empty()
                    || !diff.controls_changed.is_empty()
                {
                    println!("\nControls:");
                    for c in &diff.controls_added {
                        println!("  + Added: \"{}\" ({})", c.name, c.control_type);
                    }
                    for c in &diff.controls_changed {
                        println!(
                            "  ~ Changed: \"{}\" — {} \"{}\" → \"{}\"",
                            c.name, c.field, c.old_value, c.new_value
                        );
                    }
                    for c in &diff.controls_removed {
                        println!("  - Removed: \"{}\" ({})", c.name, c.control_type);
                    }
                }

                if !diff.rooms_added.is_empty()
                    || !diff.rooms_removed.is_empty()
                    || !diff.rooms_renamed.is_empty()
                {
                    println!("\nRooms:");
                    for r in &diff.rooms_added {
                        println!("  + Added: \"{}\"", r);
                    }
                    for r in &diff.rooms_renamed {
                        println!("  ~ Renamed: \"{}\" → \"{}\"", r.old, r.new);
                    }
                    for r in &diff.rooms_removed {
                        println!("  - Removed: \"{}\"", r);
                    }
                }

                if !diff.categories_added.is_empty()
                    || !diff.categories_removed.is_empty()
                    || !diff.categories_renamed.is_empty()
                {
                    println!("\nCategories:");
                    for c in &diff.categories_added {
                        println!("  + Added: \"{}\"", c);
                    }
                    for c in &diff.categories_renamed {
                        println!("  ~ Renamed: \"{}\" → \"{}\"", c.old, c.new);
                    }
                    for c in &diff.categories_removed {
                        println!("  - Removed: \"{}\"", c);
                    }
                }

                if !diff.users_added.is_empty() || !diff.users_removed.is_empty() {
                    println!("\nUsers:");
                    for u in &diff.users_added {
                        println!("  + Added: \"{}\"", u);
                    }
                    for u in &diff.users_removed {
                        println!("  - Removed: \"{}\"", u);
                    }
                }

                let total = diff.controls_added.len()
                    + diff.controls_removed.len()
                    + diff.controls_changed.len()
                    + diff.rooms_added.len()
                    + diff.rooms_removed.len()
                    + diff.rooms_renamed.len()
                    + diff.categories_added.len()
                    + diff.categories_removed.len()
                    + diff.categories_renamed.len()
                    + diff.users_added.len()
                    + diff.users_removed.len();

                if !diff.has_changes() {
                    println!("\nNo structural changes.");
                } else {
                    println!("\n{} changes total", total);
                }
            }
        }
        ConfigCmd::Init { path } => {
            let cfg = Config::load()?;
            let abs_path = if path.starts_with('/') || path.starts_with('~') {
                let expanded = if path.starts_with('~') {
                    path.replacen(
                        '~',
                        &dirs::home_dir().unwrap_or_default().to_string_lossy(),
                        1,
                    )
                } else {
                    path.clone()
                };
                std::path::PathBuf::from(expanded)
            } else {
                std::env::current_dir()?.join(&path)
            };
            let repo = gitops::init(&abs_path, &cfg)?;
            // Save the repo path in config
            let mut cfg = Config::load().unwrap_or_default();
            cfg.config_repo = Some(repo.to_string_lossy().to_string());
            let saved = cfg.save()?;
            println!("Config repo initialized at {}", repo.display());
            println!("Saved repo path to {}", saved.display());
            println!("\nNext: run `lox config pull` to download and commit the current config.");
        }
        ConfigCmd::Pull { quiet: pull_quiet } => {
            let cfg = Config::load()?;
            let repo_path = cfg
                .config_repo
                .as_deref()
                .context("No config repo configured. Run `lox config init <path>` first.")?;
            let q = pull_quiet || ctx.quiet;
            let committed = gitops::pull(std::path::Path::new(repo_path), &cfg, q)?;
            if q && committed {
                // In quiet/cron mode, just indicate a commit was made
                println!("committed");
            }
        }
        ConfigCmd::Log { count } => {
            let cfg = Config::load()?;
            let repo_path = cfg
                .config_repo
                .as_deref()
                .context("No config repo configured. Run `lox config init <path>` first.")?;
            let ms = if !cfg.serial.is_empty() {
                Some(cfg.serial.as_str())
            } else {
                None
            };
            let output = gitops::log(std::path::Path::new(repo_path), ms, count)?;
            if output.is_empty() {
                println!("No config history yet. Run `lox config pull` first.");
            } else {
                println!("{}", output);
            }
        }
        ConfigCmd::Restore { commit, force } => {
            let cfg = Config::load()?;
            let repo_path = cfg
                .config_repo
                .as_deref()
                .context("No config repo configured. Run `lox config init <path>` first.")?;
            gitops::restore(std::path::Path::new(repo_path), &cfg, &commit, force)?;
        }
        ConfigCmd::Compress { file, save_as } => {
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let loxcc_data = loxcc::compress_loxcc(&xml);
            let out_path = save_as.unwrap_or_else(|| {
                file.strip_suffix(".Loxone")
                    .or_else(|| file.strip_suffix(".loxone"))
                    .unwrap_or(&file)
                    .to_string()
                    + ".LoxCC"
            });
            fs::write(&out_path, &loxcc_data)?;
            println!(
                "Compressed {} KB → {} KB → {}",
                xml.len() / 1024,
                loxcc_data.len() / 1024,
                out_path
            );
        }
        ConfigCmd::Rooms { file } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let rooms = loxone_xml::parse_rooms(&xml)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&rooms)?);
            } else {
                println!("  {:<30} {:<6} UUID", "Room", "Items");
                println!(
                    "  {:<30} {:<6} {}",
                    "─".repeat(30),
                    "─".repeat(6),
                    "─".repeat(36)
                );
                for r in &rooms {
                    println!("  {:<30} {:<6} {}", r.name, r.item_count, r.uuid);
                }
                let total: usize = rooms.iter().map(|r| r.item_count).sum();
                println!("\n{} rooms, {} items total", rooms.len(), total);
            }
        }
        ConfigCmd::Controls { file, r#type, room } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let controls = loxone_xml::parse_controls(&xml, r#type.as_deref(), room.as_deref())?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&controls)?);
            } else {
                println!(
                    "  {:<20} {:<30} {:<20} {:<20} UUID",
                    "Type", "Title", "Room", "Category"
                );
                println!(
                    "  {:<20} {:<30} {:<20} {:<20} {}",
                    "─".repeat(20),
                    "─".repeat(30),
                    "─".repeat(20),
                    "─".repeat(20),
                    "─".repeat(36)
                );
                for c in &controls {
                    println!(
                        "  {:<20} {:<30} {:<20} {:<20} {}",
                        c.control_type, c.title, c.room, c.category, c.uuid
                    );
                }
                println!("\n{} controls", controls.len());
            }
        }
        ConfigCmd::Patch {
            replace,
            reboot,
            force,
        } => {
            if !force {
                eprintln!(
                    "⚠  WARNING: This will modify the live Miniserver configuration.\n\
                     \n\
                     \x20  Use --force to proceed."
                );
                std::process::exit(1);
            }
            if replace.len() % 2 != 0 {
                bail!("--replace requires pairs of OLD NEW values");
            }
            let cfg = Config::load()?;

            // Download current config
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                bail!("No configs found on the Miniserver.");
            }
            let newest = &backups[0];
            eprintln!("Downloading {}...", newest.filename);
            let zip_data = ftp::download_backup(&cfg, &newest.filename)?;

            // Extract XML
            let xml = loxcc::extract_and_decompress(&zip_data)?;
            let mut patched = xml.clone();

            // Apply replacements
            for pair in replace.chunks(2) {
                let old = pair[0].as_bytes();
                let new = pair[1].as_bytes();
                let count = patched.windows(old.len()).filter(|w| *w == old).count();
                if count == 0 {
                    eprintln!("  ⚠ Pattern '{}' not found in config", pair[0]);
                } else {
                    eprintln!(
                        "  ✓ Replacing '{}' → '{}' ({} occurrences)",
                        pair[0], pair[1], count
                    );
                    // Byte-level replacement
                    let mut result = Vec::with_capacity(patched.len());
                    let mut pos = 0;
                    while pos < patched.len() {
                        if pos + old.len() <= patched.len() && &patched[pos..pos + old.len()] == old
                        {
                            result.extend_from_slice(new);
                            pos += old.len();
                        } else {
                            result.push(patched[pos]);
                            pos += 1;
                        }
                    }
                    patched = result;
                }
            }

            if patched == xml {
                println!("No changes made.");
                return Ok(());
            }

            // Repack and upload
            let new_zip = loxcc::repack_zip(&zip_data, &patched)?;
            let upload_name = &newest.filename;
            eprintln!("Uploading patched config as {}...", upload_name);
            ftp::upload_backup(&cfg, upload_name, &new_zip)?;
            println!("✓ Patched config uploaded.");

            if reboot {
                eprintln!("Rebooting Miniserver...");
                let lox = crate::client::LoxClient::new(cfg)?;
                lox.get_text("/dev/sys/reboot")?;
                println!("✓ Reboot initiated.");
            } else {
                println!("Reboot the Miniserver to apply: lox reboot --yes");
            }
        }
        ConfigCmd::Push {
            file,
            reboot,
            force,
        } => {
            if !force {
                eprintln!(
                    "⚠  WARNING: This will upload a config to the live Miniserver.\n\
                     \n\
                     \x20  Use --force to proceed."
                );
                std::process::exit(1);
            }
            // Read the .Loxone XML
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let cfg = Config::load()?;

            // Download current backup ZIP as a template (for LoxAPP3.json, permissions.bin, etc.)
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                bail!("No configs found on the Miniserver to use as template.");
            }
            let newest = &backups[0];
            eprintln!("Downloading {} as template...", newest.filename);
            let template_zip = ftp::download_backup(&cfg, &newest.filename)?;

            // Repack with our edited XML
            let new_zip = loxcc::repack_zip(&template_zip, &xml)?;
            eprintln!("Uploading patched config ({} KB)...", new_zip.len() / 1024);
            ftp::upload_backup(&cfg, &newest.filename, &new_zip)?;
            println!("✓ Config uploaded.");

            if reboot {
                eprintln!("Rebooting Miniserver...");
                let lox = crate::client::LoxClient::new(cfg)?;
                lox.get_text("/dev/sys/reboot")?;
                println!("✓ Reboot initiated.");
            } else {
                println!("Reboot the Miniserver to apply: lox reboot --yes");
            }
        }
        ConfigCmd::PushHttp { file, force } => {
            if !force {
                eprintln!(
                    "⚠  WARNING: This will upload a config ZIP to the live Miniserver via HTTP.\n\
                     \n\
                     \x20  The Miniserver will auto-restart after receiving the file.\n\
                     \x20  Use --force to proceed."
                );
                std::process::exit(1);
            }

            let zip_data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            eprintln!("Read {} ({} KB)", file, zip_data.len() / 1024);

            let cfg = Config::load()?;
            let verify_ssl = cfg.verify_ssl.unwrap_or(false);

            // Build a client with cookie store for session persistence across requests.
            let client = Client::builder()
                .user_agent(USER_AGENT)
                .danger_accept_invalid_certs(!verify_ssl)
                .cookie_store(true)
                .timeout(Duration::from_secs(120))
                .build()
                .context("failed to build HTTP client")?;

            // Use HTTPS for all requests (fsput requires it)
            let https_host = cfg.host.replace("http://", "https://");
            eprintln!("Authenticating via HTTPS...");
            let _pub_key: serde_json::Value = client
                .get(format!("{}/jdev/sys/getPublicKey", https_host))
                .send()
                .context("getPublicKey request failed")?
                .json()
                .context("getPublicKey parse failed")?;

            // Step 2: Get key2 (HMAC key + salt + hash algorithm)
            let key2_resp: serde_json::Value = client
                .get(format!("{}/jdev/sys/getkey2/{}", https_host, cfg.user))
                .send()
                .context("getkey2 request failed")?
                .json()
                .context("getkey2 parse failed")?;
            let key2_val_raw = key2_resp
                .pointer("/LL/value")
                .ok_or_else(|| anyhow::anyhow!("getkey2: no value in response"))?;
            // value can be a JSON string (needs parsing) or already an object
            let key2_val: serde_json::Value = if let Some(s) = key2_val_raw.as_str() {
                serde_json::from_str(s).context("getkey2: failed to parse value JSON")?
            } else {
                key2_val_raw.clone()
            };
            let key_hex = key2_val
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("getkey2: missing 'key'"))?;
            let salt_hex = key2_val
                .get("salt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("getkey2: missing 'salt'"))?;

            // Step 3: Compute pwHash = SHA256("{pass}:{salt}").toUpperCase()
            // Note: salt is used as the raw hex string, NOT hex-decoded
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            let pw_hash = format!(
                "{:X}",
                <Sha256 as sha2::Digest>::digest(
                    format!("{}:{}", cfg.pass, salt_hex).as_bytes()
                )
            );

            // Step 4: sig = HMAC-SHA256(hex_decode(key), "{user}:{pwHash}") → lowercase hex
            let key_bytes =
                hex::decode(key_hex).context("getkey2: failed to hex-decode key")?;
            let mut mac =
                Hmac::<Sha256>::new_from_slice(&key_bytes).context("HMAC key error")?;
            mac.update(format!("{}:{}", cfg.user, pw_hash).as_bytes());
            let sig = hex::encode(mac.finalize().into_bytes());

            // Step 5: Get JWT token with permission 8 (CONFIG)
            let client_uuid = uuid::Uuid::new_v4().to_string();
            let jwt_resp: serde_json::Value = client
                .get(format!(
                    "{}/jdev/sys/getjwt/{}/{}/8/{}/lox-cli",
                    https_host, sig, cfg.user, client_uuid
                ))
                .send()
                .context("getjwt request failed")?
                .json()
                .context("getjwt parse failed")?;
            let jwt_code = jwt_resp
                .pointer("/LL/Code")
                .or_else(|| jwt_resp.pointer("/LL/code"))
                .and_then(crate::json_val_str)
                .unwrap_or_default();
            if jwt_code != "200" {
                bail!(
                    "getjwt failed (code {}): {}",
                    jwt_code,
                    serde_json::to_string_pretty(&jwt_resp)?
                );
            }
            eprintln!("✓ JWT acquired (permission=CONFIG)");

            // Extract token and key from getjwt response
            // The key from getjwt can be used like a getkey result for subsequent commands.
            let jwt_val = jwt_resp
                .pointer("/LL/value")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        serde_json::from_str::<serde_json::Value>(s).ok()
                    } else {
                        Some(v.clone())
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("getjwt: missing value"))?;
            let jwt_token = jwt_val
                .get("token")
                .and_then(|t| t.as_str())
                .ok_or_else(|| anyhow::anyhow!("getjwt: missing 'token' in response"))?
                .to_string();
            let jwt_key_hex = jwt_val
                .get("key")
                .and_then(|k| k.as_str())
                .ok_or_else(|| anyhow::anyhow!("getjwt: missing 'key' in response"))?
                .to_string();

            // Step 6: autht = HMAC-SHA1(hex_decode_to_ascii(key), token) — no case change
            // Per official protocol: key from getjwt is hex-decoded to ASCII bytes,
            // then used as HMAC-SHA1 key with the JWT token as message.
            use sha1::Sha1;
            type HmacSha1 = Hmac<Sha1>;
            let key_ascii_bytes =
                hex::decode(&jwt_key_hex).context("getjwt: failed to hex-decode key")?;
            let mut mac_autht =
                HmacSha1::new_from_slice(&key_ascii_bytes).context("HMAC-SHA1 key error")?;
            mac_autht.update(jwt_token.as_bytes());
            let autht = hex::encode(mac_autht.finalize().into_bytes());

            // Step 7: Establish /wsx WebSocket session (required before fsput)
            eprintln!("Establishing WebSocket session...");
            let wsx_url = format!(
                "{}/wsx?autht={}&user={}",
                https_host.replace("https://", "wss://").replace("http://", "ws://"),
                autht,
                cfg.user
            );
            let rt = tokio::runtime::Runtime::new()?;
            let wsx_result = rt.block_on(async {
                use tokio_tungstenite::connect_async_tls_with_config;
                use tokio_tungstenite::Connector;
                let tls_cfg = crate::ws::make_tls_config_pub();
                let req = tokio_tungstenite::tungstenite::http::Request::builder()
                    .uri(&wsx_url)
                    .header("Host", https_host.replace("https://", "").replace("http://", ""))
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Version", "13")
                    .header("Sec-WebSocket-Key", {
                        let mut bytes = [0u8; 16];
                        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
                        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
                    })
                    .body(())?;
                let (ws, _) = connect_async_tls_with_config(
                    req, None, false, Some(Connector::Rustls(tls_cfg))
                ).await?;
                Ok::<_, anyhow::Error>(ws)
            });
            let _wsx = match wsx_result {
                Ok(ws) => {
                    eprintln!("✓ WebSocket session established");
                    Some(ws)
                }
                Err(e) => {
                    eprintln!("⚠ WebSocket /wsx failed ({}), trying fsput anyway...", e);
                    None
                }
            };

            // Step 8: POST the ZIP via fsput
            eprintln!(
                "Uploading {} ({} KB) via HTTP POST...",
                file,
                zip_data.len() / 1024
            );
            let upload_url = format!(
                "{}/dev/fsput/lx/prog/sps_new.zip?autht={}&user={}",
                https_host, autht, cfg.user
            );
            let resp = client
                .post(&upload_url)
                .header("Content-Type", "application/octet-stream")
                .body(zip_data)
                .send()
                .context("fsput POST failed")?;
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            if !status.is_success() {
                bail!("fsput failed (HTTP {}): {}", status.as_u16(), body);
            }
            println!("✓ Config uploaded via HTTP. Miniserver will auto-restart.");
        }
        ConfigCmd::Add {
            file,
            control_type,
            title,
            room,
            category,
            parent,
            topic,
            save_as,
        } => {
            let (xml_type, default_parent) = match control_type.as_str() {
                "light" => ("LightController2", None),
                "switch" => ("Switch", None),
                "presence" => ("PresenceDetector", None),
                "alarm-clock" => ("AlarmClock", None),
                "memory" => ("Memory", None),
                "timer" => ("SwitchingTimer", None),
                "mqtt-sub" => ("GenTSensor", Some("gid:Mqtt")),
                "mqtt-pub" => ("GenTActor", Some("gid:Mqtt")),
                "calendar" => ("Calendar", None),
                "autopilot" => ("AutoPilot", None),
                other => bail!(
                    "Unknown control type '{}'. Valid: light, switch, presence, alarm-clock, memory, timer, mqtt-sub, mqtt-pub, calendar, autopilot",
                    other
                ),
            };

            let parent_sel = parent.as_deref().or(default_parent);
            let needs_parent = matches!(control_type.as_str(), "mqtt-sub" | "mqtt-pub");
            if needs_parent && parent_sel.is_none() {
                bail!("--parent is required for {}", control_type);
            }

            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            let cat_uuid = if let Some(ref c) = category {
                Some(editor.find_category_uuid(c)?)
            } else {
                None
            };

            let mut props: Vec<(&str, &str, &str)> = Vec::new();
            if let Some(ref t) = topic {
                props.push(("mqtt_topic", t.as_str(), "11"));
            }

            if let Some(actual_parent) = parent_sel {
                let uuid = editor.add_element(
                    actual_parent,
                    xml_type,
                    &title,
                    None,
                    room_uuid.as_deref(),
                    cat_uuid.as_deref(),
                    &props,
                )?;
                println!("✓ Added {} '{}' (UUID: {})", xml_type, title, uuid);
            } else {
                let uuid = editor.add_element_to_root(
                    xml_type,
                    &title,
                    room_uuid.as_deref(),
                    cat_uuid.as_deref(),
                    &props,
                )?;
                println!("✓ Added {} '{}' (UUID: {})", xml_type, title, uuid);
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::Validate { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let results = editor.validate_config();

            for r in &results {
                println!("{}", r);
            }
            let ok = results.iter().filter(|r| r.starts_with('✓')).count();
            let warn = results.iter().filter(|r| r.starts_with('⚠')).count();
            let err = results.iter().filter(|r| r.starts_with('✗')).count();
            println!("\n{} passed, {} warnings, {} errors", ok, warn, err);
        }
        ConfigCmd::UserAdd {
            file,
            name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let uuid = editor.add_user(&name)?;
            println!("✓ Added user '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::UserRemove {
            file,
            name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let uuid = editor.remove_user(&name)?;
            println!("✓ Removed user '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::DeviceBind {
            file,
            source,
            source_connector,
            target,
            target_connector,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.wire(&source, &source_connector, &target, &target_connector)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::AutopilotList { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let controls = loxone_xml::parse_controls(&data, Some("AutoPilot"), None)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&controls)?);
            } else {
                println!("  {:<30} {:<20} UUID", "Title", "Room");
                println!(
                    "  {:<30} {:<20} {}",
                    "─".repeat(30),
                    "─".repeat(20),
                    "─".repeat(36)
                );
                for c in &controls {
                    println!("  {:<30} {:<20} {}", c.title, c.room, c.uuid);
                }
                println!("\n{} autopilot rules", controls.len());
            }
        }
        ConfigCmd::AutopilotAdd {
            file,
            name,
            room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            let uuid =
                editor.add_element_to_root("AutoPilot", &name, room_uuid.as_deref(), None, &[])?;
            println!("✓ Added AutoPilot '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::CalendarAdd {
            file,
            name,
            room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            let uuid =
                editor.add_element_to_root("Calendar", &name, room_uuid.as_deref(), None, &[])?;
            println!("✓ Added Calendar '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::ModeList { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let controls = loxone_xml::parse_controls(&data, Some("Mode"), None)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&controls)?);
            } else {
                println!("  {:<30} UUID", "Title");
                println!("  {:<30} {}", "─".repeat(30), "─".repeat(36));
                for c in &controls {
                    println!("  {:<30} {}", c.title, c.uuid);
                }
                println!("\n{} operating modes", controls.len());
            }
        }
        ConfigCmd::AddVirtualIn {
            file,
            title,
            analog,
            parent,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let parent_sel = parent.as_deref().unwrap_or("Type:VirtualInCaption");
            let uuid = editor.add_virtual_in(&title, analog, parent_sel)?;
            // Find the AQ/Q connector UUID
            let conn_key = if analog { "AQ" } else { "Q" };
            let conn_uuid = editor.find_connector_uuid(&title, conn_key)?;
            if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "uuid": uuid,
                        "connector_key": conn_key,
                        "connector_uuid": conn_uuid,
                        "title": title,
                    })
                );
            } else {
                println!("✓ Created VirtualIn \"{}\" (UUID: {})", title, uuid);
                println!("  {} connector: {}", conn_key, conn_uuid);
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::WireConnector {
            file,
            target,
            source_uuid,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            // Parse target as "BlockTitle.ConnectorKey"
            let (block_title, conn_key) = target
                .rsplit_once('.')
                .ok_or_else(|| anyhow::anyhow!("Target must be 'BlockTitle.ConnectorKey'"))?;
            editor.wire_connector(block_title, conn_key, &source_uuid)?;
            if !ctx.quiet {
                println!("✓ Wired {}.{} ← {}", block_title, conn_key, source_uuid);
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::Room(action) => cmd_room(ctx, action)?,
        ConfigCmd::Control(action) => cmd_control(ctx, action)?,
        ConfigCmd::Mqtt(action) => cmd_mqtt_config(ctx, action)?,
        ConfigCmd::Xml(action) => cmd_xml_edit(ctx, action)?,
    }
    Ok(())
}

fn save_edited(editor: &ConfigEditor, original_path: &str, save_as: Option<&str>) -> Result<()> {
    let output = editor.to_bytes()?;
    let out_path = save_as.unwrap_or(original_path);
    fs::write(out_path, &output)?;
    eprintln!("✓ Saved to {}", out_path);
    Ok(())
}

fn cmd_room(_ctx: &RunContext, action: RoomCmd) -> Result<()> {
    match action {
        RoomCmd::Add {
            file,
            name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let uuid = editor.add_room(&name)?;
            println!("✓ Added room '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        RoomCmd::Rename {
            file,
            old_name,
            new_name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            // Find the room by title and rename
            let _path = editor.require_one(&old_name)?;
            let msg = editor.set_attribute(
                &format!("uuid:{}", {
                    let elem = editor.find_elements(&old_name);
                    if elem.is_empty() {
                        bail!("Room '{}' not found", old_name);
                    }
                    // Get UUID from the element
                    let p = &elem[0];
                    let mut current = &editor.root;
                    for &idx in p {
                        current = current.children[idx].as_element().unwrap();
                    }
                    current.attributes.get("U").cloned().unwrap_or_default()
                }),
                "Title",
                &new_name,
            )?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
    }
    Ok(())
}

fn cmd_control(ctx: &RunContext, action: ControlCmd) -> Result<()> {
    match action {
        ControlCmd::Move {
            file,
            to_room,
            type_filter,
            title,
            exclude,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let exclude_refs: Vec<&str> = exclude.iter().map(|s| s.as_str()).collect();

            if let Some(ref tf) = type_filter {
                let (count, room_uuid) = editor.move_to_room(tf, &to_room, &exclude_refs)?;
                println!(
                    "✓ Moved {} {} items to '{}' ({})",
                    count, tf, to_room, room_uuid
                );
            } else if let Some(ref t) = title {
                // Move single element by title
                let path = editor.require_one(t)?;
                let room_uuid = editor.find_room_uuid(&to_room)?;
                let elem = editor.get_element_mut(&path);
                for child in &mut elem.children {
                    if let Some(iodata) = child.as_mut_element()
                        && iodata.name == "IoData"
                    {
                        iodata
                            .attributes
                            .insert("Pr".to_string(), room_uuid.clone());
                    }
                }
                println!("✓ Moved '{}' to '{}' ({})", t, to_room, room_uuid);
            } else {
                bail!("Specify --type or --title to select controls to move");
            }

            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Rename {
            file,
            selector,
            new_name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.set_attribute(&selector, "Title", &new_name)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Describe { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let desc = editor.describe(&selector)?;

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&desc)?);
            } else {
                println!("  Type:     {}", desc.element_type);
                println!("  Title:    {}", desc.title);
                println!("  UUID:     {}", desc.uuid);
                if !desc.gid.is_empty() {
                    println!("  gid:      {}", desc.gid);
                }
                if !desc.room_uuid.is_empty() {
                    println!("  Room:     {}", desc.room_uuid);
                }
                if !desc.category_uuid.is_empty() {
                    println!("  Category: {}", desc.category_uuid);
                }
                if !desc.properties.is_empty() {
                    println!("  Properties:");
                    for (k, v) in &desc.properties {
                        println!("    {} = '{}' (t={})", k, v.value, v.type_code);
                    }
                }
                if !desc.connectors.is_empty() {
                    println!("  Connectors:");
                    for c in &desc.connectors {
                        println!("    {} → {}", c.kind, c.target);
                    }
                }
                if !desc.children.is_empty() {
                    println!("  Children:");
                    for c in &desc.children {
                        println!("    {}", c);
                    }
                }
            }
        }
        ControlCmd::Wire {
            file,
            source,
            target,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            // Parse "Selector.Connector" format
            let (src_sel, src_co) = source.rsplit_once('.').ok_or_else(|| {
                anyhow::anyhow!(
                    "Source must be 'ElementSelector.ConnectorName', got '{}'",
                    source
                )
            })?;
            let (tgt_sel, tgt_co) = target.rsplit_once('.').ok_or_else(|| {
                anyhow::anyhow!(
                    "Target must be 'ElementSelector.ConnectorName', got '{}'",
                    target
                )
            })?;

            let msg = editor.wire(src_sel, src_co, tgt_sel, tgt_co)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Unwire {
            file,
            connector,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let (sel, co_name) = connector.rsplit_once('.').ok_or_else(|| {
                anyhow::anyhow!(
                    "Must be 'ElementSelector.ConnectorName', got '{}'",
                    connector
                )
            })?;

            let msg = editor.unwire(sel, co_name)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Wires { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let wires = editor.list_wires(&selector)?;

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&wires)?);
            } else {
                let inputs: Vec<_> = wires.iter().filter(|w| w.direction == "input").collect();
                let outputs: Vec<_> = wires.iter().filter(|w| w.direction == "output").collect();
                let params: Vec<_> = wires
                    .iter()
                    .filter(|w| w.direction == "parameter")
                    .collect();

                if !inputs.is_empty() {
                    println!("  Inputs:");
                    for w in &inputs {
                        let status = if w.connected {
                            &w.target_uuid
                        } else {
                            "(unconnected)"
                        };
                        println!("    {:<20} ← {}", w.connector, status);
                    }
                }
                if !outputs.is_empty() {
                    println!("  Outputs:");
                    for w in &outputs {
                        let status = if w.connected {
                            &w.target_uuid
                        } else {
                            "(unconnected)"
                        };
                        println!("    {:<20} → {}", w.connector, status);
                    }
                }
                if !params.is_empty() {
                    println!("  Parameters:");
                    for w in &params {
                        let status = if w.connected {
                            &w.target_uuid
                        } else {
                            "(unconnected)"
                        };
                        println!("    {:<20}   {}", w.connector, status);
                    }
                }
                println!(
                    "\n{} connectors ({} connected, {} unconnected)",
                    wires.len(),
                    wires.iter().filter(|w| w.connected).count(),
                    wires.iter().filter(|w| !w.connected).count(),
                );
            }
        }
    }
    Ok(())
}

fn cmd_mqtt_config(ctx: &RunContext, action: MqttConfigCmd) -> Result<()> {
    match action {
        MqttConfigCmd::Setup {
            file,
            broker,
            port,
            user,
            password,
            client_id,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            editor.set_property("gid:Mqtt", "mqtt_broker_address", &broker, "11")?;
            eprintln!("  Broker: {}", broker);
            editor.set_property("gid:Mqtt", "mqtt_broker_port", &port, "7")?;
            eprintln!("  Port: {}", port);
            if let Some(u) = &user {
                editor.set_property("gid:Mqtt", "mqtt_auth_user", u, "11")?;
                eprintln!("  User: {}", u);
            }
            if let Some(p) = &password {
                editor.set_property("gid:Mqtt", "mqtt_auth_pwd", p, "11")?;
                eprintln!("  Password: (set, plaintext t=11)");
            }
            if let Some(c) = &client_id {
                editor.set_property("gid:Mqtt", "mqtt_client_id", c, "11")?;
                eprintln!("  Client ID: {}", c);
            }

            println!("✓ MQTT configured");
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        MqttConfigCmd::List { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let desc = editor.describe("gid:Mqtt")?;

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&desc)?);
            } else {
                println!("MQTT Plugin: {}", desc.title);
                if !desc.properties.is_empty() {
                    println!("  Configuration:");
                    for (k, v) in &desc.properties {
                        if k.contains("pwd") || k.contains("password") {
                            println!("    {} = *** (t={})", k, v.type_code);
                        } else {
                            println!("    {} = '{}' (t={})", k, v.value, v.type_code);
                        }
                    }
                }
                if !desc.children.is_empty() {
                    println!("  Topics ({}):", desc.children.len());
                    for c in &desc.children {
                        println!("    {}", c);
                    }
                }
            }
        }
        MqttConfigCmd::Topics { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let topics = editor.list_mqtt_topics();

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&topics)?);
            } else {
                println!("  {:<12} {:<30} {:<40} QoS", "Direction", "Title", "Topic");
                println!(
                    "  {:<12} {:<30} {:<40} {}",
                    "─".repeat(12),
                    "─".repeat(30),
                    "─".repeat(40),
                    "─".repeat(4)
                );
                for t in &topics {
                    println!(
                        "  {:<12} {:<30} {:<40} {}",
                        t.direction,
                        t.title,
                        if t.topic.is_empty() {
                            "(not set)"
                        } else {
                            &t.topic
                        },
                        if t.qos.is_empty() { "-" } else { &t.qos }
                    );
                }
                println!(
                    "\n{} topics ({} subscribe, {} publish)",
                    topics.len(),
                    topics.iter().filter(|t| t.direction == "subscribe").count(),
                    topics.iter().filter(|t| t.direction == "publish").count()
                );
            }
        }
    }
    Ok(())
}

fn cmd_xml_edit(_ctx: &RunContext, action: XmlEditCmd) -> Result<()> {
    match action {
        XmlEditCmd::SetProperty {
            file,
            selector,
            property,
            value,
            r#type,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.set_property(&selector, &property, &value, &r#type)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::SetAttr {
            file,
            selector,
            attr,
            value,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.set_attribute(&selector, &attr, &value)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::Move {
            file,
            type_filter,
            to_room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let (count, uuid) = editor.move_to_room(&type_filter, &to_room, &[])?;
            println!(
                "✓ Moved {} {} items to '{}' ({})",
                count, type_filter, to_room, uuid
            );
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::Add {
            file,
            parent,
            element_type,
            title,
            gid,
            room,
            category,
            property,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            // Resolve room/category names to UUIDs if provided
            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            // Category UUID lookup would be similar, but for now pass as-is
            let cat_uuid = category.as_deref();

            // Parse properties: "name:type=value"
            let props: Vec<(&str, &str, &str)> = property
                .iter()
                .filter_map(|p| {
                    let (name_type, value) = p.split_once('=')?;
                    let (name, type_code) = name_type.split_once(':')?;
                    Some((name, value, type_code))
                })
                .collect();

            let uuid = editor.add_element(
                &parent,
                &element_type,
                &title,
                gid.as_deref(),
                room_uuid.as_deref(),
                cat_uuid,
                &props,
            )?;
            println!("✓ Added {} '{}' (UUID: {})", element_type, title, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::Remove {
            file,
            uuid,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let title = editor.remove_element(&uuid)?;
            println!("✓ Removed '{}' (UUID: {})", title, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
    }
    Ok(())
}

pub fn cmd_completions(
    _ctx: &RunContext,
    shell: Option<clap_complete::Shell>,
    install: bool,
) -> Result<()> {
    let detected = shell.or_else(detect_shell);
    let Some(sh) = detected else {
        bail!("Could not detect shell. Specify one: lox completions bash|zsh|fish");
    };
    let mut cmd = Cli::command();
    if install {
        install_completions(sh, &mut cmd)?;
    } else {
        generate(sh, &mut cmd, "lox", &mut std::io::stdout());
    }
    Ok(())
}

pub fn cmd_schema(_ctx: &RunContext, command: Option<String>) -> Result<()> {
    let schema = build_schema(command.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
