use bindings::Windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
use crate::wrapper::window::Window;

pub struct Application {
    wndproc: Box<dyn FnMut(&Window, u32, WPARAM, LPARAM) -> LRESULT>,
}

impl Application {
    pub fn create(
        title: &str,
        size: SIZE,
        wndproc: impl FnMut(&Window, u32, WPARAM, LPARAM) -> LRESULT + 'static,
    ) -> anyhow::Result<(Box<Application>, Window)> {
        let mut app = Box::new(Application {
            wndproc: Box::new(wndproc),
        });
        let window = Window::create(
            title, size, trampoline, app.as_mut() as *mut Application as _
        )?;
        Ok((app, window))
    }
}

extern "system" fn trampoline(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_CREATE {
        let create_struct: &CREATESTRUCTW = unsafe { std::mem::transmute(lparam) };
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _) };
    }

    let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Application };
    if let Some(app) = unsafe { user_data.as_mut() } {
        let window = Window::from_handle(hwnd);
        (app.wndproc)(&window, message, wparam, lparam)
    } else {
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}
