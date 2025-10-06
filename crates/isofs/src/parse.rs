use IsoParseError::*;
use crate::spec;

#[derive(Debug, thiserror::Error)]
pub enum IsoParseError {
  #[error("Input too small, expected atleast {expected_atleast} bytes, but got {got} bytes when parsing {when_parsing}")]
  InputTooSmall {
    expected_atleast: usize,
    got: usize,
    when_parsing: &'static str,
  },
}

pub trait IsoParse: Sized {
  fn parse(input: &[u8]) -> Result<Self, IsoParseError>;
}

impl IsoParse for spec::PrimaryVolumeDescriptor {
  fn parse(inp: &[u8]) -> Result<Self, IsoParseError> {
    if inp.len() < 2048 {
      return Err(InputTooSmall {
        expected_atleast: 2048,
        got: inp.len(),
        when_parsing: "PrimaryVolumeDescriptor",
      });
    }

    todo!()
  }
}