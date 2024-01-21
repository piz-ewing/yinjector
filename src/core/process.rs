use log::*;
use scopeguard::guard;
use std::mem::{size_of_val, zeroed};
use windows::Win32::{Foundation::*, System::Diagnostics::ToolHelp::*};

pub fn enum_process<T: FnMut(u32, String)>(mut f: T) {
    unsafe {
        let h_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if h_snapshot.is_err() {
            return;
        }

        let h_snapshot = h_snapshot.unwrap();
        let _h_snapshot = guard(h_snapshot, |h_snapshot| {
            trace!("close handle");
            let _ = CloseHandle(h_snapshot);
        });

        let mut pe32: PROCESSENTRY32 = zeroed();
        pe32.dwSize = size_of_val(&pe32) as u32;

        if Process32First(h_snapshot, &mut pe32).is_err() {
            return;
        }

        loop {
            // Find the index of the first null byte (0) in the array
            let null_index = pe32
                .szExeFile
                .iter()
                .position(|&x| x == 0)
                .unwrap_or(pe32.szExeFile.len());

            f(
                pe32.th32ProcessID,
                String::from_utf8_lossy(&pe32.szExeFile[..null_index]).into_owned(),
            );

            if Process32Next(h_snapshot, &mut pe32).is_err() {
                break;
            }
        }
    }
}

pub fn enum_module<T: FnMut(String) -> bool>(pid: u32, mut f: T) {
    unsafe {
        let h_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
        if h_snapshot.is_err() {
            return;
        }

        let h_snapshot = h_snapshot.unwrap();
        let _h_snapshot = guard(h_snapshot, |h_snapshot| {
            trace!("close handle");
            let _ = CloseHandle(h_snapshot);
        });

        let mut me32: MODULEENTRY32 = zeroed();
        me32.dwSize = size_of_val(&me32) as u32;

        if Module32First(h_snapshot, &mut me32).is_err() {
            return;
        }

        loop {
            // Find the index of the first null byte (0) in the array
            let null_index = me32
                .szModule
                .iter()
                .position(|&x| x == 0)
                .unwrap_or(me32.szModule.len());

            if !f(String::from_utf8_lossy(&me32.szModule[..null_index]).into_owned()) {
                break;
            }

            if Module32Next(h_snapshot, &mut me32).is_err() {
                error!("[!] enum module failed, code {:?}", GetLastError());
                break;
            }
        }
    }
}
