use winapi;
use advapi32;
use windows;

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::ptr::null_mut;

use winapi::DWORD;
use winapi::winnt;
use winapi::minwindef::HKEY;
use winapi::winerror::ERROR_SUCCESS;


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
  pub fn to_string(&self) -> Option<String> {
    match self.var_type {
      winnt::REG_SZ |
      winnt::REG_EXPAND_SZ => Some(make_string(self.var_data.as_ptr())),
      _ => None,
    }
  }
}

fn make_string(ptr: *const u8) -> String {
  let cstr = unsafe { CStr::from_ptr(transmute(ptr)) };
  cstr.to_string_lossy().into_owned()
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
      return Err(windows::get_error_message());
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
      return Err(windows::get_error_message());
    }

    data.resize(data_size as usize, 0u8);

    Ok(Value {
      var_type: var_type,
      var_data: data,
    })
  }

  #[allow(dead_code)]
  pub fn enum_values(&self) -> Result<Vec<(String, Value)>, String> {
    const MAX_ITEMS: u32 = 1_000_000;

    let mut values = Vec::new();

    for i in 0..MAX_ITEMS {
      let mut var_type: DWORD = 0;
      let mut name = vec![0u8; 8196];
      let mut name_size: DWORD = name.len() as DWORD;
      let mut data = vec![0u8; 8196];
      let mut data_size: DWORD = data.len() as DWORD;
      let ret = unsafe {
        advapi32::RegEnumValueA(self.0,
                                i,
                                transmute(name.as_mut_ptr()),
                                &mut name_size,
                                null_mut(),
                                &mut var_type,
                                data.as_mut_ptr(),
                                &mut data_size)
      };

      match ret as u32 {
        winapi::ERROR_SUCCESS => {
          let name =
            unsafe { CStr::from_ptr(transmute(name.as_ptr())).to_string_lossy().into_owned() };
          let value = Value {
            var_type: var_type,
            var_data: data,
          };

          values.push((name, value));
        }
        winapi::ERROR_NO_MORE_ITEMS => break,
        _ => return Err(windows::get_error_message()),
      }
    }

    Ok(values)
  }
}

impl Drop for Key {
  fn drop(&mut self) {
    unsafe { advapi32::RegCloseKey(self.0) };
    self.0 = null_mut();
  }
}
