extern crate winapi;
extern crate advapi32;
extern crate kernel32;

use std::env;
use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::path::Path;
use std::process::{Command, Stdio};
use std::ptr::null_mut;

use winapi::DWORD;
use winapi::winnt;
use winapi::minwindef::HKEY;
use winapi::winerror::ERROR_SUCCESS;

const REGRUN_ALREADY_EXECUTED: &'static str = "REGRUN_ALREADY_EXECUTED";


fn expand_environment_strings(s: &str) -> Option<String> {
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
  assert_eq!(Path::new(&expand_environment_strings("%ComSpec%")).canonicalize().unwrap(),
             Path::new("C:\\Windows\\System32\\cmd.exe").canonicalize().unwrap());
}


#[derive(Debug)]
struct Value {
  var_type: DWORD,
  var_data: Vec<u8>,
}

impl Value {
  fn type_str(&self) -> &'static str {
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

  fn to_string(&self) -> Option<String> {
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

struct Key(HKEY);

impl Drop for Key {
  fn drop(&mut self) {
    unsafe { advapi32::RegCloseKey(self.0) };
    self.0 = null_mut();
  }
}

impl Key {
  fn open(hkey: HKEY, path: &str) -> Option<Key> {
    let mut key = null_mut();

    let path = CString::new(path).unwrap();
    let ret =
      unsafe { advapi32::RegOpenKeyExA(hkey, path.as_ptr(), 0, winnt::KEY_QUERY_VALUE, &mut key) };
    if ret != (unsafe { transmute(ERROR_SUCCESS) }) {
      return None;
    }

    Some(Key(key))
  }

  fn query_value(&self, name: &str) -> Option<Value> {
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
      return None;
    }

    data.resize(data_size as usize, 0u8);

    Some(Value {
      var_type: var_type,
      var_data: data,
    })
  }
}


fn read_path_from_registry() -> String {
  let sys_subkey = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";

  let system_path = Key::open(winapi::HKEY_LOCAL_MACHINE, sys_subkey).and_then(|key| {
    key.query_value("Path")
      .and_then(|value| value.to_string().map(|s| expand_environment_strings(&s).unwrap_or(s)))
  });

  let user_path = Key::open(winapi::HKEY_CURRENT_USER, "Environment").and_then(|key| {
    key.query_value("Path")
      .and_then(|value| value.to_string().map(|s| expand_environment_strings(&s).unwrap_or(s)))
  });

  let mut new_path = String::new();
  if let Some(ref path) = user_path {
    new_path += path;
  }
  if let Some(ref path) = system_path {
    if new_path != "" {
      new_path += ";";
    }
    new_path += path;
  }
  new_path
}


fn main() {
  // 再帰的に起動するのを防止する
  if env::vars().find(|&(ref key, _)| key == REGRUN_ALREADY_EXECUTED).is_some() {
    return;
  }

  // 実行ファイル名を取得
  let command = Path::new(&env::args().next().unwrap())
    .file_stem()
    .unwrap()
    .to_string_lossy()
    .into_owned();

  if command == env!("CARGO_PKG_NAME") {
    // client interface tool mode.
    // TODO 仕様を確定する
    // regrun exec hg summary

    if let Some(scmd) = env::args().nth(1) {
      if scmd == "exec" {
        let command = env::args().nth(2).unwrap().to_owned();
        let args: Vec<_> = env::args().skip(3).collect();
        let new_path = read_path_from_registry();
        match Command::new(&command)
            .env("PATH", new_path)
            .env(REGRUN_ALREADY_EXECUTED, "1")
            .args(args.as_slice())
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn() {
            Ok(child) => child,
            Err(err) => {
              println!("could not execute '{}'. The reason is: {:?}", command, err);
              return;
            }
          }
          .wait()
          .expect("failed to wait on child");
      }
    }

  } else {
    // standalone mode
    // 自身の名前と同じコマンドを Windows 側の環境変数から検索し実行する
    let args: Vec<_> = env::args().skip(1).collect();
    let new_path = read_path_from_registry();

    match Command::new(&command)
        .env("PATH", new_path)
        .env(REGRUN_ALREADY_EXECUTED, "1")
        .args(args.as_slice())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn() {
        Ok(child) => child,
        Err(err) => {
          println!("could not execute '{}'. The reason is: {:?}", command, err);
          return;
        }
      }
      .wait()
      .expect("failed to wait on child");
  }
}
