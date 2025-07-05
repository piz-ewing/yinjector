use local_encoding_ng::{Encoder, Encoding};
use log::{debug, error, info, warn};
use prettytable::{row, Table};
use std::{ops::Not, path::Path, time::Duration};
use windows::{
    core::s,
    Win32::{
        Foundation::*,
        Security::*,
        System::{
            Diagnostics::{Debug::*, ToolHelp::*},
            LibraryLoader::*,
            Memory::*,
            Threading::*,
        },
    },
};

// yapi
use yapi_rs::yapi;

#[cfg(target_arch = "x86")]
use wow64ext_rs::wow64ext;

pub trait OptionExt {
    type Value;
    fn unwrap_ref(&self) -> &Self::Value;
    fn unwrap_mut(&mut self) -> &mut Self::Value;
}

impl<T> OptionExt for Option<T> {
    type Value = T;
    fn unwrap_ref(&self) -> &T {
        self.as_ref().unwrap()
    }
    fn unwrap_mut(&mut self) -> &mut T {
        self.as_mut().unwrap()
    }
}

pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;

    let p = p.as_ref().canonicalize().unwrap().display().to_string();
    p.strip_prefix(VERBATIM_PREFIX).unwrap_or(&p).to_string()
}

pub unsafe fn enum_process<T: FnMut(u32, String)>(mut f: T) {
    let Ok(h_snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
        return;
    };

    let _h_snapshot = scopeguard::guard(h_snapshot, |h_snapshot| {
        debug!("close handle");
        let _ = CloseHandle(h_snapshot);
    });

    let mut pe32: PROCESSENTRY32 = std::mem::zeroed();
    pe32.dwSize = std::mem::size_of_val(&pe32) as u32;

    if Process32First(h_snapshot, &mut pe32).is_err() {
        return;
    }

    loop {
        let exe = pe32
            .szExeFile
            .into_iter()
            .take_while(|x| x != &0)
            .map(|c| c as u8)
            .collect::<Vec<_>>();

        f(
            pe32.th32ProcessID,
            String::from_utf8_lossy(&exe).into_owned(),
        );

        if Process32Next(h_snapshot, &mut pe32).is_err() {
            break;
        }
    }
}

pub unsafe fn enum_module<T: FnMut(String) -> bool>(pid: u32, mut f: T) {
    let Ok(h_snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)
    else {
        return;
    };

    let _h_snapshot = scopeguard::guard(h_snapshot, |h_snapshot| {
        debug!("close handle");
        let _ = CloseHandle(h_snapshot);
    });

    let mut me32: MODULEENTRY32 = std::mem::zeroed();
    me32.dwSize = std::mem::size_of_val(&me32) as u32;

    if Module32First(h_snapshot, &mut me32).is_err() {
        return;
    }

    loop {
        let module = me32
            .szModule
            .into_iter()
            .take_while(|x| x != &0)
            .map(|c| c as u8)
            .collect::<Vec<_>>();

        if !f(String::from_utf8_lossy(&module).into_owned()) {
            break;
        }

        if Module32Next(h_snapshot, &mut me32).is_err() {
            break;
        }
    }
}

unsafe fn inject_by_yapi(pid: u32, name: &str, dll: &str) {
    // open remote process
    let Ok(h_proc) = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_READ
            | PROCESS_VM_WRITE
            | PROCESS_VM_OPERATION,
        false,
        pid,
    ) else {
        error!("[!] open {name} failed, code {:?}", GetLastError());
        return;
    };

    let _h_proc = scopeguard::guard(h_proc, |h_proc| {
        debug!("close handle");
        let _ = CloseHandle(h_proc);
    });

    let mut is_wow64 = FALSE;
    if IsWow64Process(h_proc, &mut is_wow64).is_err() {
        error!("[!] IsWow64Process failed, code {:?}", GetLastError());
        return;
    }

    if cfg!(target_arch = "x86") {
        if is_wow64.as_bool() {
            drop(_h_proc);
            info!("[!] yapi -> native");
            return inject_by_native(pid, name, dll);
        } else {
            warn!("[!] yapi x86 -> x64");
        }
    } else if is_wow64.as_bool().not() {
        drop(_h_proc);
        info!("[!] yapi -> native");
        return inject_by_native(pid, name, dll);
    } else {
        warn!("[!] yapi x64 -> x86");
    }

    let mut dll_acp = Encoding::ANSI.to_bytes(dll).unwrap();
    dll_acp.push(0);

    if yapi::yinject(
        h_proc.0 as _,
        dll_acp.as_ptr() as *const ::core::ffi::c_char,
        is_wow64.0,
    ) == 0
    {
        error!("[!] yinject remote thread failed",);
        return;
    }

    info!(
        "[+] yapi_inject {pid}:{name}({}) success",
        if is_wow64.as_bool() { "*32" } else { "*64" }
    );
}

