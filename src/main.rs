extern crate winapi;
extern crate advapi32;
extern crate kernel32;
extern crate shell32;

mod windows;
mod registry;
mod csidl;

use std::env;
use std::process::{Command, Stdio};
use registry::{Key, RootKey};

fn main() {
  let name = env::args().skip(1).next().unwrap_or_else(|| {
    println!("command is empty");
    std::process::exit(1)
  });
  let args: Vec<_> = env::args().skip(2).collect();

  let path = read_path_from_registry().expect("failed to get PATH");
  std::env::set_var("PATH", path.join(";"));

  Command::new(name)
    .args(args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
    .expect("failed to spawn process");
}

fn read_path_from_registry() -> Result<Vec<String>, String> {
  let system_env = Key::open(RootKey::LocalMachine,
                             r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")?;
  let user_env = Key::open(RootKey::CurrentUser, "Environment")?;

  let system_path: Vec<_> = system_env.query_value("Path")?
    .to_string()
    .map(|s| windows::expand_env(&s).unwrap_or(s))
    .unwrap_or("".into())
    .split(";")
    .map(ToOwned::to_owned)
    .collect();

  let user_path: Vec<_> = user_env.query_value("Path")?
    .to_string()
    .map(|s| windows::expand_env(&s).unwrap_or(s))
    .unwrap_or("".into())
    .split(";")
    .map(ToOwned::to_owned)
    .collect();

  Ok(system_path.into_iter().chain(user_path).collect())
}
