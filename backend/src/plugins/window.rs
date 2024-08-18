use crate::{
  app::{App, AppExt},
  async_invoke_handlers,
  invoke::{InvokeCommand, InvokeResult},
  window::AppWindowExt,
};

async fn window_emit_label(command: InvokeCommand) -> InvokeResult {
  let label = if let Some(label) = command.args.first() {
    if let Some(label) = label.as_str() {
      label
    } else {
      return InvokeResult::error("Label must be a string");
    }
  } else {
    return InvokeResult::error("Label not provided");
  };
  let event = if let Some(event) = command.args.get(1) {
    if let Some(event) = event.as_str() {
      event
    } else {
      return InvokeResult::error("Event must be a string");
    }
  } else {
    return InvokeResult::error("Event not provided");
  };
  let payload = command.args.get(2);
  let windows = command.app.get_windows_by_label(label);

  for window in windows {
    window.emit(event, payload.unwrap_or(&serde_json::Value::Null).clone());
  }

  ().into()
}

async fn window_emit_all(command: InvokeCommand) -> InvokeResult {
  let event = if let Some(event) = command.args.first() {
    if let Some(event) = event.as_str() {
      event
    } else {
      return InvokeResult::error("Event must be a string");
    }
  } else {
    return InvokeResult::error("Event not provided");
  };
  let payload = command.args.get(1);

  command
    .app
    .emit(event, payload.unwrap_or(&serde_json::Value::Null).clone());

  ().into()
}

async fn window_get_all(command: InvokeCommand) -> InvokeResult {
  InvokeResult::json(
    command
      .app
      .windows
      .read()
      .expect("Failed to acquire lock on windows. This should never happen as the lock is poisoned")
      .keys()
      .cloned()
      .collect::<Vec<u32>>()
      .into(),
  )
}

async fn window_get_by_label(command: InvokeCommand) -> InvokeResult {
  let label = if let Some(label) = command.args.first() {
    if let Some(label) = label.as_str() {
      label
    } else {
      return InvokeResult::error("Label must be a string");
    }
  } else {
    return InvokeResult::error("Label not provided");
  };

  InvokeResult::json(
    command
      .app
      .get_windows_by_label(label)
      .iter()
      .map(|window| window.id())
      .collect::<Vec<u32>>()
      .into(),
  )
}

async fn window_set_visible(command: InvokeCommand) -> InvokeResult {
  let window_id = if let Some(window_id) = command.args.first() {
    if let Some(window_id) = window_id.as_u64() {
      window_id as u32
    } else {
      return InvokeResult::error("Window ID must be a number");
    }
  } else {
    return InvokeResult::error("Window ID not provided");
  };

  let visible = if let Some(visible) = command.args.get(1) {
    if let Some(visible) = visible.as_bool() {
      visible
    } else {
      return InvokeResult::error("Visible must be a boolean");
    }
  } else {
    return InvokeResult::error("Visible not provided");
  };

  if let Some(window) = command.app.get_window(window_id) {
    window.set_visible(visible);
  }

  ().into()
}

async fn window_close(command: InvokeCommand) -> InvokeResult {
  let window_id = if let Some(window_id) = command.args.first() {
    if let Some(window_id) = window_id.as_u64() {
      window_id as u32
    } else {
      return InvokeResult::error("Window ID must be a number");
    }
  } else {
    return InvokeResult::error("Window ID not provided");
  };

  if let Some(window) = command.app.get_window(window_id) {
    window.close();
  }

  ().into()
}

async fn window_get_title(command: InvokeCommand) -> InvokeResult {
  let window_id = if let Some(window_id) = command.args.first() {
    if let Some(window_id) = window_id.as_u64() {
      window_id as u32
    } else {
      return InvokeResult::error("Window ID must be a number");
    }
  } else {
    return InvokeResult::error("Window ID not provided");
  };

  if let Some(window) = command.app.get_window(window_id) {
    return InvokeResult::json(window.title().into());
  }

  InvokeResult::error("Window not found")
}

async fn window_set_title(command: InvokeCommand) -> InvokeResult {
  let window_id = if let Some(window_id) = command.args.first() {
    if let Some(window_id) = window_id.as_u64() {
      window_id as u32
    } else {
      return InvokeResult::error("Window ID must be a number");
    }
  } else {
    return InvokeResult::error("Window ID not provided");
  };

  let title = if let Some(title) = command.args.get(1) {
    if let Some(title) = title.as_str() {
      title
    } else {
      return InvokeResult::error("Title must be a string");
    }
  } else {
    return InvokeResult::error("Title not provided");
  };

  if let Some(window) = command.app.get_window(window_id) {
    window.set_title(title);
  }

  ().into()
}

pub fn apply(app: App) {
  async_invoke_handlers!(app, {
    "window.emit_label" => window_emit_label,
    "window.emit_all" => window_emit_all,
    "window.get_all" => window_get_all,
    "window.get_by_label" => window_get_by_label,
    "window.set_visible" => window_set_visible,
    "window.close" => window_close,
    "window.get_title" => window_get_title,
    "window.set_title" => window_set_title
  });
}
