use app::AppExt;
use window::AppWindowEvent;

mod app;
mod invoke;
mod plugins;
mod resources;
mod window;

#[tokio::main]
async fn main() {
  let event_loop = tao::event_loop::EventLoopBuilder::<AppWindowEvent>::with_user_event().build();
  let app = app::Application::new(&event_loop);

  import_map!(app, {
    "lenz/fs" => "assets://app/fs.mjs",
  });

  plugins::fs::apply_plugins(app.clone());

  app
    .build_window()
    .main()
    .with_title("Hello World!")
    .with_url("assets://app")
    .with_js_module("lodash", "https://cdn.skypack.dev/lodash")
    .build(&event_loop);

  event_loop.run(move |event, event_loop, control_flow| {
    app.handle_event(event, event_loop, control_flow);
  });
}
