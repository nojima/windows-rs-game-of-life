use bindings::Windows::Win32::UI::WindowsAndMessaging::*;
use crate::wrapper::window::Window;

pub struct Timer {
    id: usize,
}

impl Timer {
    pub fn set(window: &Window, id: usize, interval_millis: u32) -> anyhow::Result<Timer> {
        let ret = unsafe { SetTimer(window.hwnd(), id, interval_millis, None) };
        anyhow::ensure!(ret != 0, "SetTimer failed");
        Ok(Timer { id })
    }

    pub fn kill(&self, window: &Window) {
        unsafe { KillTimer(window.hwnd(), self.id) };
    }
}
