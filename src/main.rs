extern crate winapi;
extern crate advapi32;
extern crate kernel32;
extern crate shell32;

mod windows;
mod registry;
mod csidl;

use std::env;
use std::process::{Command, Stdio};

fn main() {
  let name = env::args().skip(1).next().unwrap_or_else(|| {
    println!("command is empty");
    std::process::exit(1)
  });
  let args: Vec<_> = env::args().skip(2).collect();

  let system_path: Vec<_> = registry::query_system_env("Path")
    .map(split_value)
    .unwrap();
  let user_path: Vec<_> = registry::query_user_env("Path")
    .map(split_value)
    .unwrap();
  let path = system_path.into_iter()
    .chain(user_path)
    .fold(String::new(), |mut acc, path| {
      if !acc.is_empty() {
        acc.push_str(";");
      }
      acc.push_str(&path);
      acc
    });
  env::set_var("PATH", path);

  Command::new(name)
    .args(args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .expect("failed to spawn process");
}

fn split_value(value: registry::Value) -> Vec<String> {
  value.to_string()
    .map(|s| windows::expand_env(&s).unwrap_or(s))
    .unwrap_or_default()
    .split(";")
    .map(ToOwned::to_owned)
    .collect()
}
