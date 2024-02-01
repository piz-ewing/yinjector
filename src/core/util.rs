use log::*;
use std::path::Path;

use local_encoding_ng::{Encoder, Encoding};

use windows::{
    core::s,
    Win32::{
        Foundation::*,
        Security::*,
        System::{Diagnostics::Debug::*, LibraryLoader::*, Memory::*, Threading::*},
    },
};
// yapi
use yapi_rs::yapi;

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

pub fn inject_by_yapi(pid: u32, name: &str, dll_path: &str) {
    unsafe {
        // open remote process
        let Ok(h_proc) = OpenProcess(
            PROCESS_CREATE_THREAD
                | PROCESS_QUERY_INFORMATION
                | PROCESS_VM_READ
                | PROCESS_VM_WRITE
                | PROCESS_VM_OPERATION,
            FALSE,
            pid,
        ) else {
            error!("[!] open {} failed, code {:?}", name, GetLastError());
            return;
        };

        let _h_proc = scopeguard::guard(h_proc, |h_proc| {
            trace!("close handle");
            let _ = CloseHandle(h_proc);
        });

        let mut is_wow64 = FALSE;
        if IsWow64Process(h_proc, &mut is_wow64).is_err() {
            error!("[!] IsWow64Process failed, code {:?}", GetLastError());
            return;
        }

        let mut dll_path_acp = Encoding::ANSI.to_bytes(dll_path).unwrap();
        dll_path_acp.push(0);

        if yapi::yinject(
            h_proc.0 as *mut ::core::ffi::c_void,
            dll_path_acp.as_ptr() as *const ::core::ffi::c_char,
            is_wow64.0,
        ) == 0
        {
            error!("[!] yinject remote thread failed",);
            return;
        }

        info!(
            "[+] {} inject success",
            if is_wow64.as_bool() { "x86" } else { "x64" }
        );
    }
}

pub fn inject_by_native(pid: u32, name: &str, dll_path: &str) {
    unsafe {
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
            FALSE,
            pid,
        ) else {
            error!("[!] open {} failed, code {:?}", name, GetLastError());
            return;
        };

        let _h_proc = scopeguard::guard(h_proc, |h_proc| {
            trace!("close handle");
            let _ = CloseHandle(h_proc);
        });

        let mut dll_path_acp = Encoding::ANSI.to_bytes(dll_path).unwrap();
        dll_path_acp.push(0);

        trace!("path --> {:?}", &dll_path);

        // alloc remote memory
        let v_mem = VirtualAllocEx(
            h_proc,
            Some(std::ptr::null()),
            dll_path_acp.len(),
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        );
        let _v_mem = scopeguard::guard(v_mem, |v_mem| {
            trace!("free mem");
            let _ = VirtualFreeEx(h_proc, v_mem, 0, MEM_RELEASE);
        });

        // write dll path 2 remote memory
        if WriteProcessMemory(
            h_proc,
            v_mem,
            dll_path_acp.as_ptr() as _,
            dll_path_acp.len(),
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
            Some(std::mem::transmute(load_lib)),
            Some(v_mem),
            0_u32,
            Some(std::ptr::null_mut::<u32>()),
        ) else {
            error!("[!] create remote thread failed, code {:?}", GetLastError());
            return;
        };

        let _h_remote_thd = scopeguard::guard(h_remote_thd, |h_remote_thd| {
            trace!("close thd");
            let _ = CloseHandle(h_remote_thd);
        });

        // wait for thread finish
        if WaitForSingleObject(h_remote_thd, INFINITE) != WAIT_OBJECT_0 {
            error!("[!] wait remote thread failed, code {:?}", GetLastError());
            return;
        }
        info!("[+] inject success");
    }
}
