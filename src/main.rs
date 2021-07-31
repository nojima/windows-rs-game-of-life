mod game_of_life;
mod wrapper;
mod wstr;

use bindings::Windows::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};
use wrapper::{application::Application, gdi::{DeviceContext, Pen, SolidBrush, StockBrush, StockPen}, timer::Timer, window::Window, main_loop, post_quit_message};
use game_of_life::{GameOfLife, Cell};

const CELL_PIXEL_SIZE: i32 = 10;
const MARGIN: RECT = RECT { left: 10, top: 10, right: 10, bottom: 10 };
const FPS: u32 = 15;

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

fn paint(window: &Window, game_of_life: &GameOfLife) -> anyhow::Result<()> {
    let dc = DeviceContext::begin_paint(window)?;

    let (width, height) = game_of_life.size();

    let gray_pen = Pen::create(PS_SOLID, 1, rgb(200, 200, 200))?;
    let black_brush = SolidBrush::create(rgb(64, 64, 64))?;
    let white_brush = SolidBrush::create(rgb(255, 255, 255))?;

    // draw vertical lines
    dc.select_object(&gray_pen)?;
    for col in 0..width+1 {
        let x  = MARGIN.left + CELL_PIXEL_SIZE * (col as i32);
        let y1 = MARGIN.top;
        let y2 = MARGIN.top + CELL_PIXEL_SIZE * (height as i32);
        dc.move_to(x, y1)?;
        dc.line_to(x, y2)?;
    }

    // draw horizontal lines
    dc.select_object(&gray_pen)?;
    for row in 0..height+1 {
        let y  = MARGIN.top + CELL_PIXEL_SIZE * (row as i32);
        let x1 = MARGIN.left;
        let x2 = MARGIN.left + CELL_PIXEL_SIZE * (width as i32);
        dc.move_to(x1, y)?;
        dc.line_to(x2, y)?;
    }

    // draw cells
    dc.select_object(&StockPen::null())?;
    for row in 0..height {
        for col in 0..width {
            let brush = match game_of_life.get_cell(row, col) {
                Cell::Alive => &black_brush,
                Cell::Dead => &white_brush,
            };
            dc.select_object(brush)?;
            let left   = MARGIN.left + CELL_PIXEL_SIZE * (col as i32) + 1;
            let top    = MARGIN.top  + CELL_PIXEL_SIZE * (row as i32) + 1;
            let right  = left + CELL_PIXEL_SIZE;
            let bottom = top  + CELL_PIXEL_SIZE;
            dc.rectangle(left, top, right, bottom)?;
        }
    }

    dc.select_object(&StockBrush::null())?;
    Ok(())
}

fn tick(window: &Window, game_of_life: &mut GameOfLife) -> anyhow::Result<()> {
    game_of_life.tick();
    window.invalidate(false)
}

fn calculate_window_size(game_of_life: &GameOfLife) -> SIZE {
    let (width, height) = game_of_life.size();
    let window_width  = MARGIN.left + CELL_PIXEL_SIZE * (width  as i32) + MARGIN.right;
    let window_height = MARGIN.top  + CELL_PIXEL_SIZE * (height as i32) + MARGIN.top;
    SIZE { cx: window_width, cy: window_height }
}

fn main() -> anyhow::Result<()> {
    let mut game_of_life = GameOfLife::new(100, 100)?;
    let window_size = calculate_window_size(&game_of_life);
    let mut timer = None;

    let (_app, window) = Application::create(
        "Game of Life",
        window_size,
        move |window, message, wparam, lparam| {
            match message {
                WM_CREATE => {
                    timer = Some(Timer::set(window, 1, 1000 / FPS).unwrap());
                    LRESULT::default()
                }
                WM_PAINT => {
                    paint(window, &game_of_life).unwrap();
                    LRESULT::default()
                }
                WM_TIMER => {
                    tick(window, &mut game_of_life).unwrap();
                    LRESULT::default()
                }
                WM_DESTROY => {
                    timer.iter().for_each(|t| t.kill(window));
                    post_quit_message(0);
                    LRESULT::default()
                }
                _ => window.def_window_proc(message, wparam, lparam)
            }
        })?;

    window.show(SW_SHOW);
    main_loop()
}
