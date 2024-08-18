use crate::{
  app::{App, AppExt},
  async_invoke_handlers,
  invoke::{InvokeCommand, InvokeResult},
};
use std::io::Write;

async fn fs_read(command: InvokeCommand) -> InvokeResult {
  if let Some(path) = command.args.first() {
    let path = if let Some(path) = path.as_str() {
      path.trim()
    } else {
      return InvokeResult::error("Path must be a string");
    };

    if path.is_empty() {
      return InvokeResult::error("Path cannot be empty");
    }

    if let Ok(content) = std::fs::read(path) {
      return InvokeResult::binary(content);
    }

    return InvokeResult::error("File not found!");
  }

  InvokeResult::error("Path is required")
}

async fn fs_write(command: InvokeCommand) -> InvokeResult {
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
}

pub fn apply(app: App) {
  async_invoke_handlers!(app, {
    "fs.read" => fs_read,
    "fs.write" => fs_write
  });
}
