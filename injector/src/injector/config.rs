use super::util::*;
use log::*;
use serde::Deserialize;

use std::{collections::HashMap, path::Path};

// for Deserialize
#[derive(Deserialize)]
struct TGlobal {
    monitor_interval: Option<u64>,
    native: Option<bool>,
}

#[derive(Deserialize)]
struct TConfig {
    global: Option<TGlobal>,
    process: Option<HashMap<String, String>>,
    window: Option<HashMap<String, String>>,
}

pub fn init_config(cfg: &mut Config, cfg_str: &str) -> Result<(), String> {
    let tcfg: TConfig = toml::from_str(cfg_str).map_err(|_| "parser toml config failed")?;

    if let Some(g) = tcfg.global {
        if let Some(i) = g.monitor_interval {
            cfg.set_monitor_interval(i);
        }

        if let Some(n) = g.native {
            cfg.native = n
        }
    }

    if cfg.monitor_interval == 0 {
        cfg.set_monitor_interval_default();
    }

    if let Some(i) = tcfg.process {
        for (n, d) in i {
            cfg.add(n, d)?;
        }
    }

    if let Some(i) = tcfg.window {
        for (n, d) in i {
            cfg.window.insert(n, d);
        }
    }

    if cfg.process.is_empty() {
        warn!("[!] inject none")
    }

    Ok(())
}

pub struct Config {
    monitor_interval: u64,
    native: bool,
    process: HashMap<String, String>,
    window: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            process: HashMap::new(),
            window: HashMap::new(),
            monitor_interval: 0,
            native: false,
        }
    }

    pub fn monitor_interval(&self) -> u64 {
        self.monitor_interval
    }

    pub fn set_monitor_interval(&mut self, monitor_interval: u64) {
        self.monitor_interval = monitor_interval;
        info!("[+] set monitor interval {}ms", self.monitor_interval);
    }

    pub fn set_monitor_interval_default(&mut self) {
        const DEFAULT_MONITOR_INTERVAL: u64 = 500;

        self.monitor_interval = DEFAULT_MONITOR_INTERVAL;
        warn!("[!] default monitor interval {}ms", self.monitor_interval);
    }

    pub fn add(&mut self, process_name: String, dll_path: String) -> Result<&mut Self, String> {
        // check dll file
        let abs_dll_path = Path::new(&dll_path);
        if abs_dll_path.is_file() {
            let abs_dll_path = abs_dll_path
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let abs_dll_path = adjust_canonicalization(abs_dll_path);
            self.process
                .entry(process_name.to_owned())
                .or_insert(abs_dll_path.to_owned());
            info!("[+] {} -> {}", process_name, abs_dll_path);
            return Ok(self);
        }

        // not abs path
        if !abs_dll_path.is_absolute() {
            let abs_dll_path = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join(&dll_path);
            if abs_dll_path.is_file() {
                let abs_dll_path = abs_dll_path
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let abs_dll_path = adjust_canonicalization(abs_dll_path);

                self.process
                    .entry(process_name.to_owned())
                    .or_insert(abs_dll_path.to_owned());
                info!("[+] {} --> {}", process_name, abs_dll_path);
                return Ok(self);
            }
        }

        Err(format!("dll file not exist {}", &dll_path))
    }
    pub fn query_process(&self, process_name: &String) -> Option<String> {
        let process = self.process.get::<_>(process_name);
        process.map(|process| process.to_owned())
    }

    pub fn query_window(&self, process_name: &String) -> Option<String> {
        let window = self.window.get::<_>(process_name);
        window.map(|window| window.to_owned())
    }

    pub fn check_window(&self, process_name: &String, title: &String) -> bool {
        if let Some(t) = self.query_window(process_name) {
            &t == title
        } else {
            false
        }
    }

    pub fn using_native(&self) -> bool {
        self.native
    }
}
