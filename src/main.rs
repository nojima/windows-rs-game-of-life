mod game_of_life;
mod wstr;

use bindings::Windows::Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*};
use std::{mem, ptr};
use game_of_life::{GameOfLife, Cell};

struct App {
    game_of_life: GameOfLife,
}

const CELL_PIXEL_SIZE: i32 = 10;
const MARGIN: RECT = RECT { left: 10, top: 10, right: 10, bottom: 10 };
const FPS: u32 = 15;

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

fn paint(hwnd: HWND, app: &App) {
    let mut ps: PAINTSTRUCT = Default::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

    let game_of_life = &app.game_of_life;
    let (width, height) = game_of_life.size();

    let gray_pen = unsafe { CreatePen(PS_SOLID, 1, rgb(200, 200, 200)) };
    let black_brush = unsafe { CreateSolidBrush(rgb(64, 64, 64)) };
    let white_brush = unsafe { CreateSolidBrush(rgb(255, 255, 255)) };

    // draw vertical lines
    unsafe { SelectObject(hdc, gray_pen) };
    for col in 0..width+1 {
        let x  = MARGIN.left + CELL_PIXEL_SIZE * (col as i32);
        let y1 = MARGIN.top;
        let y2 = MARGIN.top + CELL_PIXEL_SIZE * (height as i32);
        unsafe { MoveToEx(hdc, x, y1, ptr::null_mut()) };
        unsafe { LineTo(hdc, x, y2) };
    }

    // draw horizontal lines
    unsafe { SelectObject(hdc, gray_pen) };
    for row in 0..height+1 {
        let y  = MARGIN.top + CELL_PIXEL_SIZE * (row as i32);
        let x1 = MARGIN.left;
        let x2 = MARGIN.left + CELL_PIXEL_SIZE * (width as i32);
        unsafe { MoveToEx(hdc, x1, y, ptr::null_mut()) };
        unsafe { LineTo(hdc, x2, y) };
    }

    // draw cells
    unsafe { SelectObject(hdc, GetStockObject(NULL_PEN)) };
    for row in 0..height {
        for col in 0..width {
            let brush = match game_of_life.get_cell(row, col) {
                Cell::Alive => black_brush,
                Cell::Dead => white_brush,
            };
            unsafe { SelectObject(hdc, brush) };
            let left   = MARGIN.left + CELL_PIXEL_SIZE * (col as i32) + 1;
            let top    = MARGIN.top  + CELL_PIXEL_SIZE * (row as i32) + 1;
            let right  = left + CELL_PIXEL_SIZE;
            let bottom = top  + CELL_PIXEL_SIZE;
            unsafe { Rectangle(hdc, left, top, right, bottom) };
        }
    }

    unsafe { SelectObject(hdc, GetStockObject(NULL_BRUSH)) };
    unsafe { DeleteObject(white_brush) };
    unsafe { DeleteObject(black_brush) };
    unsafe { DeleteObject(gray_pen) };
    unsafe { EndPaint(hwnd, &ps) };
}

fn tick(hwnd: HWND, app: &mut App) {
    app.game_of_life.tick();
    unsafe { InvalidateRect(hwnd, ptr::null(), false) };
}

unsafe fn get_app_from_window<'a>(hwnd: HWND) -> Option<&'a mut App> {
    let user_data = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
    user_data.as_mut()
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_CREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { mem::transmute(lparam) };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _) };
            unsafe { SetTimer(hwnd, 1, 1000 / FPS, None) };
            LRESULT::default()
        }
        WM_PAINT => {
            if let Some(app) = unsafe { get_app_from_window(hwnd) } {
                paint(hwnd, app);
            }
            LRESULT::default()
        }
        WM_TIMER => {
            if let Some(app) = unsafe { get_app_from_window(hwnd) } {
                tick(hwnd, app);
            }
            LRESULT::default()
        }
        WM_DESTROY => {
            unsafe { KillTimer(hwnd, 1) };
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn create_window(app: &mut App, size: SIZE) -> anyhow::Result<HWND> {
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
            "Game of Life",
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
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

fn calculate_window_size(app: &App) -> SIZE {
    let (width, height) = app.game_of_life.size();
    let window_width  = MARGIN.left + CELL_PIXEL_SIZE * (width  as i32) + MARGIN.right;
    let window_height = MARGIN.top  + CELL_PIXEL_SIZE * (height as i32) + MARGIN.top;
    SIZE { cx: window_width, cy: window_height }
}

fn main() -> anyhow::Result<()> {
    let mut app = App {
        game_of_life: GameOfLife::new(100, 100)?
    };
    let window_size = calculate_window_size(&app);
    let hwnd = create_window(&mut app, window_size)?;
    unsafe { ShowWindow(hwnd, SW_SHOW) };
    main_loop()
}
