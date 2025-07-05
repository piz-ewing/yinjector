use windows::{
    Win32::{
        Foundation::{BOOL, HMODULE, TRUE},
        UI::WindowsAndMessaging::{MESSAGEBOX_STYLE, MessageBoxA},
    },
    core::PCSTR,
};

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    _h_module: HMODULE,
    ul_reason_for_call: u32,
    _lp_reserved: *mut core::ffi::c_void,
) -> BOOL {
    match ul_reason_for_call {
        1 => {
            // DLL_PROCESS_ATTACH
            unsafe {
                MessageBoxA(None, PCSTR::null(), PCSTR::null(), MESSAGEBOX_STYLE(0));
            }
        }
        2 => {} // DLL_THREAD_ATTACH
        3 => {} // DLL_THREAD_DETACH
        4 => {} // DLL_PROCESS_DETACH
        _ => {}
    }
    TRUE
}
