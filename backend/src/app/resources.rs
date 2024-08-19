
use wry::{
  http::{Request, StatusCode},
  RequestAsyncResponder,
};

use crate::app::App;

pub fn create_static_protocol(
  app: App,
) -> impl Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static {
  move |request, responder| {
    let builder = wry::http::response::Builder::new().header("Access-Control-Allow-Origin", "*");
    let uri = request.uri();

    let (host, path) = if uri.host().unwrap_or_default() == "localhost" {
      let (host, path) = uri.path().trim_start_matches("/").split_once("/").unwrap_or_default();
      (host, path)
    } else { (uri.host().unwrap_or_default(), uri.path()) };

    if host.is_empty() {
      return responder.respond(
        builder
          .status(StatusCode::BAD_REQUEST)
          .body::<Vec<u8>>("Host is required".into())
          .unwrap(),
      );
    };

    let static_protocol_folders = app
      .static_protocol_folders
      .read()
      .expect("Failed to acquire lock on static protocol folders");

    if !static_protocol_folders.contains_key(host) {
      return responder.respond(
        builder
          .status(StatusCode::BAD_REQUEST)
          .body::<Vec<u8>>("Invalid Host".into())
          .unwrap(),
      );
    }

    let folder = static_protocol_folders.get(host).unwrap();
    let mut path = folder.join(path.trim_start_matches('/'));

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
      Ok(content) => {
        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
        builder
        .status(StatusCode::OK)
        .header(
          "Content-Type",
          mime_type.to_string(),
        )
        .body({
          if (mime_type == mime_guess::mime::TEXT_HTML || mime_type == mime_guess::mime::TEXT_HTML_UTF_8) {
            if let Ok(html) = String::from_utf8(content.clone()) {
              let import_map = serde_json::to_string_pretty(&{
                let mut import_map = app
                  .import_map
                  .read()
                  .expect("Failed to acquire lock on import map")
                  .clone();
    
                for (name, url) in app.import_map.read().expect("Failed to lock import_map").iter() {
                  import_map.insert(name.clone(), url.clone());
                }
    
                import_map
              })
              .expect("Failed to serialize import map");

            let custom_protocol = {
              #[cfg(target_os="windows")]
              {
                "`http://${protocol}.localhost/${url}`"
              }
    
              #[cfg(not(target_os="windows"))]
              {
                "${protocol}://${url}`"
              }
            };

              html
                .replace("</head>", &format!(r#"
<script type="importmap">
{{
  "imports": {import_map}
}}
</script>
<script>
  Object.defineProperty(window, 'CUSTOM_PROTOCOL', {{
    value: (protocol, url) => {custom_protocol}
  }})
</script>
</head>
                "#))
              .as_bytes().to_vec()
            } else {
              content
            }
          } else {
            content
          }
        })
        .unwrap()
      },
      Err(err) => builder
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(err.to_string().into())
        .unwrap(),
    };

    responder.respond(response);
  }
}


pub fn custom_protocol(scheme: impl Into<String>, url: impl Into<String>) -> String {
  #[cfg(target_os="windows")]
  {format!("http://{}.localhost/{}", scheme.into(), url.into())}
  
  #[cfg(not(target_os="windows"))]
  {format!("{}://{}", scheme.into(), url.into())}
}