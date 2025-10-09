mod cli;

fn main() -> Result<(), isofs::error::Error> {
  use isofs::writer::*;

  pretty_env_logger::init();

  let mut writer = IsoWriter::new(IsoWriterOptions {
    joliet: true,
    sector_size: 2048,
  });

  writer.upsert_filesystem(
    Filesystem::capture("crates", "./crates")?,
    &OnFileConflict::Overwrite,
  )?;

  writer.finalize(std::fs::File::create("data/crates.iso")?)?;

  Ok(())
}

