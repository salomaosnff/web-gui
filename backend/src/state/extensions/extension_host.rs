use std::collections::{HashMap, HashSet};

use crate::app::app_paths;

use super::{extension_scanner::ExtensionsScanner, Extension};

pub struct ExtensionHost {
  extensions: HashMap<String, Extension>,
}

impl ExtensionHost {
  pub fn new() -> Self {
    Self {
      extensions: HashMap::new(),
    }
  }

  pub fn get_extension(&self, id: &str) -> Option<&Extension> {
    self.extensions.get(id)
  }

  pub fn search_extensions(&self) -> ExtensionsScanner {
    ExtensionsScanner::new(app_paths::extensions_search_paths().into_iter())
  }

  pub fn extensions_for_window_labels(&self, labels: &HashSet<String>) -> Vec<&Extension> {
    self
      .extensions
      .values()
      .filter(|extension| {
        extension
          .manifest()
          .activate_on
          .iter()
          .any(|label| labels.contains(label))
      })
      .collect()
  }

  pub fn has_extension(&self, id: &str) -> bool {
    self.extensions.contains_key(id)
  }

  pub fn add_extension(&mut self, extension: Extension) {
    self.extensions.insert(extension.id(), extension);
  }

  pub fn remove_extension(&mut self, id: &str) {
    self.extensions.remove(id);
  }
}
