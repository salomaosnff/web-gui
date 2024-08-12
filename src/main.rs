use std::{sync::Arc, thread};

use app::ApplicationExt;
use invoke::InvokeResult;
use resources::create_static_protocol;
use tao::event_loop::{self, ControlFlow};
use window::{AppWindowBuilder, AppWindowEvent};

mod app;
mod invoke;
mod resources;
mod window;

#[tokio::main]
async fn main() {
  let event_loop = event_loop::EventLoopBuilder::<AppWindowEvent>::with_user_event().build();
  let app = app::Application::new(&event_loop);

  app.add_invoke_handler("echo", |app, command| {
    thread::sleep(std::time::Duration::from_secs(5));
    let message = command.params.first().unwrap().clone();
    InvokeResult::Ok(message)
  });

  app
    .build_window()
    .main()
    .with_title("Hello World!")
    .with_url("assets://app")
    .build(&event_loop);

  app
    .build_window()
    .with_title("Google")
    .with_url("https://google.com")
    .build(&event_loop);

  event_loop.run(move |event, event_loop, control_flow| {
    app.handle_event(event, event_loop, control_flow);
  });
}
