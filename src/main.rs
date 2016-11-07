extern crate winapi;
extern crate advapi32;
extern crate kernel32;

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::ptr::null_mut;

use winapi::DWORD;
use winapi::winnt;
use winapi::minwindef::HKEY;
use winapi::winerror::ERROR_SUCCESS;


fn expand_environment_strings(s: &str) -> String {
  let mut dst = vec![0u8; 1024];
  let src = CString::new(s).unwrap();
  let nchars = unsafe {
    kernel32::ExpandEnvironmentStringsA(src.as_ptr(), transmute(dst.as_mut_ptr()), dst.len() as u32)
  };
  if nchars == 0 || nchars > 1024 {
    return s.to_owned();
  }

  unsafe { CStr::from_ptr(transmute(dst.as_ptr())).to_string_lossy().into_owned() }
}

#[test]
fn test_expand_environment_strings() {
  use std::path::Path;
  assert_eq!(Path::new(&expand_environment_strings("%ComSpec%")).canonicalize().unwrap(),
             Path::new("C:\\Windows\\System32\\cmd.exe").canonicalize().unwrap());
}


struct Key(HKEY);

impl Drop for Key {
  fn drop(&mut self) {
    unsafe { advapi32::RegCloseKey(self.0) };
    self.0 = null_mut();
  }
}

#[derive(Debug)]
struct Value {
  var_type: DWORD,
  var_data: Vec<u8>,
}

impl Value {
  fn type_str(&self) -> &'static str {
    match self.var_type {
      winapi::winnt::REG_NONE => "REG_NONE",
      winapi::winnt::REG_SZ => "REG_SZ",
      winapi::winnt::REG_EXPAND_SZ => "REG_EXPAND_SZ",
      winapi::winnt::REG_BINARY => "REG_BINARY",
      winapi::winnt::REG_DWORD_LITTLE_ENDIAN => "REG_DWORD_LITTLE_ENDIAN",
      winapi::winnt::REG_DWORD_BIG_ENDIAN => "REG_DWORD_BIG_ENDIAN",
      winapi::winnt::REG_LINK => "REG_LINK",
      winapi::winnt::REG_MULTI_SZ => "REG_MULTI_SZ",
      winapi::winnt::REG_RESOURCE_LIST => "REG_RESOURCE_LIST",
      winapi::winnt::REG_FULL_RESOURCE_DESCRIPTOR => "REG_FULL_RESOURCE_DESCRIPTOR",
      winapi::winnt::REG_RESOURCE_REQUIREMENTS_LIST => "REG_RESOURCE_REQUIREMENTS_LIST",
      winapi::winnt::REG_QWORD => "REG_QWORD",
      _ => "Unknown",
    }
  }
}

fn open_key(hkey: HKEY, path: &str) -> Option<Key> {
  let mut key = null_mut();

  let path = CString::new(path).unwrap();
  let ret =
    unsafe { advapi32::RegOpenKeyExA(hkey, path.as_ptr(), 0, winnt::KEY_QUERY_VALUE, &mut key) };
  if ret != (unsafe { transmute(ERROR_SUCCESS) }) {
    return None;
  }

  Some(Key(key))
}

fn query_value(hkey: HKEY, name: &str) -> Option<Value> {
  let name = CString::new(name).unwrap();
  let mut var_type: DWORD = 0;
  let mut data = vec![0u8; 8196];
  let mut data_size: DWORD = data.len() as DWORD;
  let ret = unsafe {
    advapi32::RegQueryValueExA(hkey,
                               name.as_ptr(),
                               null_mut(),
                               &mut var_type,
                               data.as_mut_ptr(),
                               &mut data_size)
  };
  if ret != (unsafe { transmute(ERROR_SUCCESS) }) {
    return None;
  }

  data.resize(data_size as usize, 0u8);

  Some(Value {
    var_type: var_type,
    var_data: data,
  })
}


fn main() {
  println!("{:?}", expand_environment_strings("%APPDATA%"));

  let key = open_key(winapi::HKEY_LOCAL_MACHINE,
                     "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment");
  if let Some(key) = key {
    if let Some(value) = query_value(key.0, "Path") {
      let s = unsafe {
        CStr::from_ptr(transmute(value.var_data.as_ptr())).to_string_lossy().into_owned()
      };
      println!("{}, {:?}", value.type_str(), s);
    }
    if let Some(value) = query_value(key.0, "OS") {
      let s = unsafe {
        CStr::from_ptr(transmute(value.var_data.as_ptr())).to_string_lossy().into_owned()
      };
      println!("{}, {:?}", value.type_str(), s);
    }
  }

  let key = open_key(winapi::HKEY_CURRENT_USER, "Environment");
  if let Some(key) = key {
    if let Some(value) = query_value(key.0, "Path") {
      let s = unsafe {
        CStr::from_ptr(transmute(value.var_data.as_ptr())).to_string_lossy().into_owned()
      };
      println!("{}, {:?}", value.type_str(), s);
    }
  }
}
