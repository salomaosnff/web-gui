use wry::{
  http::{Request, StatusCode},
  RequestAsyncResponder,
};

use crate::app::App;

pub fn create_static_protocol<T: Send + Sync + 'static>(
  app: App<T>,
) -> impl Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static {
  move |request, responder| {
    let builder = wry::http::response::Builder::new().header("Access-Control-Allow-Origin", "*");
    let uri = request.uri();
    let host = if let Some(host) = uri.host() {
      host
    } else {
      return responder.respond(
        builder
          .status(StatusCode::BAD_REQUEST)
          .body::<Vec<u8>>("Host is required".into())
          .unwrap(),
      );
    };

    if !app.static_protocol_folders.contains_key(host) {
      return responder.respond(
        builder
          .status(StatusCode::BAD_REQUEST)
          .body::<Vec<u8>>("Invalid Host".into())
          .unwrap(),
      );
    }

    let folder = app.static_protocol_folders.get(host).unwrap();
    let mut path = folder.join(uri.path().trim_start_matches('/'));

    if path.is_dir() {
      path = path.join("index.html");
    }

    if !path.exists() {
      return responder.respond(
        builder
          .status(StatusCode::NOT_FOUND)
          .body::<Vec<u8>>("File not found".into())
          .unwrap(),
      );
    }

    let response = match std::fs::read(&path) {
      Ok(content) => builder
        .status(StatusCode::OK)
        .header(
          "Content-Type",
          mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string(),
        )
        .body(content)
        .unwrap(),
      Err(err) => builder
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(err.to_string().into())
        .unwrap(),
    };

    responder.respond(response);
  }
}
