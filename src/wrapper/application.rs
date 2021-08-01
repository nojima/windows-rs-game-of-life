use crate::wrapper::window::Window;
use bindings::Windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub fn run<
    WndProc: FnMut(&Window, u32, WPARAM, LPARAM) -> LRESULT + 'static,
    Action: FnOnce(&Window) -> WndProc + 'static,
>(
    title: &str,
    size: SIZE,
    action: Action,
) -> anyhow::Result<()> {
    Window::create(
        title,
        size,
        trampoline::<WndProc, Action>,
        Box::into_raw(Box::new(action)) as _,
    )
}

extern "system" fn trampoline<
    WndProc: FnMut(&Window, u32, WPARAM, LPARAM) -> LRESULT + 'static,
    Action: FnOnce(&Window) -> WndProc + 'static,
>(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { std::mem::transmute(lparam) };
            let create_param = create_struct.lpCreateParams as *mut Action;
            let action = unsafe { Box::from_raw(create_param) };
            let window = Window::from_handle(hwnd);
            let wndproc = Box::new(action(&window));
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(wndproc) as _) };
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_NCDESTROY => {
            let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WndProc };
            let _wndproc = unsafe { Box::from_raw(user_data) };
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        _ => {
            let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WndProc };
            if let Some(wndproc) = unsafe { user_data.as_mut() } {
                let window = Window::from_handle(hwnd);
                wndproc(&window, message, wparam, lparam)
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
    }
}

pub fn main_loop() -> anyhow::Result<()> {
    loop {
        let mut message = MSG::default();
        let ret = unsafe { GetMessageW(&mut message, None, 0, 0) }.0;
        if ret == -1 {
            anyhow::bail!("GetMessageW failed");
        }
        if ret == 0 {
            return Ok(());
        }
        unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }
}

pub fn post_quit_message(exit_code: i32) {
    unsafe { PostQuitMessage(exit_code) };
}
