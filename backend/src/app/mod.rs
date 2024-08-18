use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, RwLock},
};

use invoke::{create_ipc_protocol, InvokeCommand, InvokeHandler, InvokeResponder, InvokeResult};
use resources::create_static_protocol;
use tao::{
  event::Event,
  event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
};
use window::{AppWindow, AppWindowBuilder, AppWindowEvent, AppWindowExt, ApplicationWindow};

use crate::state::AppState;

pub mod app_paths;
pub mod invoke;
pub mod resources;
pub mod window;

pub struct Application<T> {
  pub state: RwLock<T>,
  pub event_loop_proxy: Arc<EventLoopProxy<AppWindowEvent>>,
  pub windows: RwLock<HashMap<u32, AppWindow>>,
  pub main_window_id: RwLock<Option<u32>>,
  pub static_protocol_folders: RwLock<HashMap<String, PathBuf>>,
  pub invoke_handlers: Arc<RwLock<HashMap<String, Arc<InvokeHandler>>>>,
  pub import_map: RwLock<HashMap<String, String>>,
}

pub type App = Arc<Application<AppState>>;

impl Application<AppState> {
  pub fn new(event_loop: &EventLoop<AppWindowEvent>, state: AppState) -> App {
    let event_loop_proxy = event_loop.create_proxy();
    let mut static_protocol_folders = HashMap::new();

    static_protocol_folders.insert("app".to_string(), app_paths::resources_dir().join("www"));
    static_protocol_folders.insert("esm".to_string(), app_paths::resources_dir().join("esm"));

    Arc::new(Self {
      event_loop_proxy: Arc::new(event_loop_proxy),
      windows: RwLock::new(HashMap::new()),
      invoke_handlers: Arc::new(RwLock::new(HashMap::new())),
      main_window_id: RwLock::new(None),
      static_protocol_folders: RwLock::new(static_protocol_folders),
      state: RwLock::new(state),
      import_map: RwLock::new(HashMap::new()),
    })
  }
}

pub trait AppExt {
  fn add_es_module(&self, name: &str, url: &str);
  fn add_invoke_handler<F>(&self, method: &str, handler: F)
  where
    F: Fn(InvokeCommand, InvokeResponder) + Send + Sync + 'static;
  fn remove_invoke_handler(&self, method: &str);
  fn emit(&self, name: &str, payload: serde_json::Value);
  fn build_window(&self) -> AppWindowBuilder;
  fn invoke(&self, command: InvokeCommand, responder: InvokeResponder);
  fn handle_event(
    &self,
    event: Event<'_, AppWindowEvent>,
    event_loop: &EventLoopWindowTarget<AppWindowEvent>,
    control_flow: &mut ControlFlow,
  );
  fn get_window(&self, window_id: u32) -> Option<AppWindow>;
  fn get_windows_by_label(&self, label: &str) -> Vec<AppWindow>;
}

impl AppExt for App {
  fn add_es_module(&self, name: &str, url: &str) {
    self
      .import_map
      .write()
      .expect("Failed to write import map")
      .insert(name.to_string(), url.to_string());
  }
  fn emit(&self, name: &str, payload: serde_json::Value) {
    let targets: Vec<u32> = self
      .windows
      .read()
      .expect("Failed to acquire lock on windows. This should never happen as the lock is poisoned")
      .keys()
      .cloned()
      .collect();

    self
      .event_loop_proxy
      .send_event(AppWindowEvent::Event {
        name: name.to_string(),
        payload,
        target: targets,
      })
      .expect("Failed to send event");
  }
  fn build_window(&self) -> AppWindowBuilder {
    AppWindowBuilder::new(self.clone())
      .with_protocol("lenz", create_static_protocol(self.clone()))
      .with_protocol("ipc", create_ipc_protocol(self.clone()))
  }

  fn invoke(&self, invoke_request: InvokeCommand, responder: InvokeResponder) {
    let method = invoke_request.method.clone();

    match self
      .invoke_handlers
      .read()
      .expect("Invoke handlers lock is poisoned")
      .get(&method)
    {
      Some(handler) => handler(invoke_request, responder),
      None => responder.reply(InvokeResult::error(&format!(
        "No handler found for method: {}",
        method
      ))),
    }
  }

  fn add_invoke_handler<F>(&self, method: &str, handler: F)
  where
    F: Fn(InvokeCommand, InvokeResponder) + Send + Sync + 'static,
  {
    self
      .invoke_handlers
      .write()
      .expect("Invoke handlers lock is poisoned")
      .insert(
        method.to_string(),
        Arc::new(move |command, responder| handler(command, responder)),
      );
  }

  fn remove_invoke_handler(&self, method: &str) {
    self
      .invoke_handlers
      .write()
      .expect("Invoke handlers lock is poisoned")
      .remove(method);
  }

  fn handle_event(
    &self,
    event: Event<'_, AppWindowEvent>,
    _event_loop: &EventLoopWindowTarget<AppWindowEvent>,
    control_flow: &mut ControlFlow,
  ) {
    *control_flow = ControlFlow::Wait;
    match event {
      Event::WindowEvent {
        window_id,
        event: tao::event::WindowEvent::CloseRequested,
        ..
      } => {
        let window_id = ApplicationWindow::window_id_to_u32(window_id);
        let mut windows = self
          .windows
          .write()
          .expect("Failed to acquire lock on windows");

        if *self.main_window_id.read().expect(
        "Failed to acquire lock on main window id. This should never happen as the lock is poisoned",
      ) == Some(window_id) {
        windows.clear();
      } else {
        windows.remove(&window_id);
      }

        if windows.is_empty() {
          *control_flow = ControlFlow::Exit;
        }
      }
      Event::UserEvent(AppWindowEvent::Event {
        name,
        payload,
        target,
      }) => {
        for window_id in target {
          if let Some(window) = self.get_window(window_id) {
            window.eval(&format!(
              "window.__dispatch({}, {});",
              serde_json::to_string(&name).unwrap(),
              serde_json::to_string(&payload).unwrap()
            ));
          }
        }
      }
      _ => {}
    }
  }

  fn get_window(&self, window_id: u32) -> Option<AppWindow> {
    self
      .windows
      .read()
      .expect("Failed to acquire lock on windows")
      .get(&window_id)
      .cloned()
  }

  fn get_windows_by_label(&self, label: &str) -> Vec<AppWindow> {
    self
      .windows
      .read()
      .expect("Failed to acquire lock on windows")
      .values()
      .filter(|window| window.has_label(label))
      .cloned()
      .collect()
  }
}

#[macro_export]
macro_rules! import_map {
  ($app:expr, {$($name:expr => $url:expr$(,)?)*}) => {{
    let app = $app.clone();
    let mut import_map = app.import_map.write().expect("Failed to write import map");

    $(
      import_map.insert($name.to_string(), $url.to_string());
    )*
  }};
}
