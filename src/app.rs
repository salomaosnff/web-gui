use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, RwLock},
};

use tao::{
  event::Event,
  event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
};

use crate::{
  invoke::{create_ipc_protocol, InvokeRequest, InvokeResult},
  resources::create_static_protocol,
  window::{AppWindow, AppWindowBuilder, AppWindowEvent, AppWindowExt, ApplicationWindow},
};

pub type InvokeHandler<T> = Arc<dyn Fn(App<T>, InvokeRequest<T>) -> InvokeResult + Send + Sync>;

pub struct Application<T> {
  pub state: RwLock<T>,
  pub event_loop_proxy: Arc<EventLoopProxy<AppWindowEvent>>,
  pub windows: RwLock<HashMap<u32, AppWindow<T>>>,
  pub main_window_id: RwLock<Option<u32>>,
  pub static_protocol_folders: HashMap<String, PathBuf>,
  pub invoke_handlers: RwLock<HashMap<String, InvokeHandler<T>>>,
  pub import_map: RwLock<HashMap<String, String>>,
}

pub type App<T> = Arc<Application<T>>;

impl Application<()> {
  pub fn new(event_loop: &EventLoop<AppWindowEvent>) -> App<()> {
    Application::new_with_state(event_loop, ())
  }
}

impl<T> Application<T> {
  pub fn new_with_state(event_loop: &EventLoop<AppWindowEvent>, state: T) -> App<T> {
    let event_loop_proxy = event_loop.create_proxy();
    let mut static_protocol_folders = HashMap::new();

    static_protocol_folders.insert("app".to_string(), PathBuf::from("resources"));

    Arc::new(Self {
      event_loop_proxy: Arc::new(event_loop_proxy),
      windows: RwLock::new(HashMap::new()),
      invoke_handlers: RwLock::new(HashMap::new()),
      main_window_id: RwLock::new(None),
      static_protocol_folders,
      state: RwLock::new(state),
      import_map: RwLock::new(HashMap::new()),
    })
  }
}

pub trait AppExt<T> {
  fn add_es_module(&self, name: &str, url: &str);
  fn remove_invoke_handler(&self, method: &str);
  fn emit(&self, name: &str, payload: serde_json::Value);
  fn build_window(&self) -> AppWindowBuilder<T>;
  fn invoke(&self, invoke_request: InvokeRequest<T>) -> InvokeResult;
  fn handle_event(
    &self,
    event: Event<'_, AppWindowEvent>,
    event_loop: &EventLoopWindowTarget<AppWindowEvent>,
    control_flow: &mut ControlFlow,
  );
  fn get_window(&self, window_id: u32) -> Option<AppWindow<T>>;
  fn add_invoke_handler<F>(&self, method: &str, handler: F)
  where
    F: Fn(App<T>, InvokeRequest<T>) -> InvokeResult + Send + Sync + 'static;
}

impl<T: Send + Sync + 'static> AppExt<T> for App<T> {
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
  fn build_window(&self) -> AppWindowBuilder<T> {
    AppWindowBuilder::new(self.clone())
      .with_protocol("assets", create_static_protocol(self.clone()))
      .with_protocol("ipc", create_ipc_protocol(self.clone()))
  }

  fn invoke(&self, invoke_request: InvokeRequest<T>) -> InvokeResult {
    let app = self.clone();

    let method = invoke_request.method.clone();

    if let Some(handler) = self
      .invoke_handlers
      .read()
      .expect("Invoke handlers lock is poisoned")
      .get(&method)
    {
      handler(app.clone(), invoke_request)
    } else {
      InvokeResult::Err("Method not found".to_string())
    }
  }

  fn add_invoke_handler<F>(&self, method: &str, handler: F)
  where
    F: Fn(App<T>, InvokeRequest<T>) -> InvokeResult + Send + Sync + 'static,
  {
    self
      .invoke_handlers
      .write()
      .expect("Invoke handlers lock is poisoned")
      .insert(method.to_string(), Arc::new(handler));
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
              "window.__.dispatch({}, {});",
              serde_json::to_string(&name).unwrap(),
              serde_json::to_string(&payload).unwrap()
            ));
          }
        }
      }
      _ => {}
    }
  }

  fn get_window(&self, window_id: u32) -> Option<AppWindow<T>> {
    self
      .windows
      .read()
      .expect("Failed to acquire lock on windows")
      .get(&window_id)
      .cloned()
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
