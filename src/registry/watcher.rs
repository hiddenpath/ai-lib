//! Configuration Watcher
//!
//! Monitors the manifest file for changes and updates the global registry.
//! This module is gated by the "config_hot_reload" feature.

use crate::manifest::ManifestLoader;
use crate::manifest::ManifestValidator;
use crate::registry::RegistryConfig;
use crate::registry::GLOBAL_REGISTRY;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// 最近一次成功转换的Registry配置，用于解析失败时的回退
static LAST_GOOD_CONFIG: Lazy<Mutex<Option<RegistryConfig>>> = Lazy::new(|| Mutex::new(None));

/// Starts a background thread to watch the manifest file for changes.
///
/// # Arguments
///
/// * `path` - Path to the manifest file
///
/// # Returns
///
/// Returns a handle to the watcher (to keep it alive) or an error.
pub fn start_watcher<P: AsRef<Path>>(path: P) -> crate::Result<()> {
    let path = path.as_ref().to_path_buf();

    // Spawn a thread to run the watcher loop
    thread::spawn(move || {
        if let Err(e) = watch_loop(path) {
            eprintln!("Config watcher error: {:?}", e);
        }
    });

    Ok(())
}

fn watch_loop(path: PathBuf) -> notify::Result<()> {
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    eprintln!("Started watching config file: {:?}", path);

    for res in rx {
        match res {
            Ok(event) => {
                use notify::EventKind;
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        // Give it a small delay for atomic writes to settle
                        thread::sleep(Duration::from_millis(100));
                        reload_config(&path);
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

fn reload_config(path: &Path) {
    eprintln!("Reloading configuration from {:?}", path);
    match std::fs::read_to_string(path) {
        Ok(content) => match ManifestLoader::load_from_string(&content) {
            Ok(manifest) => {
                // 额外逻辑验证（冗余安全，ManifestLoader 已包含）
                if let Err(e) = ManifestValidator::validate_manifest(&manifest) {
                    eprintln!("Manifest validation failed: {}", e);
                    restore_last_good();
                    return;
                }

                let config = RegistryConfig::from_manifest(&manifest);
                GLOBAL_REGISTRY.merge_config(config.clone());
                if let Ok(mut guard) = LAST_GOOD_CONFIG.lock() {
                    *guard = Some(config);
                }
                eprintln!("Global Registry updated successfully.");
            }
            Err(e) => {
                eprintln!("Failed to parse/validate YAML manifest: {}", e);
                restore_last_good();
            }
        },
        Err(e) => eprintln!("Failed to read config file: {}", e),
    }
}

fn restore_last_good() {
    if let Ok(guard) = LAST_GOOD_CONFIG.lock() {
        if let Some(cfg) = guard.as_ref() {
            GLOBAL_REGISTRY.merge_config(cfg.clone());
            eprintln!("Restored last known good registry configuration.");
        } else {
            eprintln!("No previous valid registry configuration to restore.");
        }
    }
}
