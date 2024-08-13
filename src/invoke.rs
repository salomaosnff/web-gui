use std::sync::{Arc, RwLock};

use serde_json::json;
use wry::{http::Request, RequestAsyncResponder};

use crate::{
  app::{App, ApplicationExt},
  window::ApplicationWindow,
};

pub struct InvokeRequest {
  pub method: String,
  pub args: Vec<serde_json::Value>,
  pub window: Arc<RwLock<ApplicationWindow>>,
}

#[derive(serde::Serialize)]
pub enum InvokeResult {
  Ok(serde_json::Value),
  Err(String),
}

impl InvokeResult {
  pub fn is_ok(&self) -> bool {
    matches!(self, Self::Ok(_))
  }

  pub fn is_err(&self) -> bool {
    !self.is_ok()
  }
}

pub fn create_ipc_protocol<T: Send + Sync + 'static>(
  app: App<T>,
) -> impl Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static {
  move |request, responder| {
    let app = app.clone();

    tokio::task::spawn(async move {
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
      let builder = wry::http::response::Builder::new()
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", "application/json");

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
          let response = app.invoke(InvokeRequest {
            method,
            args,
            window: app
              .read()
              .expect("App lock is poisoned")
              .windows
              .get(&window_id)
              .unwrap()
              .clone(),
          });
          responder.respond(
            builder
              .status(200)
              .body::<Vec<u8>>(json!(response).to_string().into_bytes())
              .unwrap(),
          );
        }
        Err(err) => {
          responder.respond(
            builder
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
    });
  }
}
