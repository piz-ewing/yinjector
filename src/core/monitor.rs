use log::*;
use std::{
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use super::window::*;
use super::{process::*, Config};

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

pub struct WindowInfo {
    pub p: ProcessInfo,
    pub title: String,
}

// pub struct ModulesInfo {
//     pub p: ProcessInfo,
//     pub modules: HashSet<String>,
// }

pub enum MonitorEvent {
    AddProcess(ProcessInfo),
    DelProcess(ProcessInfo),

    NewWindow(WindowInfo),
    IncludeModule(ProcessInfo),
}

pub trait Reactor {
    fn received_notification(&mut self, _: MonitorEvent);
}

pub struct Monitor {
    callbacks: Vec<Box<dyn Reactor>>,
    config: Rc<Config>,
}

impl Monitor {
    pub fn build(config: Rc<Config>) -> Self {
        Self {
            callbacks: Vec::new(),
            config,
        }
    }

    pub fn register(&mut self, r: Box<dyn Reactor>) -> &mut Self {
        self.callbacks.push(r);
        self
    }

    pub fn run(&mut self) {
        let running = Arc::new(AtomicBool::new(true));
        let r: Arc<AtomicBool> = running.clone();

        ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))
            .expect("[!] setting Ctrl-C handler error");

        info!("[+] waiting for Ctrl-C...");

        let mut process_statistics = HashMap::new();
        let mut window_statistics = HashMap::new();
        let mut module_statistics = HashMap::new();

        let mut monitor_count = 0;
        loop {
            monitor_count += 1;

            // get all process
            enum_process(|pid, name| {
                process_statistics
                    .entry(pid)
                    .and_modify(|v: &mut (usize, String)| {
                        v.0 = monitor_count;
                    })
                    .or_insert_with(|| {
                        for cb in self.callbacks.iter_mut() {
                            cb.received_notification(MonitorEvent::AddProcess(ProcessInfo {
                                pid,
                                name: name.to_owned(),
                            }));
                        }
                        (monitor_count, name.clone())
                    });

                if let Some(module) = self.config.module.get(&name) {
                    match module_statistics.entry(pid.to_string() + name.as_str()) {
                        Entry::Occupied(mut o) => {
                            *o.get_mut() = monitor_count;
                        }
                        Entry::Vacant(v) => {
                            let mut found_module = false;
                            enum_module(pid, |module_name| -> bool {
                                if module.to_lowercase() == module_name.to_lowercase() {
                                    for cb in self.callbacks.iter_mut() {
                                        cb.received_notification(MonitorEvent::IncludeModule(
                                            ProcessInfo {
                                                pid,
                                                name: name.to_owned(),
                                            },
                                        ));
                                    }
                                    found_module = true;
                                    return false;
                                }
                                true
                            });

                            if found_module {
                                v.insert(monitor_count);
                            }
                        }
                    }
                }
            });

            // remove invalid module
            module_statistics.retain(|_, v| *v == monitor_count);

            // remove invalid process
            process_statistics.retain(|k, v| {
                if v.0 != monitor_count {
                    for cb in self.callbacks.iter_mut() {
                        cb.received_notification(MonitorEvent::DelProcess(ProcessInfo {
                            pid: *k,
                            name: v.1.to_owned(),
                        }));
                    }
                    return false;
                }
                true
            });

            enum_window(|pid, title| {
                window_statistics
                    .entry(pid.to_string() + title.as_str())
                    .and_modify(|v: &mut usize| {
                        *v = monitor_count;
                    })
                    .or_insert_with(|| {
                        if let Some(v) = process_statistics.get(&pid) {
                            for cb in self.callbacks.iter_mut() {
                                cb.received_notification(MonitorEvent::NewWindow(WindowInfo {
                                    p: ProcessInfo {
                                        pid,
                                        name: v.1.clone(),
                                    },
                                    title: title.clone(),
                                }));
                            }
                        }
                        monitor_count
                    });
            });

            // remove invalid window
            window_statistics.retain(|_, v| *v == monitor_count);

            if !running.load(Ordering::SeqCst) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(
                self.config.monitor_interval,
            ));
        }
    }
}
