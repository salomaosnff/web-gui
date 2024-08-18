use serde_json::json;

use crate::{
  app::{App, AppExt},
  async_invoke_handlers,
  invoke::{InvokeCommand, InvokeResult},
  window::AppWindowExt,
};

async fn dialog_show(command: InvokeCommand) -> InvokeResult {
  let options = if let Some(options) = command.args.first() {
    if let Some(options) = options.as_object() {
      options
    } else {
      return InvokeResult::error("Dialog Options must be an object");
    }
  } else {
    return InvokeResult::error("Dialog Options not provided");
  };

  let title = options.get("title").and_then(|v| v.as_str());
  let message = options.get("message").and_then(|v| v.as_str());
  let level = options
    .get("level")
    .and_then(|v| v.as_str())
    .unwrap_or("info");

  let mut builder = rfd::MessageDialog::new();

  if let Some(title) = title {
    builder = builder.set_title(title);
  }

  if let Some(message) = message {
    builder = builder.set_description(message);
  }

  let level = match level {
    "warning" => rfd::MessageLevel::Warning,
    "error" => rfd::MessageLevel::Error,
    _ => rfd::MessageLevel::Info,
  };

  builder
    .set_parent(&command.window.window_handle())
    .set_level(level)
    .show();

  ().into()
}

async fn dialog_confirm(command: InvokeCommand) -> InvokeResult {
  let options = if let Some(options) = command.args.first() {
    if let Some(options) = options.as_object() {
      options
    } else {
      return InvokeResult::error("Dialog Options must be an object");
    }
  } else {
    return InvokeResult::error("Dialog Options not provided");
  };

  let title = options.get("title").and_then(|v| v.as_str());
  let message = options.get("message").and_then(|v| v.as_str());
  let level = options
    .get("level")
    .and_then(|v| v.as_str())
    .unwrap_or("info");

  let mut builder = rfd::MessageDialog::new();

  if let Some(title) = title {
    builder = builder.set_title(title);
  }

  if let Some(message) = message {
    builder = builder.set_description(message);
  }

  let level = match level {
    "warning" => rfd::MessageLevel::Warning,
    "error" => rfd::MessageLevel::Error,
    _ => rfd::MessageLevel::Info,
  };

  let result = builder
    .set_parent(&command.window.window_handle())
    .set_level(level)
    .set_buttons(rfd::MessageButtons::YesNo)
    .show();

  (result == rfd::MessageDialogResult::Yes).into()
}

async fn dialog_files_open(command: InvokeCommand) -> InvokeResult {
  let options = if let Some(options) = command.args.first() {
    if let Some(options) = options.as_object() {
      options
    } else {
      return InvokeResult::error("Dialog Options must be an object");
    }
  } else {
    return InvokeResult::error("Dialog Options not provided");
  };

  let title = options.get("title").and_then(|v| v.as_str());
  let default_path = options.get("defaultPath").and_then(|v| v.as_str());
  let filters = options
    .get("filters")
    .and_then(|v| v.as_object())
    .map(|filters| {
      filters
        .iter()
        .map(|(name, pattern)| {
          (
            name.as_str(),
            pattern
              .as_array()
              .map(|v| v.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>()),
          )
        })
        .collect::<Vec<_>>()
    });

  let mut builder = rfd::FileDialog::new()
    .set_parent(&command.window.window_handle())
    .set_title(title.unwrap_or("Open File"))
    .set_directory(default_path.unwrap_or(""));

  for (name, patterns) in filters.unwrap_or_default() {
    builder = builder.add_filter(name, patterns.unwrap_or_default().as_slice());
  }

  if options
    .get("multiple")
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
  {
    builder
      .pick_files()
      .map_or_else(std::vec::Vec::new, |paths| {
        paths
          .into_iter()
          .map(|path| json!(path.to_string_lossy().to_string()))
          .collect()
      })
      .into()
  } else {
    builder
      .pick_file()
      .map_or_else(
        || serde_json::Value::Null,
        |path| serde_json::json!(path.to_string_lossy()),
      )
      .into()
  }
}

async fn dialog_files_save(command: InvokeCommand) -> InvokeResult {
  let options = if let Some(options) = command.args.first() {
    if let Some(options) = options.as_object() {
      options
    } else {
      return InvokeResult::error("Dialog Options must be an object");
    }
  } else {
    return InvokeResult::error("Dialog Options not provided");
  };

  let title = options.get("title").and_then(|v| v.as_str());
  let default_path = options.get("defaultPath").and_then(|v| v.as_str());
  let filters = options
    .get("filters")
    .and_then(|v| v.as_object())
    .map(|filters| {
      filters
        .iter()
        .map(|(name, pattern)| {
          (
            name.as_str(),
            pattern
              .as_array()
              .map(|v| v.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>()),
          )
        })
        .collect::<Vec<_>>()
    });
  let can_create_directories = options
    .get("canCreateDirectories")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);

  let file_name = options
    .get("defaultFileName")
    .and_then(|v| v.as_str())
    .map(|v| v.to_string());

  let mut builder = rfd::FileDialog::new()
    .set_parent(&command.window.window_handle())
    .set_title(title.unwrap_or("Save File"))
    .set_directory(default_path.unwrap_or(""))
    .set_can_create_directories(can_create_directories);

  if let Some(file_name) = file_name {
    builder = builder.set_file_name(file_name);
  }

  for (name, patterns) in filters.unwrap_or_default() {
    builder = builder.add_filter(name, patterns.unwrap_or_default().as_slice());
  }

  builder.save_file().map_or_else(
    || ().into(),
    |path| InvokeResult::json(serde_json::json!(path)),
  )
}

async fn dialog_select_folder(command: InvokeCommand) -> impl Into<InvokeResult> {
  let options = if let Some(options) = command.args.first() {
    if let Some(options) = options.as_object() {
      options
    } else {
      return InvokeResult::error("Dialog Options must be an object");
    }
  } else {
    return InvokeResult::error("Dialog Options not provided");
  };

  let title = options.get("title").and_then(|v| v.as_str());
  let default_path = options.get("defaultPath").and_then(|v| v.as_str());
  let can_create_directories = options
    .get("canCreateDirectories")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);

  let builder = rfd::AsyncFileDialog::new()
    .set_parent(&command.window.window_handle())
    .set_title(title.unwrap_or("Open Folder"))
    .set_directory(default_path.unwrap_or(""))
    .set_can_create_directories(can_create_directories);

  if options
    .get("multiple")
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
  {
    match builder.pick_folders().await {
      Some(file_handlers) => file_handlers
        .into_iter()
        .map(|file_handle| {
          let path = file_handle.path().to_string_lossy().to_string();
          json!({
            "path": path.clone(),
            "type": "folder",
          })
        })
        .collect::<Vec<_>>()
        .into(),
      None => ().into(),
    }
  } else {
    match builder.pick_folder().await {
      Some(file_handle) => {
        let path = file_handle.path().to_string_lossy().to_string();
        json!({
          "path": path.clone(),
          "type": "folder",
        })
        .into()
      }
      None => ().into(),
    }
  }
}

pub fn apply(app: App) {
  async_invoke_handlers!(app, {
    "dialog.show" => dialog_show,
    "dialog.confirm" => dialog_confirm,
    "dialog.files.open" => dialog_files_open,
    "dialog.files.save" => dialog_files_save,
    "dialog.folder.select" => dialog_select_folder
  });
}
