use std::{
  collections::HashMap,
  sync::{Arc, RwLock, Weak},
};

use tao::{event::Event, event_loop::EventLoop, rwh_06::HasWindowHandle, window::WindowId};
use wry::{http::Request, RequestAsyncResponder};

use crate::app::App;

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
}

impl Into<AppWindowEvent> for Event<'_, AppWindowEvent> {
  fn into(self) -> AppWindowEvent {
    match self {
      Event::WindowEvent {
        window_id, event, ..
      } => match event {
        tao::event::WindowEvent::CloseRequested => AppWindowEvent::Close {
          window_id: ApplicationWindow::window_id_to_u32(window_id),
        },
        _ => AppWindowEvent::Unknown,
      },
      _ => AppWindowEvent::Unknown,
    }
  }
}

pub struct ApplicationWindow<T> {
  app: App<T>,
  tao_window: Arc<tao::window::Window>,
  wry_webview: wry::WebView,
}

unsafe impl<T> Send for ApplicationWindow<T> {}
unsafe impl<T> Sync for ApplicationWindow<T> {}

pub type AppWindow<T> = Arc<RwLock<ApplicationWindow<T>>>;

impl ApplicationWindow<()> {
  pub fn window_id_to_u32(window_id: WindowId) -> u32 {
    let id = &format!("{:?}", window_id)[18..];
    let id = id.trim_end_matches(")");
    id.parse().expect("Failed to parse window id")
  }
}

pub trait AppWindowExt<T> {
  fn id(&self) -> u32;
  fn set_title(&self, title: &str);
  fn show(&self);
  fn hide(&self);
  fn eval(&self, script: &str);
  fn emit(&self, event: &str, payload: serde_json::Value);
  fn app(&self) -> App<T>;
}

impl<T> AppWindowExt<T> for AppWindow<T> {
  fn id(&self) -> u32 {
    ApplicationWindow::window_id_to_u32(
      self
        .read()
        .expect("Window lock is poisoned")
        .tao_window
        .id(),
    )
  }

  fn set_title(&self, title: &str) {
    self
      .read()
      .expect("Window lock is poisoned")
      .tao_window
      .set_title(title);
  }

  fn show(&self) {
    self
      .read()
      .expect("Window lock is poisoned")
      .tao_window
      .set_visible(true);
  }

  fn hide(&self) {
    self
      .read()
      .expect("Window lock is poisoned")
      .tao_window
      .set_visible(false);
  }

  fn eval(&self, script: &str) {
    self
      .read()
      .expect("Window lock is poisoned")
      .wry_webview
      .evaluate_script(script)
      .expect("Failed to evaluate script");
  }

  fn emit(&self, event: &str, payload: serde_json::Value) {
    self
      .read()
      .expect("Window lock is poisoned")
      .app
      .event_loop_proxy
      .send_event(AppWindowEvent::Event {
        name: event.to_string(),
        payload,
        target: vec![self.id()],
      })
      .expect("Failed to send event");
  }

  fn app(&self) -> App<T> {
    self.read().expect("Window lock is poisoned").app.clone()
  }
}

pub type CustomProtocolHandler = dyn Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static;

pub struct AppWindowBuilder<T> {
  is_main: bool,
  app: App<T>,
  tao_window_builder: tao::window::WindowBuilder,
  url: Option<String>,
  custom_protocols: HashMap<String, Box<CustomProtocolHandler>>,
}

impl<T> AppWindowBuilder<T> {
  pub fn new(app: App<T>) -> Self {
    let tao_window_builder = tao::window::WindowBuilder::new();

    Self {
      is_main: false,
      app,
      tao_window_builder,
      url: None,
      custom_protocols: HashMap::new(),
    }
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

  pub fn main(mut self) -> Self {
    self.is_main = true;

    self
  }

  pub fn build(self, event_loop: &EventLoop<AppWindowEvent>) -> AppWindow<T> {
    let tao_window = Arc::new(
      self
        .tao_window_builder
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

    builder = builder.with_initialization_script(&format!(
      "Object.defineProperty(window, 'ID', {{ value: {}, writable: false, enumerable: true }});",
      ApplicationWindow::window_id_to_u32(tao_window.id())
    ));
    builder = builder.with_initialization_script(include_str!("./scripts/init.js"));

    if let Some(url) = self.url {
      builder = builder.with_url(&url);
    }

    for (name, handler) in self.custom_protocols {
      builder = builder.with_asynchronous_custom_protocol(name, handler);
    }
    #[cfg(debug_assertions)]
    {
      builder = builder.with_devtools(true);
    }

    let wry_webview = builder.build().expect("Failed to build webview");

    #[cfg(debug_assertions)]
    {
      wry_webview.open_devtools();
    }

    let window = Arc::new(RwLock::new(ApplicationWindow {
      tao_window: tao_window.clone(),
      wry_webview,
      app: self.app.clone(),
    }));

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