#[cfg(target_arch = "x86")]
unsafe fn inject_by_wow64ext(pid: u32, name: &str, dll: &str) {
    // open remote process
    let Ok(h_proc) = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_READ
            | PROCESS_VM_WRITE
            | PROCESS_VM_OPERATION,
        false,
        pid,
    ) else {
        error!("[!] open {} failed, code {:?}", name, GetLastError());
        return;
    };

    let _h_proc = scopeguard::guard(h_proc, |h_proc| {
        debug!("close handle");
        let _ = CloseHandle(h_proc);
    });

    let mut is_wow64 = FALSE;
    if IsWow64Process(h_proc, &mut is_wow64).is_err() {
        error!("[!] IsWow64Process failed, code {:?}", GetLastError());
        return;
    }

    if is_wow64.as_bool() {
        drop(_h_proc);
        info!("[!] wow64ext -> native");
        return inject_by_native(pid, name, dll);
    } else {
        warn!("[!] wow64ext x86 -> x64");
    }

    let mut dll_acp = Encoding::ANSI.to_bytes(dll).unwrap();
    dll_acp.push(0);

    if wow64ext::inject64(
        h_proc.0 as *mut ::core::ffi::c_void,
        dll_acp.as_ptr() as *const ::core::ffi::c_char,
        INFINITE,
    ) == 0
    {
        error!("[!] inject64 remote thread failed",);
        return;
    }

    info!(
        "[+] wow64ext_inject {pid}:{name}({}) success",
        if is_wow64.as_bool() { "*32" } else { "*64" }
    );
}

unsafe fn inject_by_native(pid: u32, name: &str, dll: &str) {
    // get kernel32 module
    let Ok(kernel_module) = GetModuleHandleA(s!("kernel32.dll")) else {
        error!("[!] get kernel32 module failed, code {:?}", GetLastError());
        return;
    };

    // get LoadLibraryA address
    let Some(load_lib) = GetProcAddress(kernel_module, s!("LoadLibraryA")) else {
        error!(
            "[!] get func LoadLibraryA failed, code {:?}",
            GetLastError()
        );
        return;
    };

    // open remote process
    let Ok(h_proc) = OpenProcess(
        PROCESS_CREATE_THREAD
            | PROCESS_QUERY_INFORMATION
            | PROCESS_VM_READ
            | PROCESS_VM_WRITE
            | PROCESS_VM_OPERATION,
        false,
        pid,
    ) else {
        error!("[!] open {name} failed, code {:?}", GetLastError());
        return;
    };

    let _h_proc = scopeguard::guard(h_proc, |h_proc| {
        debug!("close handle");
        let _ = CloseHandle(h_proc);
    });

    let mut is_wow64 = FALSE;
    if IsWow64Process(h_proc, &mut is_wow64).is_err() {
        error!("[!] IsWow64Process failed, code {:?}", GetLastError());
        return;
    }

    if cfg!(target_arch = "x86") {
        if !is_wow64.as_bool() {
            warn!("[!] x86_native inject {name} x64 failed");
            return;
        }
    } else if is_wow64.as_bool() {
        warn!("[!] x64_native inject {name} x86 failed");
        return;
    }

    let mut dll_acp = Encoding::ANSI.to_bytes(dll).unwrap();
    dll_acp.push(0);

    // alloc remote memory
    let v_mem = VirtualAllocEx(
        h_proc,
        Some(std::ptr::null()),
        dll_acp.len(),
        MEM_RESERVE | MEM_COMMIT,
        PAGE_READWRITE,
    );
    let _v_mem = scopeguard::guard(v_mem, |v_mem| {
        debug!("free mem");
        let _ = VirtualFreeEx(h_proc, v_mem, 0, MEM_RELEASE);
    });

    // write dll path 2 remote memory
    if WriteProcessMemory(
        h_proc,
        v_mem,
        dll_acp.as_ptr() as _,
        dll_acp.len(),
        Some(std::ptr::null_mut::<usize>()),
    )
    .is_err()
    {
        error!(
            "[!] write remote process mem failed, code {:?}",
            GetLastError()
        );
        return;
    }

    // create remote thread
    let Ok(h_remote_thd) = CreateRemoteThread(
        h_proc,
        Some(std::ptr::null::<SECURITY_ATTRIBUTES>()),
        0_usize,
        // SAFETY: load_lib is guaranteed to point to a valid LoadLibraryA function pointer
        #[allow(clippy::missing_transmute_annotations)]
        Some(std::mem::transmute(load_lib)),
        Some(v_mem),
        0_u32,
        Some(std::ptr::null_mut::<u32>()),
    ) else {
        error!("[!] create remote thread failed, code {:?}", GetLastError());
        return;
    };

    let _h_remote_thd = scopeguard::guard(h_remote_thd, |h_remote_thd| {
        debug!("close thd");
        let _ = CloseHandle(h_remote_thd);
    });

    // wait for thread finish
    if WaitForSingleObject(h_remote_thd, INFINITE) != WAIT_OBJECT_0 {
        error!("[!] wait remote thread failed, code {:?}", GetLastError());
        return;
    }

    info!(
        "[+] native_inject {pid}:{name}({}) success",
        if is_wow64.as_bool() { "*32" } else { "*64" }
    );
}

pub unsafe fn inject_to_process(
    mode: &str,
    exit: bool,
    pid: u32,
    name: String,
    dll: String,
    delay: u32,
) {
    info!("[+] Inject Task...");
    {
        let mut table = Table::new();
        table.add_row(row!["pid", "name", "dll", "delay", "mode", "exit"]);
        table.add_row(row![pid, name, dll, delay, mode, exit]);
        table.printstd();
    }

    let mode_clone = mode.to_owned();
    let _ = std::thread::spawn(move || {
        if delay > 0 {
            std::thread::sleep(Duration::from_millis(delay as _));
        }
        match mode_clone.as_str() {
            "native" => inject_by_native(pid, &name, &dll),
            "yapi" => inject_by_yapi(pid, &name, &dll),
            #[cfg(target_arch = "x86")]
            "wow64ext" => inject_by_wow64ext(pid, &name, &dll),

            #[cfg(target_arch = "x86_64")]
            "wow64ext" => panic!("'wow64ext' mode is only supported on x86 builds"),
            _ => {
                panic!("invalid mode");
            }
        }
        if exit {
            std::process::exit(0);
        }
    });
}
