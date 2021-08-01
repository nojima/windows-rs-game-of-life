pub mod window;
pub mod application;
pub mod gdi;
pub mod timer;

use bindings::Windows::Win32::UI::WindowsAndMessaging::*;

pub fn post_quit_message(exit_code: i32) {
    unsafe { PostQuitMessage(exit_code) };
}
