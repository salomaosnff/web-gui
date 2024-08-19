use app::{
  resources::custom_protocol, window::{AppWindowEvent, AppWindowExt}, AppExt
};
use serde_json::json;

mod app;
mod plugins;
mod state;

#[tokio::main]
async fn main() {
  let event_loop = tao::event_loop::EventLoopBuilder::<AppWindowEvent>::with_user_event().build();
  let app = app::Application::new(&event_loop, state::AppState::new());

  plugins::apply(app.clone());

  let main = app
    .build_window()
    .main()
    .with_visible(false)
    .with_title("Lenz")
    .with_url(std::env::var("LENZ_URL").unwrap_or_else(|_| "lenz://app/index.html".to_owned()))
    .with_devtools()
    .at_center()
    .build(&event_loop);

  let splash = main
    .build_window()
    .with_label("splash")
    .with_visible(true)
    .with_transparent(true)
    .with_decorations(false)
    .with_closable(false)
    .with_size(400.0, 400.0)
    .at_center()
    .with_url(custom_protocol("lenz", "app/splash.html"))
    .build(&event_loop);

  let app2 = app.clone();

  tokio::task::spawn(async move {
    splash.block_until_ready();
    main.block_until_ready();

    let extensions = app2
      .state
      .read()
      .unwrap()
      .extension_host
      .search_extensions();

    for extension in extensions {
      let has_main_script = extension.has_main_script();
      let extension_json = json!({
        "id": extension.id(),
        "name": extension.manifest().name,
        "main_script_url": extension.main_script_url()
      });
      splash.emit("extension.activate", extension_json.clone());

      extension.activate(app2.clone());

      splash.emit("extension.activated", extension_json.clone());

      if has_main_script {
        main.emit("extension.activated", extension_json.clone());
      }
    }

    main.show();
    splash.close();
  });

  event_loop.run(move |event, event_loop, control_flow| {
    app.handle_event(event, event_loop, control_flow);
  });
}
