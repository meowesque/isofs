use clap::*;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Command {
  Create {
    output: PathBuf,
    #[clap(short = 'v', long, default_value = "ISO_VOLUME")]
    volume_id: String,
    #[clap(long)]
    publisher: Option<String>,
    #[clap(long)]
    preparer: Option<String>,
    #[clap(required = true)]
    files: Vec<PathBuf>,
    #[clap(long)]
    joliet: bool,
    #[clap(long)]
    rock_ridge: bool,
    // TODO(meowesque): Add El Torito support for bootable ISOs.
  },
  Extract {
    #[clap(short, long)]
    input: PathBuf,
    #[clap(short, long, default_value = ".")]
    output: PathBuf,
    // TODO(meowesque): Add more options for extracting specific files, etc.
  },
  List {
    #[clap(short, long)]
    input: PathBuf,
    #[clap(short, long)]
    verbose: bool,
  },
  Info {
    #[clap(short, long)]
    input: PathBuf,
  },
  Validate {
    #[clap(short, long)]
    input: PathBuf,
  },
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[clap(subcommand)]
  pub command: Command,
}
