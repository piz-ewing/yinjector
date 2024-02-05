use anyhow::Context;
use log::*;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

use super::util;

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";

// for Deserialize
#[derive(Deserialize)]
struct TGlobal {
    monitor_interval: Option<u64>,
    native: Option<bool>,
    exit_on_injected: Option<bool>,
}

#[derive(Deserialize)]
struct TConfig {
    global: Option<TGlobal>,
    base: Option<HashMap<String, String>>,
    window: Option<HashMap<String, String>>,
    module: Option<HashMap<String, String>>,
    delay: Option<HashMap<String, u64>>,
}

pub struct Config {
    pub monitor_interval: u64,
    pub native: bool,
    pub exit_on_injected: bool,
    pub base: HashMap<String, String>,
    pub window: HashMap<String, String>,
    pub module: HashMap<String, String>,
    pub delay: HashMap<String, u64>,
}

impl Config {
    const DEFAULT_MONITOR_INTERVAL: u64 = 500;
    const DEFAULT_NATIVE: bool = false;
    const DEFAULT_EXIT_ON_INJECTED: bool = false;

    pub fn parser(content: &str) -> anyhow::Result<Config> {
        let raw_config: TConfig = toml::from_str(content).context("")?;

        let mut config = Self {
            monitor_interval: Self::DEFAULT_MONITOR_INTERVAL,
            native: Self::DEFAULT_NATIVE,
            exit_on_injected: Self::DEFAULT_EXIT_ON_INJECTED,
            base: HashMap::new(),
            window: HashMap::new(),
            module: HashMap::new(),
            delay: HashMap::new(),
        };

        if let Some(global) = raw_config.global {
            if let Some(v) = global.monitor_interval {
                config.monitor_interval = v;
            }

            if let Some(v) = global.native {
                config.native = v;
            }

            if let Some(v) = global.exit_on_injected {
                config.exit_on_injected = v;
            }
            info!("[+] monitor_interval {}ms", config.monitor_interval);
            info!("[+] native {}", config.native);
            info!("[+] exit_on_injected {}", config.exit_on_injected);
        }

        if let Some(ps) = raw_config.base {
            for (process_name, dll_path) in &ps {
                if let Err(e) = config.add(process_name, dll_path) {
                    warn!("{}", e);
                }
            }
        }

        if let Some(ws) = raw_config.window {
            for (process_name, title_name) in ws {
                config.window.insert(process_name, title_name);
            }
        }

        if let Some(ms) = raw_config.module {
            for (process_name, module_name) in ms {
                config.module.insert(process_name, module_name);
            }
        }

        if let Some(ds) = raw_config.delay {
            for (process_name, delay) in ds {
                config.delay.insert(process_name, delay);
            }
        }

        if config.base.is_empty() {
            warn!("[!] inject empty")
        }

        Ok(config)
    }

    fn add(&mut self, process_name: &str, dll_path: &str) -> anyhow::Result<()> {
        // check dll file
        let abs_dll_path = Path::new(&dll_path);
        if abs_dll_path.is_file() {
            let abs_dll_path = util::adjust_canonicalization(abs_dll_path);

            self.base
                .insert(process_name.to_owned(), abs_dll_path.to_owned());

            info!("[+] {} -> {}", process_name, abs_dll_path);
            return Ok(());
        }

        // not abs path
        if !abs_dll_path.is_absolute() {
            let abs_dll_path = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join(dll_path);
            if abs_dll_path.is_file() {
                let abs_dll_path = util::adjust_canonicalization(abs_dll_path);

                self.base
                    .insert(process_name.to_owned(), abs_dll_path.to_owned());

                info!("[+] {} --> {}", process_name, abs_dll_path);
                return Ok(());
            }
        }

        anyhow::bail!(format!("dll file not exist {}", &dll_path))
    }
}
