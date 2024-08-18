use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, RwLock},
};

use tao::{
  dpi::PhysicalSize,
  event::Event,
  event_loop::EventLoop,
  rwh_06::{HasWindowHandle, RawWindowHandle},
  window::WindowId,
};
use wry::{http::Request, RequestAsyncResponder};

use crate::app::{App, AppExt};

#[derive(Debug)]
pub enum AppWindowEvent {
  Event {
    name: String,
    payload: serde_json::Value,
    target: Vec<u32>,
  },
  Close {
    window_id: u32,
  },
  Unknown,
  Ready,
}

impl From<Event<'_, AppWindowEvent>> for AppWindowEvent {
  fn from(val: Event<'_, AppWindowEvent>) -> Self {
    match val {
      Event::WindowEvent {
        window_id,
        event: tao::event::WindowEvent::CloseRequested,
        ..
      } => AppWindowEvent::Close {
        window_id: ApplicationWindow::window_id_to_u32(window_id),
      },
      _ => AppWindowEvent::Unknown,
    }
  }
}

pub struct ApplicationWindow {
  receiver: std::sync::mpsc::Receiver<AppWindowEvent>,
  pub labels: RwLock<HashSet<String>>,
  pub app: App,
  pub tao_window: Arc<tao::window::Window>,
  pub wry_webview: wry::WebView,
  pub import_map: RwLock<HashMap<String, String>>,
}

unsafe impl Send for ApplicationWindow {}
unsafe impl Sync for ApplicationWindow {}

pub type AppWindow = Arc<ApplicationWindow>;

impl ApplicationWindow {
  pub fn window_id_to_u32(window_id: WindowId) -> u32 {
    let id = &format!("{:?}", window_id)[18..];
    let id = id.trim_end_matches(")");
    id.parse().expect("Failed to parse window id")
  }
}

pub struct WindowHandle(RawWindowHandle);

impl HasWindowHandle for WindowHandle {
  fn window_handle(
    &self,
  ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(self.0) })
  }
}

pub trait AppWindowExt {
  fn id(&self) -> u32;
  fn title(&self) -> String;
  fn close(&self);
  fn set_title(&self, title: &str);
  fn set_visible(&self, visible: bool);
  fn show(&self);
  fn hide(&self);
  fn eval(&self, script: &str);
  fn emit(&self, event: &str, payload: serde_json::Value);
  fn app(&self) -> App;
  fn has_label(&self, label: &str) -> bool;
  fn window_handle(&self) -> WindowHandle;
  fn center(&self);
  fn block_until_ready(&self);
  fn build_window(&self) -> AppWindowBuilder;
}

impl AppWindowExt for AppWindow {
  fn id(&self) -> u32 {
    ApplicationWindow::window_id_to_u32(self.tao_window.id())
  }

  fn block_until_ready(&self) {
    loop {
      if let Ok(AppWindowEvent::Ready) = self.receiver.recv() {
        break;
      }
    }
  }

  fn title(&self) -> String {
    self.tao_window.title()
  }

  fn close(&self) {
    self.hide();
    let app = self.app();

    let id = self.id();
    let mut windows = app
      .windows
      .write()
      .expect("Failed to acquire lock on windows")
      .clone();

    windows.remove(&id);
  }

  fn window_handle(&self) -> WindowHandle {
    WindowHandle(
      self
        .tao_window
        .window_handle()
        .expect("Failed to get window handle")
        .as_raw(),
    )
  }

  fn set_title(&self, title: &str) {
    self.tao_window.set_title(title);
  }

  fn set_visible(&self, visible: bool) {
    self.tao_window.set_visible(visible);
  }

  fn show(&self) {
    self.set_visible(true);
  }

  fn hide(&self) {
    self.set_visible(false);
  }

  fn eval(&self, script: &str) {
    self
      .wry_webview
      .evaluate_script(script)
      .expect("Failed to evaluate script");
  }

