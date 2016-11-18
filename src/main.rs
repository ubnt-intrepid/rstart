extern crate winapi;
extern crate advapi32;
extern crate kernel32;

mod windows;
mod registry;

use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

use registry::{Key, RootKey};

const REGRUN_ALREADY_EXECUTED: &'static str = "REGRUN_ALREADY_EXECUTED";


fn main() {
  let envs = registry::Key::open(RootKey::CurrentUser, "Environment").unwrap();
  for env in envs.enum_values().unwrap() {
    println!("{} = {:?}", env.0, env.1.to_string());
  }
  let envs = registry::Key::open(RootKey::CurrentUser, "Volatile Environment").unwrap();
  for env in envs.enum_values().unwrap() {
    println!("{} = {:?}", env.0, env.1.to_string());
  }

  let envs = Key::open(RootKey::LocalMachine,
                       r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
    .unwrap();
  for env in envs.enum_values().unwrap() {
    println!("{} = {:?}", env.0, env.1.to_string());
  }

  println!("------------");
  for (key, value) in std::env::vars() {
    println!("{} = {}", key, value);
  }

  return;

  // prevent to execute the command infinitely
  if env::var(REGRUN_ALREADY_EXECUTED).is_ok() {
    return;
  }

  // 実行ファイル名を取得
  let command = Path::new(&env::args().next().unwrap())
    .file_stem()
    .unwrap()
    .to_string_lossy()
    .into_owned();

  if command != env!("CARGO_PKG_NAME") {
    let args: Vec<_> = env::args().skip(1).collect();
    let new_path = read_path_from_registry().unwrap();

    execute(&command, &args, &new_path);
  }
}


fn read_path_from_registry() -> Result<String, String> {
  let system_env = Key::open(RootKey::LocalMachine,
                             r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")?;
  let user_env = Key::open(RootKey::CurrentUser, "Environment")?;

  let system_path = system_env.query_value("Path")?
    .to_string()
    .map(|s| windows::expand_env(&s).unwrap_or(s));

  let user_path = user_env.query_value("Path")?
    .to_string()
    .map(|s| windows::expand_env(&s).unwrap_or(s));

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
  Ok(new_path)
}

fn execute(command: &str, args: &[String], path: &str) {
  match Command::new(command)
      .env(REGRUN_ALREADY_EXECUTED, "1")
      .env("PATH", path)
      .args(args)
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
