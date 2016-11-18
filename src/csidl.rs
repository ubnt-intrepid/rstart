use winapi;
use shell32;
use windows;

use std::ffi::CStr;
use std::mem::transmute;
use std::ptr::null_mut;


pub enum CSIDL {
  CommonAppData,
  ProgramFiles,
  ProgramFilesX86,
  CommonProgramFiles,
  CommonProgramFilesX86,
}

impl Into<i32> for CSIDL {
  fn into(self) -> i32 {
    match self {
      CSIDL::CommonAppData => winapi::CSIDL_APPDATA,
      CSIDL::ProgramFiles => winapi::CSIDL_PROGRAM_FILES,
      CSIDL::ProgramFilesX86 => winapi::CSIDL_PROGRAM_FILESX86,
      CSIDL::CommonProgramFiles => winapi::CSIDL_PROGRAM_FILES_COMMON,
      CSIDL::CommonProgramFilesX86 => winapi::CSIDL_PROGRAM_FILES_COMMONX86,
    }
  }
}

pub fn get_special_folder_path(csidl: CSIDL) -> Result<String, String> {
  let mut buf = vec![0u8; 8096];

  let ret = unsafe {
    shell32::SHGetSpecialFolderPathA(null_mut(), transmute(buf.as_mut_ptr()), csidl.into(), 0)
  };
  if ret == 0 {
    return Err(windows::get_error_message());
  }

  Ok(unsafe { CStr::from_ptr(transmute(buf.as_ptr())).to_string_lossy().into_owned() })
}

#[test]
fn test_special_folder() {
  assert_eq!(get_special_folder_path(CSIDL::ProgramFiles),
             Ok(r"C:\Program Files".to_owned()));
  assert_eq!(get_special_folder_path(CSIDL::CommonAppData),
             Ok(r"C:\Program Files".to_owned()));
}
