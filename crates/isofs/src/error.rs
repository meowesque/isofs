use std::path::PathBuf;

use crate::serialize::IsoSerializeError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("I/O error: {0}")]
  Io(#[from] std::io::Error),
  #[error("walkdir error: {0}")]
  WalkDir(#[from] walkdir::Error),
  #[error("Not a file: {0}")]
  NotAFile(PathBuf),
  #[error("ISO serialization error: {0}")]
  IsoSerialize(#[from] IsoSerializeError)
}
