use std::mem;
use std::ops::{Deref, DerefMut};
use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
use winapi::shared::windef::{HDC, HMONITOR, LPRECT, RECT};
use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO, MONITORINFOF_PRIMARY};

#[derive(Clone)]
pub struct Monitors(Vec<MonitorInfo>);

impl Monitors {
    pub fn get_primary(&self) -> Option<MonitorInfo> {
        self.iter().find(|m| m.is_primary).map(|m| m.clone())
    }
}

impl Deref for Monitors {
    type Target = Vec<MonitorInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Monitors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct MonitorInfo {
    pub rect: RECT,
    pub work_rect: RECT,
    pub is_primary: bool,
}

impl MonitorInfo {
    pub fn width(&self) -> i32 {
        self.rect.right - self.rect.left
    }

    pub fn height(&self) -> i32 {
        self.rect.bottom - self.rect.top
    }
}

pub fn get_all_monitors() -> Monitors {
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _hdc: HDC,
        _rect: LPRECT,
        data: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(data as *mut Vec<MonitorInfo>);
        let mut info: MONITORINFO = mem::zeroed();
        info.cbSize = mem::size_of::<MONITORINFO>() as u32;

        if GetMonitorInfoW(monitor, &mut info) != 0 {
            monitors.push(MonitorInfo {
                rect: info.rcMonitor,
                work_rect: info.rcWork,
                is_primary: info.dwFlags & MONITORINFOF_PRIMARY != 0,
            });
        }
        TRUE
    }

    let mut monitors: Vec<MonitorInfo> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(callback),
            &mut monitors as *mut _ as LPARAM,
        );
    }
    Monitors(monitors)
}