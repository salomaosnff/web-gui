use std::path::PathBuf;

use crate::app::App;

use super::{ExtensionError, ExtensionManifest};

pub struct Extension {
  path: PathBuf,
  manifest: ExtensionManifest,
}

impl Extension {
  pub fn from_dir(path: &PathBuf) -> Result<Self, ExtensionError> {
    ExtensionManifest::from_path(path).map(|manifest| Extension {
      path: path.clone(),
      manifest,
    })
  }

  pub fn id(&self) -> String {
    self.manifest.id.clone()
  }

  pub fn public_url(&self) -> String {
    format!("lenz://{}", self.manifest.id)
  }

  pub fn manifest(&self) -> &ExtensionManifest {
    &self.manifest
  }

  pub fn dir(&self) -> &PathBuf {
    &self.path
  }

  pub fn main_script_url(&self) -> String {
    format!(
      "{}/{}",
      self.public_url(),
      self.manifest.main.trim_start_matches("/")
    )
  }

  pub fn activate(self, app: App) {
    if app
      .state
      .read()
      .unwrap()
      .extension_host
      .has_extension(&self.manifest.id)
    {
      println!("Extension {} already activated", self.id());
      return;
    }

    app
      .static_protocol_folders
      .write()
      .expect("Failed to acquire lock on static protocol folders")
      .insert(self.id(), self.path.clone());

    app
      .state
      .write()
      .unwrap()
      .extension_host
      .add_extension(self);
  }

  pub fn deactivate(self, app: App) {
    if !app
      .state
      .read()
      .unwrap()
      .extension_host
      .has_extension(&self.manifest.id)
    {
      println!("Extension {} already deactivated", self.id());
      return;
    }

    app
      .static_protocol_folders
      .write()
      .expect("Failed to acquire lock on static protocol folders")
      .remove(&self.id());

    app
      .state
      .write()
      .unwrap()
      .extension_host
      .remove_extension(&self.manifest.id);
  }
}
