use clap::*;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Command {
  Create {
    output: PathBuf,
    #[clap(required = true)]
    directory: PathBuf,
  },
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[clap(subcommand)]
  pub command: Command,
}

pub fn parse() -> Cli {
  Cli::parse()
}