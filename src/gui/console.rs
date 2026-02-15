use winapi::um::wincon::GetConsoleWindow;
use winapi::um::winuser::{IsWindowVisible, ShowWindow, SW_HIDE, SW_SHOW};

pub fn is_console_visible() -> bool {
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_null() {
            return false;
        }
        IsWindowVisible(hwnd) != 0
    }
}

pub fn set_console_visible(visible: bool) {
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_null() {
            return;
        }
        ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE });
    }
}
