use std::io::Write;

use serde_json::json;

use crate::{
  app::{App, AppExt},
  invoke::InvokeResult,
};

pub fn apply_plugins<T: Sync + Send + 'static>(app: App<T>) {
  app.add_invoke_handler("echo", |_app, command| {
    std::thread::sleep(std::time::Duration::from_secs(10));
    let message = command.args.first().unwrap().clone();
    InvokeResult::json(message)
  });

  app.add_invoke_handler("env.get", |_app, command| {
    if let Some(key) = command.args.first() {
      if let Ok(value) = std::env::var(key.as_str().unwrap()) {
        return InvokeResult::json(value.into());
      }

      return InvokeResult::Err("Variable not found".to_string());
    }

    InvokeResult::Err("Invalid key".to_string())
  });

  app.add_invoke_handler("fs.read", |_app, command| {
    if let Some(path) = command.args.first() {
      if let Ok(content) = std::fs::read(path.as_str().unwrap()) {
        return InvokeResult::binary(content);
      }

      return InvokeResult::Err("File not found".to_string());
    }

    InvokeResult::Err("Invalid path".to_string())
  });

  app.add_invoke_handler("shell.exec", |_app, command| {
    let args = command.args.iter();
    let bin = if let Some(bin) = args.clone().next() {
      bin.as_str().unwrap()
    } else {
      return InvokeResult::Err("Invalid command".to_string());
    };

    let args = args.map(|arg| arg.as_str().unwrap()).collect::<Vec<&str>>();

    if let Ok(output) = std::process::Command::new(bin).args(args).output() {
      let stdout = String::from_utf8_lossy(&output.stdout).to_string();
      let stderr = String::from_utf8_lossy(&output.stderr).to_string();
      let exit_code = output.status.code().unwrap_or(1);

      return InvokeResult::json(json!({
        "stdout": stdout,
        "stderr": stderr,
        "exit_code": exit_code,
      }));
    }

    InvokeResult::Err("Command failed".to_string())
  });

  app.add_invoke_handler("bytes.random", |_app, command| {
    let length = if let Some(length) = command.args.first() {
      length.as_u64().unwrap() as usize
    } else {
      return InvokeResult::Err("Invalid length".to_string());
    };

    let bytes: Vec<u8> = (0..length).map(|_| rand::random()).collect();

    InvokeResult::binary(bytes)
  });

  app.add_invoke_handler("fs.write", |_app, command| {
    let filename = if let Some(filename) = command.args.first() {
      filename.as_str().unwrap()
    } else {
      return InvokeResult::Err("Invalid filename".to_string());
    };

    let content = if let Some(content) = command.args.get(1) {
      content.as_str().unwrap()
    } else {
      return InvokeResult::Err("Invalid content".to_string());
    };

    if let Ok(mut file) = std::fs::File::create(filename) {
      if let Err(err) = file.write_all(content.as_bytes()) {
        return InvokeResult::Err(err.to_string());
      }

      return InvokeResult::json(serde_json::Value::Null);
    }

    InvokeResult::Err("Failed to write file".to_string())
  });
}
