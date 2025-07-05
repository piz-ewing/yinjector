use std::{collections::HashMap, path::Path};

use log::warn;
use prettytable::{row, Table};
use serde::Deserialize;

use super::util::{self, OptionExt};

// for Deserialize
#[derive(Deserialize)]
struct TGlobal {
    mode: Option<String>,
    exit: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct MixLimit {
    pub module: Option<String>,
    pub gui: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct Mix {
    pub name: String,
    pub dll: String,
    pub delay: Option<u32>,
    pub limit: Option<MixLimit>,
}

#[derive(Deserialize)]
struct TConfig {
    global: Option<TGlobal>,
    easy: Option<HashMap<String, String>>,
    mix: Option<Vec<Mix>>,
}

pub struct Global {
    pub mode: String,
    pub exit: bool,
}

impl Global {
    pub fn new() -> Self {
        Self {
            mode: "wow64ext".to_owned(),
            exit: false,
        }
    }
}

pub struct Config {
    pub global: Global,
    pub mix: HashMap<String, Mix>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            global: Global::new(),
            mix: HashMap::new(),
        }
    }
}

const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";

fn defalut_path() -> String {
    let program_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(DEFAULT_CONFIG_FILE_NAME);

    if program_path.is_file() {
        return util::adjust_canonicalization(program_path);
    }

    let run_path = std::env::current_dir()
        .unwrap()
        .join(DEFAULT_CONFIG_FILE_NAME);

    if run_path.is_file() {
        return util::adjust_canonicalization(run_path);
    }

    panic!("config not exist");
}

fn real_path(path: Option<&str>) -> String {
    let Some(p) = path else {
        return defalut_path();
    };

    let p = Path::new(p);
    if !p.is_file() {
        panic!("config not exist");
    }

    util::adjust_canonicalization(p)
}

fn real_dll_path(path: &str) -> String {
    // check dll file
    let abs = Path::new(&path);
    if abs.is_file() {
        return util::adjust_canonicalization(abs);
    }

    // not abs path
    if !abs.is_absolute() {
        let abs = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(path);
        if abs.is_file() {
            return util::adjust_canonicalization(abs);
        }
    }

    panic!("dll {path} not exist");
}

pub fn build(path: Option<&str>) -> Config {
    let r = real_path(path);

    let content = std::fs::read_to_string(r).unwrap();

    let tcfg: TConfig = toml::from_str(&content).unwrap();
    let mut cfg = Config::new();

    if let Some(global) = tcfg.global {
        if let Some(mode) = global.mode {
            cfg.global.mode = mode;
        }

        if let Some(exit) = global.exit {
            cfg.global.exit = exit;
        }
    }

    if let Some(easy) = tcfg.easy {
        for (name, dll) in easy.iter() {
            cfg.mix.insert(
                name.to_lowercase(),
                Mix {
                    name: name.to_lowercase(),
                    dll: real_dll_path(dll),
                    delay: None,
                    limit: None,
                },
            );
        }
    }

    if let Some(mix) = tcfg.mix {
        for mut t in mix {
            t.dll = real_dll_path(&t.dll);
            let name = t.name.to_lowercase();
            if cfg.mix.insert(name.clone(), t).is_some() {
                warn!("[!] {name} exist and update");
            }
        }
    }

    let mut table = Table::new();
    table.add_row(row![
        bFg->"mode", bFg->"exit", bFg->"name", bFg->"dll", bFg->"delay", bFg->"module", bFg->"gui"
    ]);

    for t in cfg.mix.values() {
        if t.limit.is_some() {
            table.add_row(row![
                cfg.global.mode,
                cfg.global.exit,
                t.name.to_lowercase(),
                t.dll.to_lowercase(),
                t.delay.unwrap_or(0),
                t.limit
                    .unwrap_ref()
                    .module
                    .as_ref()
                    .unwrap_or(&"None".to_string()),
                t.limit.unwrap_ref().gui.as_ref().unwrap_or(&false),
            ]);
        } else {
            table.add_row(row![
                t.name.to_lowercase(),
                t.dll.to_lowercase(),
                t.delay.unwrap_or(0),
                "None",
                "None"
            ]);
        }
    }
    table.printstd();

    cfg
}
