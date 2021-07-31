use bindings::Windows::Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*};
use crate::wstr;

pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn show(&self, cmd: SHOW_WINDOW_CMD) {
        unsafe { ShowWindow(self.hwnd, cmd) };
    }

    pub fn invalidate(&self, erase: bool) -> anyhow::Result<()> {
        unsafe { InvalidateRect(self.hwnd, std::ptr::null(), erase) }.ok().map_err(|e| e.into())
    }

    pub fn def_window_proc(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { DefWindowProcW(self.hwnd, message, wparam, lparam) }
    }
}

pub struct Application {
    wndproc: Box<dyn FnMut(&Window, u32, WPARAM, LPARAM) -> LRESULT>,
}

impl Application {
    pub fn create(
        title: &str,
        size: SIZE,
        wndproc: impl FnMut(&Window, u32, WPARAM, LPARAM) -> LRESULT + 'static,
    ) -> anyhow::Result<(Box<Application>, Window)> {

        let instance = unsafe { GetModuleHandleW(None) };
        anyhow::ensure!(!instance.is_null(), "GetModuleHandleW failed");

        let mut class_name: wstr::WSTR = "GameOfLife".into();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
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

        let mut app = Box::new(Application {
            wndproc: Box::new(wndproc),
        });

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
                app.as_mut() as *mut Application as _,
            )
        };
        anyhow::ensure!(!hwnd.is_null(), "CreateWindowExW failed");

        let window = Window { hwnd };

        Ok((app, window))
    }

}

extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_CREATE {
        let create_struct: &CREATESTRUCTW = unsafe { std::mem::transmute(lparam) };
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _) };
    }

    let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Application };
    if let Some(app) = unsafe { user_data.as_mut() } {
        let window = Window { hwnd };
        (app.wndproc)(&window, message, wparam, lparam)
    } else {
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
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

pub trait GdiObject {
    fn gdi_object_handle(&self) -> HGDIOBJ;
}

pub struct DeviceContext<'a> {
    window: &'a Window,
    hdc: HDC,
    paint_struct: PAINTSTRUCT,
}

impl<'a> DeviceContext<'a> {
    pub fn begin_paint(window: &Window) -> anyhow::Result<DeviceContext> {
        let hwnd = window.hwnd();
        let mut paint_struct: PAINTSTRUCT = Default::default();
        let hdc = unsafe { BeginPaint(hwnd, &mut paint_struct) };
        anyhow::ensure!(!hdc.is_null(), "BeginPaint failed");
        Ok(DeviceContext { window, hdc, paint_struct })
    }

    pub fn select_object(&self, obj: &dyn GdiObject) -> anyhow::Result<()> {
        let prev = unsafe { SelectObject(self.hdc, obj.gdi_object_handle()) };
        anyhow::ensure!(!prev.is_null(), "SelectObject failed");
        Ok(())
    }

    pub fn move_to(&self, x: i32, y: i32) -> anyhow::Result<()> {
        unsafe { MoveToEx(self.hdc, x, y, std::ptr::null_mut()) }.ok().map_err(|e| e.into())
    }

    pub fn line_to(&self, x: i32, y: i32) -> anyhow::Result<()> {
        unsafe { LineTo(self.hdc, x, y) }.ok().map_err(|e| e.into())
    }

    pub fn rectangle(&self, left: i32, top: i32, right: i32, bottom: i32) -> anyhow::Result<()> {
        unsafe { Rectangle(self.hdc, left, top, right, bottom)}.ok().map_err(|e| e.into())
    }
}

impl<'a> Drop for DeviceContext<'a> {
    fn drop(&mut self) {
        let hwnd = self.window.hwnd();
        unsafe { EndPaint(hwnd, &self.paint_struct) };
    }
}

pub struct Pen {
    handle: HPEN,
}

impl Pen {
    pub fn create(istyle: PEN_STYLE, cwidth: i32, color: u32) -> anyhow::Result<Pen> {
        let handle = unsafe { CreatePen(istyle, cwidth, color) };
        anyhow::ensure!(!handle.is_null(), "CreatePan failed");
        Ok(Pen { handle })
    }
}

impl Drop for Pen {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.handle) };
    }
}

impl GdiObject for Pen {
    fn gdi_object_handle(&self) -> HGDIOBJ {
        HGDIOBJ(self.handle.0)
    }
}

pub struct StockPen {
    handle: HPEN,
}

impl StockPen {
    pub fn null() -> StockPen {
        let handle = unsafe { GetStockObject(NULL_PEN) };
        StockPen { handle: HPEN(handle.0) }
    }
}

impl GdiObject for StockPen {
    fn gdi_object_handle(&self) -> HGDIOBJ {
        HGDIOBJ(self.handle.0)
    }
}

pub struct SolidBrush {
    handle: HBRUSH,
}

impl SolidBrush {
    pub fn create(color: u32) -> anyhow::Result<SolidBrush> {
        let handle = unsafe { CreateSolidBrush(color) };
        anyhow::ensure!(!handle.is_null(), "CreateSolidBrush failed");
        Ok(SolidBrush { handle })
    }
}

impl Drop for SolidBrush {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.handle) };
    }
}

impl GdiObject for SolidBrush {
    fn gdi_object_handle(&self) -> HGDIOBJ {
        HGDIOBJ(self.handle.0)
    }
}

pub struct StockBrush {
    handle: HBRUSH,
}

impl StockBrush {
    pub fn null() -> StockBrush {
        let handle = unsafe { GetStockObject(NULL_BRUSH) };
        StockBrush { handle: HBRUSH(handle.0) }
    }
}

impl GdiObject for StockBrush {
    fn gdi_object_handle(&self) -> HGDIOBJ {
        HGDIOBJ(self.handle.0)
    }
}

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
        unsafe { KillTimer(window.hwnd, self.id) };
    }
}
