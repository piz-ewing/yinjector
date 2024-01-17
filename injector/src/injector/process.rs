use std::mem::{size_of_val, zeroed};
use windows::Win32::{Foundation::*, System::Diagnostics::ToolHelp::*};

pub struct Process {
    pub name: String,
    pub pid: u32,
}

pub struct ProcessQuerier {
    is_first: bool,
    h_snapshot: HANDLE,
}

impl ProcessQuerier {
    pub fn new() -> ProcessQuerier {
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
