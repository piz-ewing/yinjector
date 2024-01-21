use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId,
};

unsafe extern "system" fn callback_enum_windows<T: FnMut(u32, String)>(
    window: HWND,
    param: LPARAM,
) -> BOOL {
    let mut pid: u32 = 0;
    if GetWindowThreadProcessId(window, Some(&mut pid)) == 0 {
        return TRUE;
    }

    const BUF_SIZE: usize = 512;
    let mut buff: [u16; BUF_SIZE] = [0; BUF_SIZE];

    let writ_chars = GetWindowTextW(window, &mut buff);
    if writ_chars == 0 {
        return TRUE;
    }

    let f = &mut *(param.0 as *mut T);
    f(pid, String::from_utf16_lossy(&buff[0..writ_chars as usize]));

    TRUE
}

pub fn enum_window<T: FnMut(u32, String)>(mut f: T) {
    let _ = unsafe {
        EnumWindows(
            Some(callback_enum_windows::<T>),
            LPARAM(&mut f as *mut _ as isize),
        )
    };
}
