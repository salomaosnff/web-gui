use crate::app::App;

mod app;
mod dialog;
mod fs;
mod window;

pub fn apply(app: App) {
  app::apply(app.clone());
  fs::apply(app.clone());
  window::apply(app.clone());
  dialog::apply(app.clone());
}
