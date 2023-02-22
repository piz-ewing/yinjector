use std::mem::{size_of_val, zeroed};

use windows::Win32::{Foundation::*, System::Diagnostics::ToolHelp::*};

fn wchar_arr_to_string(arr: &[CHAR]) -> String {
    let mut result = String::new();
    for c in arr.iter() {
        if c.0 == 0 {
            break;
        }
        result.push(c.0 as char);
    }
    result
}

pub struct Process {
    pub name: String,
    pub pid: u32,
}

pub struct Monitor {
    h_snapshot: HANDLE,
    is_first: bool,
}

impl Monitor {
    pub fn new() -> Monitor {
        unsafe {
            let h_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
            Monitor {
                is_first: true,
                h_snapshot,
            }
        }
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.h_snapshot);
        }
    }
}

impl Iterator for Monitor {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut pe32: PROCESSENTRY32 = zeroed();
            pe32.dwSize = size_of_val(&pe32) as u32;

            if self.is_first {
                self.is_first = false;
                if Process32First(self.h_snapshot, &mut pe32) == FALSE {
                    return None;
                }
            } else if Process32Next(self.h_snapshot, &mut pe32) == FALSE {
                return None;
            }

            Some(Process {
                name: wchar_arr_to_string(&pe32.szExeFile),
                pid: pe32.th32ProcessID,
            })
        }
    }
}
