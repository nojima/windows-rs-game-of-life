use bindings::Windows::Win32::Graphics::Gdi::*;
use crate::wrapper::window::Window;

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
