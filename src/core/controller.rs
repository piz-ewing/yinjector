use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use log::trace;

use super::config::{self, Config};
use super::monitor::{Event, Listener};

use crate::core::util::{self, OptionExt};

pub struct Controller {
    config: Arc<RwLock<Option<Config>>>,
    cache: Arc<Mutex<HashMap<u32, config::Mix>>>,
}

impl Controller {
    fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn build(path: Option<&str>) -> Self {
        let c = Self::new();
        c.config.write().unwrap().replace(config::build(path));
        c
    }
}

impl Listener for Controller {
    fn trigger(&mut self, e: Event) {
        match e {
            Event::ProcessStart(pid, name) => {
                let r = self.config.read().unwrap();
                let cfg = r.unwrap_ref();
                let Some(v) = cfg.mix.get(&name) else {
                    return;
                };

                trace!("[+] ProcessStart: {pid} {name}");

                if v.limit.is_some() {
                    if self.cache.lock().unwrap().insert(pid, v.clone()).is_some() {
                        panic!("duplicate cache insert for pid {pid:?}");
                    }

                    // NOTE: Explicit return is here to prevent accidental fallthrough when future logic is added.
                    #[allow(clippy::needless_return)]
                    return;
                } else {
                    util::inject_to_process(
                        &cfg.global.mode,
                        cfg.global.exit,
                        pid,
                        v.name.clone(),
                        v.dll.clone(),
                        v.delay.unwrap_or(0),
                    );
                }
            }
            Event::ProcessStop(pid, name) => {
                let _ = self.cache.lock().unwrap().remove(&pid);

                let r = self.config.read().unwrap();
                let cfg = r.unwrap_ref();
                let Some(_) = cfg.mix.get(&name) else {
                    return;
                };
                trace!("[-] ProcessStop: {pid} {name}");
            }
            Event::GUIProcessStart(pid) => {
                let mut c = self.cache.lock().unwrap();
                let Some(v) = c.get_mut(&pid) else {
                    return;
                };

                trace!("[+] GUIProcessStart {pid}");

                let limit = v.limit.unwrap_mut();

                let Some(_) = limit.gui.as_ref() else {
                    return;
                };

                if limit.module.is_none() {
                    let r = self.config.read().unwrap();
                    let cfg = r.unwrap_ref();

                    util::inject_to_process(
                        &cfg.global.mode,
                        cfg.global.exit,
                        pid,
                        v.name.clone(),
                        v.dll.clone(),
                        v.delay.unwrap_or(0),
                    );

                    let _ = c.remove(&pid);
                } else {
                    limit.gui.take();
                }
            }
            Event::ImageLoad(pid, name) => {
                let mut c = self.cache.lock().unwrap();
                let Some(v) = c.get_mut(&pid) else {
                    return;
                };

                trace!("[+] ImageLoad: {pid} {name}");

                let limit = v.limit.unwrap_mut();

                let Some(m) = limit.module.as_ref() else {
                    return;
                };

                if m != &name {
                    return;
                }

                if limit.gui.is_none() {
                    let r = self.config.read().unwrap();
                    let cfg = r.unwrap_ref();

                    util::inject_to_process(
                        &cfg.global.mode,
                        cfg.global.exit,
                        pid,
                        v.name.clone(),
                        v.dll.clone(),
                        v.delay.unwrap_or(0),
                    );

                    let _ = c.remove(&pid);
                } else {
                    limit.module.take();
                }
            }
        }
    }
}
