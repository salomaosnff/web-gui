use serde::Serialize;
use serde_json::json;
use wry::{http::Request, RequestAsyncResponder};

use crate::app::{App, AppExt};

use super::window::AppWindow;

pub type InvokeHandler = dyn Fn(InvokeCommand, InvokeResponder) + Send + Sync + 'static;

pub struct InvokeCommand {
  pub app: App,
  pub method: String,
  pub args: Vec<serde_json::Value>,
  pub window: AppWindow,
}

pub struct InvokeResponder(RequestAsyncResponder);

impl InvokeResponder {
  pub fn reply(self, response: InvokeResult) {
    let Self(responder) = self;
    let builder = wry::http::response::Builder::new()
      .header("Access-Control-Allow-Origin", "*")
      .header("Access-Control-Allow-Methods", "POST, OPTIONS")
      .header("Access-Control-Expose-Headers", "X-Invoke-Result")
      .header(
        "X-Invoke-Result",
        if response.is_ok() { "Ok" } else { "Err" },
      );

    responder.respond(match response {
      InvokeResult::Ok(data) => match data {
        InvokeResultData::Json(data) => builder
          .header("Content-Type", "application/json")
          .body(data.to_string().into_bytes())
          .unwrap(),
        InvokeResultData::Binary(value) => builder
          .header("Content-Type", "application/octet-stream")
          .body(value)
          .unwrap(),
      },
      InvokeResult::Err(err) => builder
        .status(200)
        .header("Content-Type", "application/json")
        .body::<Vec<u8>>(json!(err).to_string().into_bytes())
        .unwrap(),
    })
  }
}

#[derive(serde::Serialize)]
pub enum InvokeResultData {
  Json(serde_json::Value),
  Binary(Vec<u8>),
}

#[derive(serde::Serialize)]
pub enum InvokeResult {
  Ok(InvokeResultData),
  Err(String),
}

unsafe impl Send for InvokeResult {}
unsafe impl Sync for InvokeResult {}

impl From<serde_json::Value> for InvokeResult {
  fn from(value: serde_json::Value) -> Self {
    Self::Ok(InvokeResultData::Json(value))
  }
}

impl From<Vec<u8>> for InvokeResult {
  fn from(value: Vec<u8>) -> Self {
    Self::Ok(InvokeResultData::Binary(value))
  }
}

impl<E: Into<String>> From<Result<serde_json::Value, E>> for InvokeResult {
  fn from(result: Result<serde_json::Value, E>) -> Self {
    match result {
      Ok(value) => Self::Ok(InvokeResultData::Json(value)),
      Err(err) => Self::Err(err.into()),
    }
  }
}

impl From<&str> for InvokeResult {
  fn from(value: &str) -> Self {
    Self::Err(value.into())
  }
}

impl<E: Into<String>> From<Result<Vec<u8>, E>> for InvokeResult {
  fn from(result: Result<Vec<u8>, E>) -> Self {
    match result {
      Ok(value) => Self::Ok(InvokeResultData::Binary(value)),
      Err(err) => Self::Err(err.into()),
    }
  }
}

impl<T: Serialize> From<Option<T>> for InvokeResult {
  fn from(value: Option<T>) -> Self {
    match value {
      Some(value) => Self::Ok(InvokeResultData::Json(serde_json::to_value(value).unwrap())),
      None => Self::Ok(InvokeResultData::Json(serde_json::Value::Null)),
    }
  }
}

impl From<()> for InvokeResult {
  fn from(_: ()) -> Self {
    Self::Ok(InvokeResultData::Json(serde_json::Value::Null))
  }
}

impl From<bool> for InvokeResult {
  fn from(value: bool) -> Self {
    Self::Ok(InvokeResultData::Json(serde_json::Value::Bool(value)))
  }
}

impl From<Vec<serde_json::Value>> for InvokeResult {
  fn from(value: Vec<serde_json::Value>) -> Self {
    Self::Ok(InvokeResultData::Json(value.into()))
  }
}

impl InvokeResult {
  pub fn is_ok(&self) -> bool {
    matches!(self, Self::Ok(_))
  }

  pub fn is_err(&self) -> bool {
    !self.is_ok()
  }

  pub fn json(data: serde_json::Value) -> Self {
    Self::Ok(InvokeResultData::Json(data))
  }

  pub fn binary(data: Vec<u8>) -> Self {
    Self::Ok(InvokeResultData::Binary(data))
  }

  pub fn error(message: &str) -> Self {
    Self::Err(message.into())
  }
}

pub fn create_ipc_protocol(app: App) -> impl Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static {
  move |request, responder| {
    let app = app.clone();

    let host = request.uri().host().unwrap().to_string();

    if host != "invoke" {
      return responder.respond(
        wry::http::response::Builder::new()
          .status(400)
          .body::<Vec<u8>>("Invalid host".into())
          .unwrap(),
      );
    }

    let method = request.uri().path().trim_start_matches('/').to_string();
    let builder = wry::http::response::Builder::new();

    let window_id: u32 = request
      .headers()
      .get("X-Window-Id")
      .unwrap()
      .to_str()
      .unwrap()
      .parse()
      .expect("Invalid window id");

    match serde_json::from_slice::<Vec<serde_json::Value>>(request.body()) {
      Ok(args) => {
        app.invoke(
          InvokeCommand {
            app: app.clone(),
            method,
            args,
            window: app.get_window(window_id).unwrap().clone(),
          },
          InvokeResponder(responder),
        );
      }
      Err(err) => {
        responder.respond(
          builder
            .header("Access-Control-Allow-Origin", "*")
            .status(400)
            .body::<Vec<u8>>(
              json!(InvokeResult::Err(err.to_string()))
                .to_string()
                .into_bytes(),
            )
            .unwrap(),
        );
      }
    };
  }
}

#[macro_export]
macro_rules! sync_handler {
  ($handler:expr) => {
    move |app, command| Box::pin(async move { $handler(app, command) })
  };
}

#[macro_export]
macro_rules! async_handler {
  ($handler:expr) => {
    move |app, command| Box::pin($handler(app, command))
  };
}

#[macro_export]
macro_rules! async_invoke_handlers {
  ($app:expr, {$($name:expr => $handler:expr),*}) => {
    $(
      $app.add_invoke_handler($name, |command, responder| {
        tokio::task::spawn(async move {
          let result = $handler(command).await;
          responder.reply(result.into());
        });
      });
    )*
  };
}

#[macro_export]
macro_rules! blocking_invoke_handlers {
  ($app:expr, {$($name:expr => $handler:expr),*}) => {
    $(
      $app.add_invoke_handler($name, |command, responder| {
        let result = $handler(command);
        responder.reply(result.into());
      });
    )*
  };
}
