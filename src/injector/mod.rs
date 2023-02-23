mod util;
use util::*;

mod config;
use config::*;

mod monitor;
use monitor::*;

use log::*;
use scopeguard::guard;
use serde_json::Value;
use std::{
    ffi::CString,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

// win32
use windows::{
    s,
    Win32::{
        Foundation::*,
        Security::*,
        System::{
            Diagnostics::Debug::*, LibraryLoader::*, Memory::*, Threading::*, WindowsProgramming::*,
        },
    },
};

pub struct Injector {
    cfg: Config,
}

impl Injector {
    fn inject(process: Process, dll_path: String) {
        unsafe {
            // get kernel32 module
            let kernel_module = GetModuleHandleA(s!("kernel32.dll"));
            if kernel_module.is_err() {
                error!("[!] get kernel32 module failed, code {:?}", GetLastError());
                return;
            }

            // get LoadLibraryA address
            let load_lib = GetProcAddress(kernel_module.unwrap(), s!("LoadLibraryA"));

            if load_lib.is_none() {
                error!(
                    "[!] get func LoadLibraryA failed, code {:?}",
                    GetLastError()
                );
                return;
            }

            // open remote process
            let h_proc = OpenProcess(
                PROCESS_CREATE_THREAD
                    | PROCESS_QUERY_INFORMATION
                    | PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION,
                FALSE,
                process.pid,
            );

            if h_proc.is_err() {
                error!(
                    "[!] open {} failed, code {:?}",
                    process.name,
                    GetLastError()
                );
                return;
            }

            let h_proc = h_proc.unwrap();
            let _h_proc = guard(h_proc, |h_proc| {
                trace!("close handle");
                CloseHandle(h_proc);
            });

            let dll_path = CString::new(dll_path).unwrap();
            let path_len = dll_path.to_bytes_with_nul().len();

            trace!("path --> {:?}", &dll_path);

            // alloc remote memory
            let v_mem = VirtualAllocEx(
                h_proc,
                Some(std::ptr::null()),
                path_len,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            );
            let _v_mem = guard(v_mem, |v_mem| {
                trace!("free mem");
                VirtualFreeEx(h_proc, v_mem, 0, MEM_RELEASE);
            });

            // write dll path 2 remote memory
            if WriteProcessMemory(
                h_proc,
                v_mem,
                dll_path.as_ptr() as *const ::core::ffi::c_void,
                path_len,
                Some(std::ptr::null_mut::<usize>()),
            ) == FALSE
            {
                error!(
                    "[!] write remote process mem failed, code {:?}",
                    GetLastError()
                );
                return;
            }

            // create remote thread
            let h_remote_thd = CreateRemoteThread(
                h_proc,
                Some(std::ptr::null::<SECURITY_ATTRIBUTES>()),
                0_usize,
                Some(std::mem::transmute(load_lib.unwrap())),
                Some(v_mem),
                0_u32,
                Some(std::ptr::null_mut::<u32>()),
            );

            if h_remote_thd.is_err() {
                error!("[!] create remote thread failed, code {:?}", GetLastError());
                return;
            }

            let h_remote_thd = h_remote_thd.unwrap();
            let _h_remote_thd = guard(h_remote_thd, |h_remote_thd| {
                trace!("close thd");
                CloseHandle(h_remote_thd);
            });

            // wait for thread finish
            if WaitForSingleObject(h_remote_thd, INFINITE).is_err() {
                error!("[!] wait remote thread failed, code {:?}", GetLastError());
                return;
            }
            info!("[+] inject success");
        }
    }

    pub fn new() -> Injector {
        Injector { cfg: Config::new() }
    }

    fn get_config_file(&mut self) -> Result<String, String> {
        let config_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("config.json");

        if config_path.is_file() {
            let config_path =
                adjust_canonicalization(config_path.canonicalize().unwrap().to_str().unwrap());
            return Ok(config_path);
        }

        let config_path = std::env::current_dir().unwrap().join("config.json");

        if config_path.is_file() {
            let config_path =
                adjust_canonicalization(config_path.canonicalize().unwrap().to_str().unwrap());
            return Ok(config_path);
        }

        Err("config file not exist".to_string())
    }

    pub fn build(&mut self, cfg_path: Option<String>) -> Result<&mut Self, String> {
        // check file exist
        let cfg_file = if let Some(s) = cfg_path {
            let p = Path::new(&s);
            if !p.is_file() {
                return Err("config file not exist".to_string());
            }
            adjust_canonicalization(p.canonicalize().unwrap().to_str().unwrap())
        } else {
            self.get_config_file()?
        };

        info!("[+] config path -> {:?}", cfg_file);
        let cfg_json = std::fs::read_to_string(cfg_file).map_err(|_| "read config failed")?;

        let cfg_json: Value =
            serde_json::from_str(&cfg_json).map_err(|_| "parser json config failed")?;

        let cfg_json = cfg_json.as_object();
        if cfg_json.is_none() {
            return Err("json format error".to_string());
        }

        for info in cfg_json.unwrap() {
            if info.1.is_string() {
                self.cfg
                    .add(info.0.to_string(), info.1.as_str().unwrap().to_string())?;
            } else if info.1.is_object() {
                unimplemented!();
            } else {
                return Err("invaild cfg".to_string());
            }
        }

        Ok(self)
    }

    pub fn watch(&mut self) -> Result<&mut Self, String> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))
            .expect("[!] setting Ctrl-C handler error");

        info!("[+] waiting for Ctrl-C...");

        Monitor::new()
            .register(|cfg, status| match status {
                ProcessStatus::AddProcess(process) => {
                    let dll_path = cfg.get(&process.name);
                    if dll_path.is_empty() {
                        return;
                    }
                    info!(
                        "[+] inject {} --> {} [{}]",
                        Path::new(&dll_path).file_name().unwrap().to_str().unwrap(),
                        process.name,
                        process.pid
                    );
                    Injector::inject(process, dll_path);
                }
                ProcessStatus::SubProcess(process) => {
                    if !cfg.get(&process.name).is_empty() {
                        info!("[-] process destory: {} [{}]", process.name, process.pid);
                    }
                }
            })
            .watch_dog(&self.cfg, &running);
        Ok(self)
    }
}
