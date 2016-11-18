use winapi;
use advapi32;
use kernel32;

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::ptr::null_mut;

use winapi::DWORD;
use winapi::winnt;
use winapi::minwindef::HKEY;
use winapi::winerror::ERROR_SUCCESS;


fn get_error_message() -> String {
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


#[derive(Debug)]
pub enum RootKey {
  LocalMachine,
  CurrentUser,
}

impl Into<HKEY> for RootKey {
  fn into(self) -> HKEY {
    match self {
      RootKey::LocalMachine => winapi::HKEY_LOCAL_MACHINE,
      RootKey::CurrentUser => winapi::HKEY_CURRENT_USER,
    }
  }
}


#[derive(Debug)]
pub struct Value {
  var_type: DWORD,
  var_data: Vec<u8>,
}

impl Value {
  pub fn type_str(&self) -> &'static str {
    match self.var_type {
      winnt::REG_NONE => "REG_NONE",
      winnt::REG_SZ => "REG_SZ",
      winnt::REG_EXPAND_SZ => "REG_EXPAND_SZ",
      winnt::REG_BINARY => "REG_BINARY",
      winnt::REG_DWORD_LITTLE_ENDIAN => "REG_DWORD_LITTLE_ENDIAN",
      winnt::REG_DWORD_BIG_ENDIAN => "REG_DWORD_BIG_ENDIAN",
      winnt::REG_LINK => "REG_LINK",
      winnt::REG_MULTI_SZ => "REG_MULTI_SZ",
      winnt::REG_RESOURCE_LIST => "REG_RESOURCE_LIST",
      winnt::REG_FULL_RESOURCE_DESCRIPTOR => "REG_FULL_RESOURCE_DESCRIPTOR",
      winnt::REG_RESOURCE_REQUIREMENTS_LIST => "REG_RESOURCE_REQUIREMENTS_LIST",
      winnt::REG_QWORD => "REG_QWORD",
      _ => "Unknown",
    }
  }

  pub fn to_string(&self) -> Option<String> {
    match self.type_str() {
      "REG_SZ" | "REG_EXPAND_SZ" => {
        Some(unsafe {
          CStr::from_ptr(transmute(self.var_data.as_ptr())).to_string_lossy().into_owned()
        })
      }
      _ => None,
    }
  }
}


pub struct Key(HKEY);

impl Key {
  pub fn open(root: RootKey, path: &str) -> Result<Key, String> {
    let mut key = null_mut();

    let path = CString::new(path).unwrap();
    let ret = unsafe {
      advapi32::RegOpenKeyExA(root.into(),
                              path.as_ptr(),
                              0,
                              winnt::KEY_QUERY_VALUE,
                              &mut key)
    };
    if ret != (unsafe { transmute(ERROR_SUCCESS) }) {
      return Err(get_error_message());
    }

    Ok(Key(key))
  }

  pub fn query_value(&self, name: &str) -> Result<Value, String> {
    let name = CString::new(name).unwrap();
    let mut var_type: DWORD = 0;
    let mut data = vec![0u8; 8196];
    let mut data_size: DWORD = data.len() as DWORD;
    let ret = unsafe {
      advapi32::RegQueryValueExA(self.0,
                                 name.as_ptr(),
                                 null_mut(),
                                 &mut var_type,
                                 data.as_mut_ptr(),
                                 &mut data_size)
    };
    if ret != (unsafe { transmute(ERROR_SUCCESS) }) {
      return Err(get_error_message());
    }

    data.resize(data_size as usize, 0u8);

    Ok(Value {
      var_type: var_type,
      var_data: data,
    })
  }
}

impl Drop for Key {
  fn drop(&mut self) {
    unsafe { advapi32::RegCloseKey(self.0) };
    self.0 = null_mut();
  }
}
