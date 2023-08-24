use std::{
    collections::HashMap,
    mem::{size_of_val, zeroed},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::*;
use windows::Win32::{Foundation::*, System::Diagnostics::ToolHelp::*};

use super::config::Config;

pub struct Process {
    pub name: String,
    pub pid: u32,
}

pub enum ProcessStatus {
    AddProcess(Process),
    SubProcess(Process),
}

struct ProcessQuerier {
    is_first: bool,
    h_snapshot: HANDLE,
}

impl ProcessQuerier {
    fn new() -> ProcessQuerier {
        unsafe {
            let h_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
            ProcessQuerier {
                is_first: true,
                h_snapshot,
            }
        }
    }
}

impl Drop for ProcessQuerier {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.h_snapshot);
        }
    }
}

impl Iterator for ProcessQuerier {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut pe32: PROCESSENTRY32 = zeroed();
            pe32.dwSize = size_of_val(&pe32) as u32;

            if self.is_first {
                self.is_first = false;
                if Process32First(self.h_snapshot, &mut pe32).is_err() {
                    return None;
                }
            } else if Process32Next(self.h_snapshot, &mut pe32).is_err() {
                return None;
            }

            // Find the index of the first null byte (0) in the array
            let null_index = pe32
                .szExeFile
                .iter()
                .position(|&x| x == 0)
                .unwrap_or(pe32.szExeFile.len());

            Some(Process {
                name: String::from_utf8_lossy(&pe32.szExeFile[..null_index]).into_owned(),
                pid: pe32.th32ProcessID,
            })
        }
    }
}

type CB = Box<dyn Fn(&Config, ProcessStatus)>;
pub struct Monitor {
    cbs: Vec<CB>,
    rec: HashMap<u32, (usize, String)>,
    ic: usize,
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            cbs: Vec::new(),
            rec: HashMap::new(),
            ic: 0,
        }
    }

    pub fn register<F>(&mut self, f: F) -> &mut Self
    where
        F: 'static + Fn(&Config, ProcessStatus),
    {
        self.cbs.push(Box::new(f));
        self
    }

    pub fn watch_dog(&mut self, cfg: &Config, running: &Arc<AtomicBool>) {
        loop {
            self.ic += 1;

            let querier = ProcessQuerier::new();
            for process in querier {
                self.rec
                    .entry(process.pid)
                    .and_modify(|v| v.0 = self.ic)
                    .or_insert_with(|| {
                        for cb in self.cbs.iter() {
                            cb(
                                cfg,
                                ProcessStatus::AddProcess(Process {
                                    name: process.name.clone(),
                                    pid: process.pid,
                                }),
                            );
                        }
                        (self.ic, process.name)
                    });
            }

            self.rec = self
                .rec
                .iter()
                .filter_map(|(k, v)| {
                    if v.0 == self.ic {
                        Some((*k, v.clone()))
                    } else {
                        for cb in self.cbs.iter() {
                            cb(
                                cfg,
                                ProcessStatus::SubProcess(Process {
                                    name: v.1.clone(),
                                    pid: *k,
                                }),
                            );
                        }
                        None
                    }
                })
                .collect();

            if !running.load(Ordering::SeqCst) {
                break;
            }

            trace!("map size: {}", self.rec.len());
            std::thread::sleep(std::time::Duration::from_millis(cfg.monitor_interval()));
        }
    }
}
