mod cli;

fn main() -> Result<(), isofs::error::Error> {
  use isofs::writer::*;

  pretty_env_logger::init();

  let cli = cli::parse();

  match cli.command {
    cli::Command::Create { output, directory } => {
      let mut writer = IsoWriter::new(IsoWriterOptions::compatibility());

      writer.upsert_filesystem(
        Filesystem::capture("", &directory)?,
        &OnFileConflict::Overwrite,
      )?;

      writer.finalize(std::fs::File::create(output)?)?;
    }
  }

  Ok(())
}
