use std::path::PathBuf;

use super::Extension;

pub struct ExtensionsScanner {
  search_paths: Box<dyn Iterator<Item = PathBuf>>,
}

unsafe impl Send for ExtensionsScanner {}
unsafe impl Sync for ExtensionsScanner {}

impl ExtensionsScanner {
  pub fn new<T: Iterator<Item = PathBuf> + 'static>(search_paths: T) -> Self {
    Self {
      search_paths: Box::new(search_paths),
    }
  }
}

impl Iterator for ExtensionsScanner {
  type Item = Extension;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let path = self.search_paths.next()?;

      if !path.exists() {
        eprintln!("{} does not exist", path.display());
        continue;
      }

      if !path.is_dir() {
        eprintln!("{} is not a directory", path.display());
        continue;
      }

      match std::fs::read_dir(&path) {
        Ok(entries) => {
          for entry in entries {
            match entry {
              Ok(entry) => {
                if !path.is_dir() {
                  continue;
                }

                match Extension::from_dir(&entry.path()) {
                  Ok(extension) => return Some(extension),
                  Err(err) => {
                    eprintln!(
                      "Failed to load extension at {} > {}",
                      entry.path().display(),
                      err
                    );
                    continue;
                  }
                }
              }
              Err(err) => {
                eprintln!(
                  "Failed to search for extensions in {} > {}",
                  path.display(),
                  err
                );
                continue;
              }
            }
          }
        }
        Err(err) => {
          eprintln!("Failed to read directory: {}", err);
        }
      }
    }
  }
}
