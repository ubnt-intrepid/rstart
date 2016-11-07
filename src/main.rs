extern crate winapi;
extern crate advapi32;
extern crate kernel32;

use std::ffi::{CStr, CString};
use std::mem::transmute;

fn expand_environment_strings(s: &str) -> String {
    let mut dst = vec![0u8; 1024];
    let src = CString::new(s).unwrap();
    let nchars = unsafe {
        kernel32::ExpandEnvironmentStringsA(src.as_ptr(),
                                            transmute(dst.as_mut_ptr()),
                                            dst.len() as u32)
    };
    if nchars == 0 || nchars > 1024 {
        return s.to_owned();
    }

    unsafe { CStr::from_ptr(transmute(dst.as_ptr())).to_string_lossy().into_owned() }
}

fn main() {
    println!("{:?}", expand_environment_strings("%APPDATA%"));
}