  fn emit(&self, event: &str, payload: serde_json::Value) {
    self
      .app
      .event_loop_proxy
      .send_event(AppWindowEvent::Event {
        name: event.to_string(),
        payload,
        target: vec![self.id()],
      })
      .expect("Failed to send event");
  }

  fn app(&self) -> App {
    self.app.clone()
  }

  fn has_label(&self, label: &str) -> bool {
    self
      .labels
      .read()
      .expect("Failed to acquire lock on labels")
      .contains(label)
  }

  fn center(&self) {
    let tao_window = self.tao_window.clone();

    let PhysicalSize { width, height } = tao_window.inner_size();

    let PhysicalSize {
      width: screen_width,
      height: screen_height,
    } = tao_window
      .current_monitor()
      .expect("Failed to get current monitor")
      .size();

    let x = (screen_width - width) / 2;
    let y = (screen_height - height) / 2;

    tao_window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
  }

  fn build_window(&self) -> AppWindowBuilder {
    self.app().build_window().with_parent(self.clone())
  }
}

pub type CustomProtocolHandler = dyn Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static;

pub struct AppWindowBuilder {
  parent: Option<AppWindow>,
  is_main: bool,
  app: App,
  tao_window_builder: tao::window::WindowBuilder,
  url: Option<String>,
  custom_protocols: HashMap<String, Box<CustomProtocolHandler>>,
  labels: HashSet<String>,
  devtools: bool,
  transparent: bool,
  at_center: bool,
  pub import_map: HashMap<String, String>,
}

impl AppWindowBuilder {
  pub fn new(app: App) -> Self {
    let tao_window_builder = tao::window::WindowBuilder::new();

    Self {
      parent: None,
      is_main: false,
      app,
      tao_window_builder,
      url: None,
      custom_protocols: HashMap::new(),
      import_map: HashMap::new(),
      labels: HashSet::new(),
      devtools: false,
      at_center: false,
      transparent: false,
    }
  }

  pub fn with_parent(mut self, parent: AppWindow) -> Self {
    self.parent = Some(parent);

    self
  }

  pub fn with_label(mut self, label: &str) -> Self {
    self.labels.insert(label.to_string());

    self
  }

  pub fn with_title(mut self, title: &str) -> Self {
    self.tao_window_builder = self.tao_window_builder.with_title(title);

    self
  }

  pub fn with_url(mut self, url: &str) -> Self {
    self.url = Some(url.to_string());

    self
  }

  pub fn with_html(self, html: &str) -> Self {
    self.with_url(&format!("data:text/html,{}", html))
  }

  pub fn with_protocol<H>(mut self, schema: &str, handler: H) -> Self
  where
    H: Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static,
  {
    self
      .custom_protocols
      .insert(schema.to_string(), Box::new(handler));

    self
  }

  pub fn with_js_module(mut self, name: &str, url: &str) -> Self {
    self.import_map.insert(name.to_string(), url.to_string());

    self
  }

  pub fn main(mut self) -> Self {
    self.is_main = true;

    self.with_label("main")
  }

  pub fn with_visible(mut self, visible: bool) -> Self {
    self.tao_window_builder = self.tao_window_builder.with_visible(visible);

    self
  }

  pub fn with_transparent(mut self, transparent: bool) -> Self {
    self.transparent = transparent;

    self
  }

  pub fn with_decorations(mut self, decorations: bool) -> Self {
    self.tao_window_builder = self.tao_window_builder.with_decorations(decorations);

    self
  }

  pub fn with_closable(mut self, closeable: bool) -> Self {
    self.tao_window_builder = self.tao_window_builder.with_closable(closeable);

    self
  }

  pub fn with_size(mut self, width: f64, height: f64) -> Self {
    self.tao_window_builder = self
      .tao_window_builder
      .with_inner_size(PhysicalSize::new(width, height));

    self
  }

  pub fn at_center(mut self) -> Self {
    self.at_center = true;

    self
  }

