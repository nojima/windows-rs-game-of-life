mod wstr;

use bindings::Windows::Win32::{
    Foundation::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
};
use std::mem;

struct App {}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_CREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { mem::transmute(lparam) };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _) };
            LRESULT::default()
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn create_window(app: &mut App) -> anyhow::Result<HWND> {
    let instance = unsafe { GetModuleHandleW(None) };
    anyhow::ensure!(!instance.is_null(), "GetModuleHandleW failed");

    let mut class_name: wstr::WSTR = "GameOfLife".into();

    let wc = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
        lpszClassName: class_name.as_pwstr(),
        ..Default::default()
    };
    let atom = unsafe { RegisterClassExW(&wc) };
    anyhow::ensure!(atom != 0, "RegisterClassExW failed");

    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            class_name.as_pwstr(),
            "Game of Life",
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            640,
            480,
            None,
            None,
            instance,
            app as *mut _ as _,
        )
    };
    anyhow::ensure!(!hwnd.is_null(), "CreateWindowExW failed");

    Ok(hwnd)
}

fn main_loop() -> anyhow::Result<()> {
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

fn main() -> anyhow::Result<()> {
    let mut app = App {};
    let hwnd = create_window(&mut app)?;
    unsafe { ShowWindow(hwnd, SW_SHOW) };
    main_loop()
}
