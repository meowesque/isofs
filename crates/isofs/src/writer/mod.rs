use crate::{
  serialize::IsoSerialize,
  spec::{self, VolumeDescriptorSetTerminator},
  writer::volume::VolumeLike,
};

pub mod error;
pub mod fs;
pub mod lba;
pub mod sector;
pub mod volume;
pub mod path_table;

pub enum Standard {
  Iso9660,
}

impl Standard {
  fn standard_identifier(&self) -> spec::StandardIdentifier {
    match self {
      Standard::Iso9660 => spec::StandardIdentifier::Cd001,
    }
  }
}

pub struct WriterOptions {
  pub sector_size: u16,
  pub standard: Standard,
}

pub struct IsoWriter {
  options: WriterOptions,
  volumes: Vec<volume::Volume>,
}

impl IsoWriter {
  pub fn new(options: WriterOptions) -> Self {
    Self {
      options,
      volumes: vec![],
    }
  }

  pub fn add_volume(&mut self, volume: impl Into<volume::Volume>) {
    self.volumes.push(volume.into());
  }

  pub fn write<W>(&mut self, mut writer: W) -> Result<(), error::Error>
  where
    W: std::io::Write + std::io::Seek,
  {
    /// Write the contents of the file to it's allocated LBA
    fn write_file_entry<W>(
      writer: &mut W,
      file_entry: &fs::FileEntry,
      sector_size: u64,
    ) -> Result<(), error::Error>
    where
      W: std::io::Write + std::io::Seek,
    {
      log::info!(
        "Writing file content (ext. LBA {}): {}",
        file_entry.extent_lba.unwrap(),
        file_entry.name()
      );

      let mut reader = std::io::BufReader::new(&file_entry.handle);

      writer.seek(std::io::SeekFrom::Start(
        file_entry.extent_lba.unwrap() as u64 * sector_size,
      ))?;

      std::io::copy(&mut reader, &mut *writer)?;

      Ok(())
    }

    fn write_directory_entry<W, D>(
      writer: &mut W,
      parent_directory_entry_descriptor: Option<spec::DirectoryRecord<spec::NoExtension>>,
      directory_entry: &D,
      sector_size: u64,
    ) -> Result<(), error::Error>
    where
      W: std::io::Write + std::io::Seek,
      D: fs::DirectoryLike + fs::EntryLike,
    {
      let mut sector_writer = sector::SectorWriter::new(
        &mut *writer,
        directory_entry.extent_lba().unwrap() as u64,
        sector_size,
      );

      let directory_descriptor = directory_entry.descriptor();

      log::info!(
        "Writing directory entry (ext. LBA {}): {}",
        &directory_descriptor.extent_location,
        std::str::from_utf8(
          &directory_descriptor.file_identifier.0[..directory_descriptor.file_identifier.extent()]
        )
        .unwrap()
      );

      // TODO(meowesque): Write . and .. entries.

      let mut byte_buf = vec![];

      let dot_entry = spec::DirectoryRecord::<spec::NoExtension> {
        extended_attribute_length: 0,
        extent_location: directory_descriptor.extent_location,
        data_length: directory_descriptor.data_length,
        recording_date: directory_descriptor.recording_date.clone(),
        file_flags: spec::FileFlags::DIRECTORY,
        file_unit_size: 0,
        interleave_gap_size: 0,
        volume_sequence_number: 1,
        file_identifier_length: 1,
        file_identifier: spec::FileIdentifier::from_bytes_truncated(&[0u8; 1]),
      };

      byte_buf.resize(dot_entry.extent(), 0);
      dot_entry.serialize(&mut byte_buf[..])?;
      sector_writer.write_aligned(&byte_buf[..dot_entry.extent() as usize])?;

      // No .. entry for the root directory.
      if let Some(parent_directory_entry_descriptor) = parent_directory_entry_descriptor {
        let dotdot_entry = spec::DirectoryRecord::<spec::NoExtension> {
          extended_attribute_length: 0,
          extent_location: parent_directory_entry_descriptor.extent_location,
          data_length: parent_directory_entry_descriptor.data_length,
          recording_date: parent_directory_entry_descriptor.recording_date.clone(),
          file_flags: spec::FileFlags::DIRECTORY,
          file_unit_size: 0,
          interleave_gap_size: 0,
          volume_sequence_number: 1,
          file_identifier_length: 1,
          file_identifier: spec::FileIdentifier::from_bytes_truncated(&[1u8; 1]),
        };

        byte_buf.resize(dotdot_entry.extent(), 0);
        dotdot_entry.serialize(&mut byte_buf[..])?;
        sector_writer.write_aligned(&byte_buf[..dotdot_entry.extent() as usize])?;
      }

      for entry in directory_entry.entries_iter() {
        let entry_descriptor = entry.descriptor();

        log::info!(
          "Writing entry (ext. LBA {}): {}",
          entry_descriptor.extent_location,
          entry.name()
        );

        byte_buf.resize(entry_descriptor.extent(), 0);
        entry_descriptor.serialize(&mut byte_buf[..])?;

        sector_writer.write_aligned(&byte_buf[..entry_descriptor.extent() as usize])?;
      }

      for entry in directory_entry.entries_iter() {
        write_entry(
          &mut *writer,
          Some(directory_entry.descriptor()),
          entry,
          sector_size,
        )?;
      }

      Ok(())
    }

    fn write_entry<W>(
      writer: &mut W,
      parent_directory_entry_descriptor: Option<spec::DirectoryRecord<spec::NoExtension>>,
      entry: &fs::Entry,
      sector_size: u64,
    ) -> Result<(), error::Error>
    where
      W: std::io::Write + std::io::Seek,
    {
      match entry {
        fs::Entry::File(file_entry) => write_file_entry(&mut *writer, file_entry, sector_size),
        fs::Entry::Directory(dir_entry) => write_directory_entry(
          &mut *writer,
          parent_directory_entry_descriptor,
          dir_entry,
          sector_size,
        ),
      }
    }

    let mut allocator = lba::LbaAllocator::new(
      self.options.sector_size as u32,
      /* System use */ 16 + self.volumes.len() as u32 + /* Set terminator */ 1,
    );

    let context = volume::VolumeContext {
      sector_size: self.options.sector_size as u32,
      standard_identifier: self.options.standard.standard_identifier(),
    };

    {
      let mut bytes: [u8; 2048] = [0; 2048];

      writer.seek(std::io::SeekFrom::Start(16 * 2048))?;

      for volume in self.volumes.iter_mut() {
        match volume {
          volume::Volume::Primary(pv) => {
            pv.filesystem.assign_extent_lbas(&mut allocator);
            pv.descriptor(&context).serialize(&mut bytes)?;
            writer.write_all(&bytes)?;
            write_directory_entry(
              &mut writer,
              None,
              &pv.filesystem.root,
              context.sector_size as u64,
            )?;
          }
        }
      }

      writer.seek(std::io::SeekFrom::Start(
        (self.volumes.len() as u64 + 16) * self.options.sector_size as u64,
      ))?;

      spec::VolumeDescriptorSetTerminator.serialize(&mut bytes)?;

      writer.write_all(&bytes)?;
    }

    Ok(())
  }
}