  pub fn with_devtools(mut self) -> Self {
    self.devtools = true;

    self
  }

  pub fn build(self, event_loop: &EventLoop<AppWindowEvent>) -> AppWindow {
    let (tx, rx) = std::sync::mpsc::channel();
    let tao_window = Arc::new(
      self
        .tao_window_builder
        .with_transparent(self.transparent)
        .build(event_loop)
        .expect("Failed to build window"),
    );

    #[cfg(any(
      target_os = "windows",
      target_os = "macos",
      target_os = "ios",
      target_os = "android"
    ))]
    let mut builder = WebViewBuilder::new(&tao_window);

    #[cfg(not(any(
      target_os = "windows",
      target_os = "macos",
      target_os = "ios",
      target_os = "android"
    )))]
    let mut builder = {
      use tao::platform::unix::WindowExtUnix;
      use wry::WebViewBuilderExtUnix;
      let vbox = tao_window.default_vbox().unwrap();
      wry::WebViewBuilder::new_gtk(vbox)
    };

    let tao_window2 = tao_window.clone();
    builder = builder
      .with_transparent(self.transparent)
      .with_document_title_changed_handler(move |title| {
        tao_window2.set_title(title.as_str());
      });

    builder = builder.with_initialization_script(&format!(
      "Object.defineProperty(window, 'ID', {{ value: {}, writable: false, enumerable: true }});",
      ApplicationWindow::window_id_to_u32(tao_window.id())
    ));

    builder = builder.with_on_page_load_handler(move |event, _| {
      if let wry::PageLoadEvent::Finished = event {
        tx.send(AppWindowEvent::Ready)
          .expect("Failed to send ready event");
      }
    });

    builder = builder.with_initialization_script(
      include_str!("./scripts/modules.js")
        .replace("get_import_map()", {
          serde_json::to_string(&{
            let mut import_map = self
              .app
              .import_map
              .read()
              .expect("Failed to acquire lock on import map")
              .clone();

            for (name, url) in &self.import_map {
              import_map.insert(name.clone(), url.clone());
            }

            import_map
          })
          .expect("Failed to serialize import map")
          .as_str()
        })
        .replace(
          "get_extensions()",
          serde_json::to_string::<Vec<serde_json::value::Value>>(&{
            let extensions = self
              .app
              .state
              .read()
              .expect("Failed to acquire lock on state");
            let extensions = extensions
              .extension_host
              .extensions_for_window_labels(&self.labels);

            extensions
              .into_iter()
              .map(|extension| {
                serde_json::json!({
                  "id": extension.id(),
                  "dir": extension.dir(),
                  "public_url": extension.public_url(),
                  "main_script_url": extension.main_script_url(),
                })
              })
              .collect::<Vec<_>>()
          })
          .expect("Failed to serialize extensions")
          .as_str(),
        )
        .as_str(),
    );

    if let Some(url) = self.url {
      builder = builder.with_url(&url);
    }

    for (name, handler) in self.custom_protocols {
      builder = builder.with_asynchronous_custom_protocol(name, handler);
    }
    #[cfg(debug_assertions)]
    {
      builder = builder.with_devtools(self.devtools);
    }

    let wry_webview = builder.build().expect("Failed to build webview");

    #[cfg(debug_assertions)]
    {
      if self.devtools {
        wry_webview.open_devtools();
      }
    }

    let window = Arc::new(ApplicationWindow {
      tao_window: tao_window.clone(),
      wry_webview,
      app: self.app.clone(),
      import_map: RwLock::new(self.import_map),
      labels: RwLock::new(self.labels),
      receiver: rx,
    });

    if self.at_center {
      window.center();
    }

    let window_id = window.id();

    self
      .app
      .windows
      .write()
      .expect("Failed to acquire lock on windows")
      .insert(window_id, window.clone());

    if self.is_main {
      self
        .app
        .main_window_id
        .write()
        .expect("Failed to acquire lock on main window id")
        .replace(window_id);
    }

    window
  }
}
