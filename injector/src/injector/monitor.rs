use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::*;

use super::{
    config::Config,
    process::{Process, ProcessQuerier},
    window::{query_all_window, Window},
};

pub enum MonitorStatus {
    AddProcess(Process),
    SubProcess(Process),
    SyncWindow(Window),
}

type CB = Box<dyn Fn(&Config, MonitorStatus, &mut HashSet<u32>)>;
pub struct Monitor {
    cbs: Vec<(CB, HashSet<u32>)>,
    pec: HashMap<u32, (usize, String)>,
    wec: HashMap<String, (usize, u32, String)>,
    ic: usize,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            cbs: Vec::new(),
            pec: HashMap::new(),
            wec: HashMap::new(),
            ic: 0,
        }
    }

    pub fn register<F>(&mut self, f: F) -> &mut Self
    where
        F: 'static + Fn(&Config, MonitorStatus, &mut HashSet<u32>),
    {
        self.cbs.push((Box::new(f), HashSet::new()));
        self
    }

    pub fn watch_dog(&mut self, cfg: &Config, running: &Arc<AtomicBool>) {
        loop {
            self.ic += 1;

            let querier = ProcessQuerier::new();
            for process in querier {
                self.pec
                    .entry(process.pid)
                    .and_modify(|v: &mut (usize, String)| {
                        v.0 = self.ic;
                    })
                    .or_insert_with(|| {
                        for cb in self.cbs.iter_mut() {
                            cb.0(
                                cfg,
                                MonitorStatus::AddProcess(Process {
                                    name: process.name.clone(),
                                    pid: process.pid,
                                }),
                                &mut cb.1,
                            );
                        }
                        (self.ic, process.name)
                    });
            }

            self.pec = self
                .pec
                .iter()
                .filter_map(|(k, v)| {
                    if v.0 == self.ic {
                        Some((*k, v.clone()))
                    } else {
                        for cb in self.cbs.iter_mut() {
                            cb.0(
                                cfg,
                                MonitorStatus::SubProcess(Process {
                                    name: v.1.clone(),
                                    pid: *k,
                                }),
                                &mut cb.1,
                            );
                        }
                        None
                    }
                })
                .collect();

            query_all_window(|pid, title| {
                self.wec
                    .entry(pid.to_string() + title.as_str())
                    .and_modify(|v| {
                        v.0 = self.ic;
                    })
                    .or_insert_with(|| {
                        if let Entry::Occupied(o) = self.pec.entry(pid) {
                            for cb in self.cbs.iter_mut() {
                                cb.0(
                                    cfg,
                                    MonitorStatus::SyncWindow(Window {
                                        pid,
                                        name: o.get().1.clone(),
                                        title: title.clone(),
                                    }),
                                    &mut cb.1,
                                );
                            }
                        }

                        (self.ic, pid, title)
                    });
            });

            self.wec = self
                .wec
                .iter()
                .filter_map(|(k, v)| {
                    if v.0 == self.ic {
                        Some((k.to_owned(), v.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            if !running.load(Ordering::SeqCst) {
                break;
            }

            trace!("map size: {} {}", self.wec.len(), self.pec.len());
            std::thread::sleep(std::time::Duration::from_millis(cfg.monitor_interval()));
        }
    }
}
