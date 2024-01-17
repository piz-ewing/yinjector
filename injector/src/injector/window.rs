use windows_win::{
    raw::window::{enum_by, get_text, get_thread_process_id},
    sys::HWND,
};

pub struct Window {
    pub title: String,
    pub name: String,
    pub pid: u32,
}

pub fn query_all_window<T: FnMut(u32, String)>(mut f: T) {
    let _ = enum_by(None, |handle: HWND| {
        let process_id = get_thread_process_id(handle);
        if process_id.0 == 0 {
            return;
        }

        let window_title = get_text(handle);
        if window_title.is_err() {
            return;
        }

        f(process_id.0, window_title.unwrap());
    });
}
