use log::*;
use std::{collections::HashMap, path::Path};

use super::util::*;

pub struct Config {
    info: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            info: HashMap::new(),
        }
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
            self.info
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

                self.info
                    .entry(process_name.to_owned())
                    .or_insert(abs_dll_path.to_owned());
                info!("[+] {} --> {}", process_name, abs_dll_path);
                return Ok(self);
            }
        }

        return Err(format!("dll file not exist {}", &dll_path));
    }

    pub fn get(&self, process_name: &String) -> String {
        let info = self.info.get::<_>(process_name);
        if let Some(info) = info {
            info.to_owned()
        } else {
            String::new()
        }
    }
}
