use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("I/O error: {0}")]
  Io(#[from] std::io::Error),
  #[error("walkdir error: {0}")]
  WalkDir(#[from] walkdir::Error),
  #[error("Not a file: {0}")]
  NotAFile(PathBuf),
}
