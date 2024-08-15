use std::io::Write;

use crate::{
  app::{App, AppExt},
  invoke::InvokeResult,
};

pub fn apply_plugins<T: Sync + Send + 'static>(app: App<T>) {
  app.add_invoke_handler("env.get", |_app, command| {
    if let Some(key) = command.args.first() {
      let key = if let Some(key) = key.as_str() {
        key
      } else {
        return InvokeResult::error("Key must be a string");
      };

      if let Ok(value) = std::env::var(key) {
        return InvokeResult::json(serde_json::json!(value));
      }

      return InvokeResult::json(serde_json::Value::Null);
    }

    InvokeResult::error("Key not provided")
  });

  app.add_invoke_handler("fs.read", |_app, command| {
    if let Some(path) = command.args.first() {
      if let Ok(content) = std::fs::read(path.as_str().unwrap()) {
        return InvokeResult::binary(content);
      }

      return InvokeResult::error("File not found");
    }

    InvokeResult::error("Path not provided")
  });

  app.add_invoke_handler("fs.write", |_app, command| {
    let filename = if let Some(filename) = command.args.first() {
      filename.as_str().unwrap()
    } else {
      return InvokeResult::error("Invalid filename");
    };

    let content = if let Some(content) = command.args.get(1) {
      content.as_str().unwrap()
    } else {
      return InvokeResult::error("Invalid content");
    };

    if let Ok(mut file) = std::fs::File::create(filename) {
      if let Err(err) = file.write_all(content.as_bytes()) {
        return InvokeResult::Err(err.to_string());
      }

      return InvokeResult::json(serde_json::Value::Null);
    }

    InvokeResult::error("Failed to write file")
  });
}
