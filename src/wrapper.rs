pub mod window;
pub mod application;
pub mod gdi;
pub mod timer;

use bindings::Windows::Win32::UI::WindowsAndMessaging::*;

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
