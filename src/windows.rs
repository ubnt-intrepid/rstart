use winapi;
use kernel32;

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::ptr::null_mut;
use winapi::DWORD;


pub fn get_error_message() -> String {
  let mut buf = vec![0u8; 1024];

  unsafe {
    kernel32::FormatMessageA(winapi::FORMAT_MESSAGE_ALLOCATE_BUFFER |
                             winapi::FORMAT_MESSAGE_FROM_SYSTEM,
                             null_mut(),
                             kernel32::GetLastError(),
                             winapi::LANG_USER_DEFAULT as DWORD,
                             buf.as_mut_ptr() as winapi::LPSTR,
                             0,
                             null_mut())
  };

  unsafe { CStr::from_ptr(transmute(buf.as_ptr())).to_string_lossy().into_owned() }
}



pub fn expand_env(s: &str) -> Option<String> {
  let mut dst = vec![0u8; 1024];
  let src = CString::new(s).unwrap();
  let nchars = unsafe {
    kernel32::ExpandEnvironmentStringsA(src.as_ptr(), transmute(dst.as_mut_ptr()), dst.len() as u32)
  };
  if nchars == 0 || nchars > 1024 {
    return None;
  }

  Some(unsafe { CStr::from_ptr(transmute(dst.as_ptr())).to_string_lossy().into_owned() })
}


#[test]
fn test_expand_environment_strings() {
  use std::path::Path;
  assert_eq!(Path::new(&expand_env("%ComSpec%").unwrap()).canonicalize().unwrap(),
             Path::new(r"C:\Windows\System32\cmd.exe").canonicalize().unwrap());
}
