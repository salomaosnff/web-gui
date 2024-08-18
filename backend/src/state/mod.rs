mod extensions;

pub struct AppState {
  pub extension_host: extensions::ExtensionHost,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      extension_host: extensions::ExtensionHost::new(),
    }
  }
}
