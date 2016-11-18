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
  // prevent to execute the command infinitely
  if env::var(REGRUN_ALREADY_EXECUTED).is_ok() {
    for (key, value) in env::vars() {
      println!("{} = {}", key, value);
    }

  } else {
    let command = Path::new(&env::args().next().unwrap())
      .file_stem()
      .unwrap()
      .to_string_lossy()
      .into_owned();
    let args: Vec<_> = env::args().skip(1).collect();

    execute(&command, &args);
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

fn execute(name: &str, args: &[String]) {
  let mut envs = Vec::new();
  envs.extend(registry::Key::open(RootKey::CurrentUser, "Environment")
    .unwrap()
    .enum_values()
    .unwrap());
  envs.extend(registry::Key::open(RootKey::CurrentUser, "Volatile Environment")
    .unwrap()
    .enum_values()
    .unwrap());
  envs.extend(Key::open(RootKey::LocalMachine,
                        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
    .unwrap()
    .enum_values()
    .unwrap());
  let path = read_path_from_registry().unwrap();

  let mut command = Command::new(name);
  command.env_clear()
    .args(args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit());

  for (key, value) in envs {
    let value = value.to_string().unwrap();
    command.env(key, windows::expand_env(&value).unwrap_or(value));
  }
  command.env(REGRUN_ALREADY_EXECUTED, "1");
  command.env("PATH", path);

  match command.spawn() {
      Ok(child) => child,
      Err(err) => {
        println!("could not execute '{} {:?}'. The reason is: {:?}",
                 name,
                 args,
                 err);
        return;
      }
    }
    .wait()
    .expect("failed to wait on child");
}
