use clap::Parser;

mod cli;

fn main() {
  use isofs::*;

  pretty_env_logger::init();

  let _ = dbg!(isofs::builder::Filesystem::capture("some_dir", "./crates"));
}
