use anyhow::Context;
use log::*;
use std::{collections::HashSet, path::Path, rc::Rc};

// mod
mod config;
mod err;
mod util;

mod process;
mod window;

use config::*;
use util::*;

mod monitor;
use monitor::*;

struct Executor {
    already_injected: HashSet<u32>,
    config: Rc<Config>,
}

impl Executor {
    fn new(config: Rc<Config>) -> Self {
        Self {
            config,
            already_injected: HashSet::new(),
        }
    }

    fn inject_to_process(&mut self, pid: u32, name: &str, tag: &str) {
        if let Some(dll_path) = self.config.base.get(name) {
            info!(
                "[+] inject process:[{}] {} --> {} [{}]",
                tag,
                Path::new(&dll_path).file_name().unwrap().to_str().unwrap(),
                name,
                pid
            );

            if self.config.native {
                util::inject_by_native(pid, name, dll_path);
            } else {
                util::inject_by_yapi(pid, name, dll_path);
            }

            self.already_injected.insert(pid);
        }
    }
}

impl monitor::Reactor for Executor {
    fn received_notification(&mut self, e: MonitorEvent) {
        match e {
            MonitorEvent::AddProcess(p) => {
                if self.already_injected.get(&p.pid).is_some() {
                    return;
                }

                if self.config.module.get(&p.name).is_some() {
                    return;
                }

                if self.config.window.get(&p.name).is_some() {
                    return;
                }

                self.inject_to_process(p.pid, &p.name, "proc");
            }
            MonitorEvent::DelProcess(p) => {
                if self.already_injected.remove(&p.pid) {
                    info!("[-] process destory: {} [{}]", p.name, p.pid);
                }
            }
            MonitorEvent::NewWindow(w) => {
                let p = &w.p;
                if self.already_injected.get(&p.pid).is_some() {
                    return;
                }

                let title = self.config.window.get(&p.name);
                if title.is_none() || title.unwrap() == &w.title {
                    return;
                }

                self.inject_to_process(p.pid, &p.name, "win");
            }
            MonitorEvent::IncludeModule(p) => {
                if self.already_injected.get(&p.pid).is_some() {
                    return;
                }

                self.inject_to_process(p.pid, &p.name, "mod");
            }
        }
    }
}

pub struct Injector {
    config_path: Option<String>,
}

impl Injector {
    pub fn build() -> Self {
        Injector { config_path: None }
    }

    pub fn config_path(&mut self, file_path: Option<String>) -> &mut Self {
        self.config_path = file_path;
        self
    }

    pub fn watch(&mut self) -> anyhow::Result<()> {
        self.real_config_path()?;
        info!("[+] config path -> {}", self.config_path.unwrap_ref());

        let content = std::fs::read_to_string(self.config_path.unwrap_ref())
            .context(err::ERROR_READ_CONFIG)?;

        let config = Config::parser(&content)?;
        let config = Rc::new(config);
        Monitor::build(config.clone())
            .register(Box::new(Executor::new(config.clone())))
            .run();

        Ok(())
    }

    fn defalut_config_path() -> anyhow::Result<String> {
        let config_in_program_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(config::DEFAULT_CONFIG_FILE_NAME);

        if config_in_program_path.is_file() {
            return Ok(util::adjust_canonicalization(config_in_program_path));
        }

        let config_in_run_path = std::env::current_dir()
            .unwrap()
            .join(config::DEFAULT_CONFIG_FILE_NAME);

        if config_in_run_path.is_file() {
            return Ok(util::adjust_canonicalization(config_in_run_path));
        }

        anyhow::bail!(err::ERROR_CONFIG_NOT_EXIST);
    }

    fn real_config_path(&mut self) -> anyhow::Result<()> {
        if let Some(config_path) = self.config_path.as_ref() {
            let config_path = Path::new(config_path);
            if !config_path.is_file() {
                anyhow::bail!(err::ERROR_CONFIG_NOT_EXIST);
            }

            self.config_path = Some(util::adjust_canonicalization(config_path));

            return Ok(());
        } else {
            self.config_path = Some(Self::defalut_config_path()?);
        }

        Ok(())
    }
}
