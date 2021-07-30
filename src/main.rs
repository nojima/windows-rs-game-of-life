mod game_of_life;
mod wstr;

use bindings::Windows::Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*};
use std::{mem, ptr};
use game_of_life::{GameOfLife, Cell};

struct App {
    game_of_life: GameOfLife,
}

const CELL_PIXEL_SIZE: usize = 10;

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

fn paint(hwnd: HWND, app: &App) {
    let mut ps: PAINTSTRUCT = Default::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

    let game_of_life = &app.game_of_life;
    let (width, height) = game_of_life.size();

    let (mergin_top, mergin_left) = (10, 10);

    let gray_pen = unsafe { CreatePen(PS_SOLID, 1, rgb(200, 200, 200)) };
    let black_brush = unsafe { CreateSolidBrush(rgb(64, 64, 64)) };

    // draw vertical lines
    unsafe { SelectObject(hdc, gray_pen) };
    for col in 0..width+1 {
        let x = mergin_left + CELL_PIXEL_SIZE * col;
        let y1 = mergin_top;
        let y2 = mergin_top + CELL_PIXEL_SIZE * height;
        unsafe { MoveToEx(hdc, x as i32, y1 as i32, ptr::null_mut()) };
        unsafe { LineTo(hdc, x as i32, y2 as i32) };
    }

    // draw horizontal lines
    unsafe { SelectObject(hdc, gray_pen) };
    for row in 0..height+1 {
        let y = mergin_top + CELL_PIXEL_SIZE * row;
        let x1 = mergin_left;
        let x2 = mergin_left + CELL_PIXEL_SIZE * width;
        unsafe { MoveToEx(hdc, x1 as i32, y as i32, ptr::null_mut()) };
        unsafe { LineTo(hdc, x2 as i32, y as i32) };
    }

    // draw cells
    unsafe { SelectObject(hdc, black_brush) };
    unsafe { SelectObject(hdc, GetStockObject(NULL_PEN)) };
    for row in 0..height {
        for col in 0..width {
            if game_of_life.get_cell(row, col) == Cell::Alive {
                let left = mergin_left + CELL_PIXEL_SIZE * col;
                let top = mergin_top + CELL_PIXEL_SIZE * row;
                let right = left + CELL_PIXEL_SIZE;
                let bottom = top + CELL_PIXEL_SIZE;
                unsafe { Rectangle(hdc, left as i32, top as i32, right as i32, bottom as i32) };
            }
        }
    }

    unsafe { DeleteObject(black_brush) };
    unsafe { DeleteObject(gray_pen) };
    unsafe { EndPaint(hwnd, &ps) };
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_CREATE => {
            let create_struct: &CREATESTRUCTW = unsafe { mem::transmute(lparam) };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _) };
            LRESULT::default()
        }
        WM_PAINT => {
            let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const App };
            if let Some(app) = unsafe { user_data.as_ref() } {
                paint(hwnd, app);
            }
            LRESULT::default()
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn create_window(app: &mut App, width: i32, height: i32) -> anyhow::Result<HWND> {
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
        right: width,
        bottom: height,
    };
    unsafe { AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, false) };

    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
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

fn main() -> anyhow::Result<()> {
    let mut app = App {
        game_of_life: GameOfLife::new(100, 100)?
    };

    let hwnd = create_window(&mut app, 1020, 1020)?;
    unsafe { ShowWindow(hwnd, SW_SHOW) };
    main_loop()
}
