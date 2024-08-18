use crate::app::{app_paths, App, AppExt};

pub fn apply(app: App) {
  let modules = std::fs::read_dir(app_paths::esm_dir()).expect("Failed to read esm directory");

  for module in modules {
    let path = module.expect("Failed to read module").path();
    let (name, path) = if path.is_dir() {
      (
        path
          .file_name()
          .expect("Failed to get file name for module directory")
          .to_string_lossy()
          .to_string(),
        path.join("index.mjs"),
      )
    } else {
      (
        path
          .file_name()
          .expect("Failed to get file name for module directory")
          .to_string_lossy()
          .replace(
            &path
              .extension()
              .expect("Failed to get extension")
              .to_str()
              .map(|ext| format!(".{}", ext))
              .unwrap(),
            "",
          ),
        path.clone(),
      )
    };

    app.add_es_module(
      &format!("lenz/{}", name),
      &format!("lenz://esm/{}", path.file_name().unwrap().to_str().unwrap()),
    );
  }
}
