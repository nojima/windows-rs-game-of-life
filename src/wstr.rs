use bindings::Windows::Win32::Foundation::PWSTR;
use core::fmt;
use std::char;
use std::convert;

pub struct WSTR(Vec<u16>);

impl convert::From<&'_ str> for WSTR {
    fn from(s: &'_ str) -> Self {
        let wsz: Vec<_> = s.encode_utf16().chain(Some(0).into_iter()).collect();
        WSTR(wsz)
    }
}

impl fmt::Display for WSTR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wsz = &self.0;
        let ws = &wsz[0..wsz.len() - 1];
        let s: String = char::decode_utf16(ws.iter().cloned())
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        s.fmt(f)
    }
}

impl WSTR {
    pub fn len(&self) -> i32 {
        (self.0.len() - 1) as i32
    }

    pub fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.0.as_mut_ptr())
    }
}

#[test]
fn test_wstr() {
    let original = "I ❤ Rust and 𠮷野家";
    let ws: WSTR = original.into();
    let s = ws.to_string();
    assert_eq!(original, s);
}
