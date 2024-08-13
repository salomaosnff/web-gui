use std::os;

use app::ApplicationExt;
use invoke::InvokeResult;
use serde_json::json;
use window::{AppWindowEvent, AppWindowExt};

mod app;
mod invoke;
mod resources;
mod window;

#[tokio::main]
async fn main() {
  let event_loop = tao::event_loop::EventLoopBuilder::<AppWindowEvent>::with_user_event().build();
  let app = app::Application::new(&event_loop);

  app.add_invoke_handler("echo", |_app, command| {
    let message = command.args.first().unwrap().clone();
    InvokeResult::Ok(message)
  });

  app.add_invoke_handler("env.get", |_app, command| {
    if let Some(key) = command.args.first() {
      if let Ok(value) = std::env::var(key.as_str().unwrap()) {
        return InvokeResult::Ok(value.into());
      }

      return InvokeResult::Err("Variable not found".to_string());
    }

    InvokeResult::Err("Invalid key".to_string())
  });

  app.add_invoke_handler("fs.read", |_app, command| {
    if let Some(path) = command.args.first() {
      if let Ok(content) = std::fs::read_to_string(path.as_str().unwrap()) {
        return InvokeResult::Ok(content.into());
      }

      return InvokeResult::Err("File not found".to_string());
    }

    InvokeResult::Err("Invalid path".to_string())
  });

  app.add_invoke_handler("shell.execSync", |_app, command| {
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

      return InvokeResult::Ok(json!({ "stdout": stdout, "stderr": stderr }));
    }

    InvokeResult::Err("Command failed".to_string())
  });

  let window = app
    .build_window()
    .main()
    .with_title("Hello World!")
    .with_url("assets://app")
    .build(&event_loop);

  tokio::task::spawn(async move {
    let window = window.clone();

    loop {
      let stat = std::fs::read_to_string("/proc/stat").expect("Failed to read /proc/stat");

      window.emit("cpu-usage", json!(stat));

      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
  });

  event_loop.run(move |event, event_loop, control_flow| {
    app.handle_event(event, event_loop, control_flow);
  });
}
