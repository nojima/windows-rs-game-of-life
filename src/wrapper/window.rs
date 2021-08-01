use std::ffi::c_void;

use crate::wrapper::wstr;
use bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};

pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn create(
        title: &str,
        size: SIZE,
        wndproc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
        user_data: *mut c_void,
    ) -> anyhow::Result<()> {
        let instance = unsafe { GetModuleHandleW(None) };
        anyhow::ensure!(!instance.is_null(), "GetModuleHandleW failed");

        let mut class_name: wstr::WSTR = "GameOfLife".into();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
            lpszClassName: class_name.as_pwstr(),
            hbrBackground: HBRUSH(unsafe { GetStockObject(WHITE_BRUSH) }.0),
            ..Default::default()
        };
        let atom = unsafe { RegisterClassExW(&wc) };
        anyhow::ensure!(atom != 0, "RegisterClassExW failed");

        let mut window_rect = RECT {
            left: 0,
            top: 0,
            right: size.cx,
            bottom: size.cy,
        };
        unsafe { AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, false) };

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_COMPOSITED,
                class_name.as_pwstr(),
                title,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window_rect.right - window_rect.left,
                window_rect.bottom - window_rect.top,
                None,
                None,
                instance,
                user_data,
            )
        };
        anyhow::ensure!(!hwnd.is_null(), "CreateWindowExW failed");

        Ok(())
    }

    pub fn from_handle(hwnd: HWND) -> Window {
        Window { hwnd }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn show(&self, cmd: SHOW_WINDOW_CMD) {
        unsafe { ShowWindow(self.hwnd, cmd) };
    }

    pub fn invalidate(&self, erase: bool) -> anyhow::Result<()> {
        unsafe { InvalidateRect(self.hwnd, std::ptr::null(), erase) }
            .ok()
            .map_err(|e| e.into())
    }

    pub fn def_window_proc(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { DefWindowProcW(self.hwnd, message, wparam, lparam) }
    }
}
