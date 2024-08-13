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

pub struct Application<T> {
  pub state: Arc<RwLock<T>>,
  pub event_loop_proxy: Arc<EventLoopProxy<AppWindowEvent>>,
  pub windows: HashMap<u32, AppWindow<T>>,
  pub main_window_id: Option<u32>,
  pub static_protocol_folders: HashMap<String, PathBuf>,
  pub invoke_handlers:
    HashMap<String, Arc<dyn Fn(App<T>, InvokeRequest<T>) -> InvokeResult + Send + Sync>>,
}

pub type App<T> = Arc<RwLock<Application<T>>>;

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

    Arc::new(RwLock::new(Self {
      event_loop_proxy: Arc::new(event_loop_proxy),
      windows: HashMap::new(),
      invoke_handlers: HashMap::new(),
      main_window_id: None,
      static_protocol_folders,
      state: Arc::new(RwLock::new(state)),
    }))
  }
}

pub trait ApplicationExt<T> {
  fn emit(&self, name: &str, payload: serde_json::Value);
  fn build_window(&self) -> AppWindowBuilder<T>;
  fn invoke(&self, invoke_request: InvokeRequest<T>) -> InvokeResult;
  fn handle_event(
    &self,
    event: Event<'_, AppWindowEvent>,
    event_loop: &EventLoopWindowTarget<AppWindowEvent>,
    control_flow: &mut ControlFlow,
  );
  fn add_invoke_handler<F>(&self, method: &str, handler: F)
  where
    F: Fn(App<T>, InvokeRequest<T>) -> InvokeResult + Send + Sync + 'static;
}

impl<T: Send + Sync + 'static> ApplicationExt<T> for App<T> {
  fn emit(&self, name: &str, payload: serde_json::Value) {
    let app = self.read().expect("App lock is poisoned");
    let targets: Vec<u32> = app.windows.keys().cloned().collect();

    app
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
    let app_rw = app.read().expect("App lock is poisoned");

    let method = invoke_request.method.clone();

    if let Some(handler) = app_rw.invoke_handlers.get(&method) {
      handler(app.clone(), invoke_request)
    } else {
      InvokeResult::Err("Method not found".to_string())
    }
  }

  fn add_invoke_handler<F>(&self, method: &str, handler: F)
  where
    F: Fn(App<T>, InvokeRequest<T>) -> InvokeResult + Send + Sync + 'static,
  {
    let mut app = self.write().expect("App lock is poisoned");

    app
      .invoke_handlers
      .insert(method.to_string(), Arc::new(handler));
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
        window_id, event, ..
      } => match event {
        tao::event::WindowEvent::CloseRequested => {
          let window_id = ApplicationWindow::window_id_to_u32(window_id);
          let mut app = self.write().expect("App lock is poisoned");

          if app.main_window_id == Some(window_id) {
            app.windows.clear();
          } else {
            app.windows.remove(&window_id);
          }

          if app.windows.is_empty() {
            *control_flow = ControlFlow::Exit;
          }
        }
        _ => (),
      },
      Event::UserEvent(event) => match event {
        AppWindowEvent::Event {
          name,
          payload,
          target,
        } => {
          let app = self.read().expect("App lock is poisoned");

          for window_id in target {
            if let Some(window) = app.windows.get(&window_id) {
              window.eval(&format!(
                "window.__.dispatch({}, {});",
                serde_json::to_string(&name).unwrap(),
                serde_json::to_string(&payload).unwrap()
              ));
            }
          }
        }
        _ => {}
      },
      _ => {}
    }
  }
}
