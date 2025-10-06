use std::path::{Path, PathBuf};

use crate::{serialize::IsoSerialize, spec};

pub trait EntryLike {
  fn extent_lba(&self) -> Option<u32>;

  fn set_extent_lba(&mut self, lba: u32);

  fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension>;

  fn assign_extent_lba(&mut self, allocator: &mut super::lba::LbaAllocator) {
    let descriptor = self.descriptor();
    self.set_extent_lba(allocator.allocate(descriptor.data_length));
  }
}

pub trait DirectoryLike: EntryLike {
  fn entries_iter(&self) -> impl Iterator<Item = &Entry>;

  fn entries_mut(&mut self) -> &mut Vec<Entry>;

  fn find_mut(&mut self, name: &str) -> Option<&mut Entry> {
    self.entries_mut().iter_mut().find(|e| e.name() == name)
  }

  fn upsert(&mut self, entry: Entry) {
    match (self.find_mut(entry.name()), entry) {
      // If a file with the same name exists, replace it.
      (Some(Entry::File(dup)), Entry::File(file)) => {
        let _ = std::mem::replace(dup, file);
      }
      // If a directory with the same name exists, upsert entries.
      (Some(Entry::Directory(dup)), Entry::Directory(dir)) => {
        dir.entries.into_iter().for_each(|x| dup.upsert(x))
      }
      // Otherwise, just add the entry.
      (_, entry) => self.entries_mut().push(entry),
    }
  }

  fn assign_extent_lbas(&mut self, allocator: &mut super::lba::LbaAllocator) {
    self.set_extent_lba(allocator.allocate(self.descriptor().data_length));

    for entry in self.entries_mut() {
      entry.assign_extent_lba(allocator);

      if let Entry::Directory(dir) = entry {
        dir.assign_extent_lbas(allocator);
      }
    }
  }
}

#[derive(Debug)]
pub struct FileEntry {
  pub(crate) extent_lba: Option<u32>,
  name: String,
  metadata: std::fs::Metadata,
  pub(crate) handle: std::fs::File,
}

impl FileEntry {
  pub fn name(&self) -> &str {
    &self.name
  }
}

impl EntryLike for FileEntry {
  fn extent_lba(&self) -> Option<u32> {
    self.extent_lba
  }

  fn set_extent_lba(&mut self, lba: u32) {
    self.extent_lba = Some(lba);
  }

  fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension> {
    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.extent_lba.unwrap_or(0),
      data_length: self.metadata.len() as u32,
      // TODO(meowesque): Time handling?
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::empty(),
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 1,
      file_identifier_length: self.name.len() as u8,
      file_identifier: spec::FileIdentifier::from_bytes_truncated(self.name.as_bytes()),
    }
  }
}

impl FileEntry {
  pub fn new(name: String, source: impl AsRef<Path>) -> Result<Self, std::io::Error> {
    let handle = std::fs::File::open(source.as_ref())?;
    let metadata = handle.metadata()?;

    Ok(Self {
      extent_lba: None,
      // TODO(meowesque): Handle revision numbers properly.
      name: format!("{};1", name),
      metadata,
      handle,
    })
  }
}

#[derive(Debug)]
pub struct DirectoryEntry {
  extent_lba: Option<u32>,
  name: String,
  entries: Vec<Entry>,
}

impl EntryLike for DirectoryEntry {
  fn extent_lba(&self) -> Option<u32> {
    self.extent_lba
  }

  fn set_extent_lba(&mut self, lba: u32) {
    self.extent_lba = Some(lba);
  }

  fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension> {
    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.extent_lba.unwrap_or(0),
      // TODO(meowesque): This seems inefficient.
      data_length: self
        .entries
        .iter()
        .map(|e| e.descriptor().extent() as u32)
        .sum::<u32>()
        // . and .. entries
        + (34 * 2),
      // TODO(meowesque): Time handling?
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::DIRECTORY,
      file_unit_size: 0,
      interleave_gap_size: 0,
      // TODO(meowesque): Support multi-volume?
      volume_sequence_number: 1,
      file_identifier_length: self.name.len() as u8,
      file_identifier: spec::FileIdentifier::from_bytes_truncated(self.name.as_bytes()),
    }
  }
}

impl DirectoryLike for DirectoryEntry {
  fn entries_iter(&self) -> impl Iterator<Item = &Entry> {
    self.entries.iter()
  }

  fn entries_mut(&mut self) -> &mut Vec<Entry> {
    &mut self.entries
  }
}

#[derive(Debug)]
pub enum Entry {
  File(FileEntry),
  Directory(DirectoryEntry),
}

impl EntryLike for Entry {
  fn extent_lba(&self) -> Option<u32> {
    match self {
      Entry::File(x) => x.extent_lba(),
      Entry::Directory(x) => x.extent_lba(),
    }
  }

  fn set_extent_lba(&mut self, lba: u32) {
    match self {
      Entry::File(x) => x.set_extent_lba(lba),
      Entry::Directory(x) => x.set_extent_lba(lba),
    }
  }

  fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension> {
    match self {
      Entry::File(x) => x.descriptor(),
      Entry::Directory(x) => x.descriptor(),
    }
  }
}

impl Entry {
  pub fn name(&self) -> &str {
    match self {
      Entry::File(x) => &x.name,
      Entry::Directory(x) => &x.name,
    }
  }

  pub(crate) fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension> {
    match self {
      Entry::File(x) => x.descriptor(),
      Entry::Directory(x) => x.descriptor(),
    }
  }
}

#[derive(Default, Debug)]
pub struct RootDirectory {
  pub extent_lba: Option<u32>,
  pub entries: Vec<Entry>,
}

impl EntryLike for RootDirectory {
  fn extent_lba(&self) -> Option<u32> {
    self.extent_lba
  }

  fn set_extent_lba(&mut self, lba: u32) {
    self.extent_lba = Some(lba);
  }

  fn descriptor(&self) -> spec::DirectoryRecord<spec::NoExtension> {
    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.extent_lba.unwrap_or(0),
      // TODO(meowesque): This seems inefficient.
      data_length: self
        .entries
        .iter()
        .map(|e| e.descriptor().extent() as u32)
        .sum::<u32>()
        // . entry
        + 34,
      // TODO(meowesque): Time handling?
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::DIRECTORY,
      file_unit_size: 0,
      interleave_gap_size: 0,
      // TODO(meowesque): Support multi-volume?
      volume_sequence_number: 1,
      file_identifier_length: 1,
      file_identifier: spec::FileIdentifier::from_bytes_truncated(&[0]),
    }
  }
}

impl DirectoryLike for RootDirectory {
  fn entries_iter(&self) -> impl Iterator<Item = &Entry> {
    self.entries.iter()
  }

  fn entries_mut(&mut self) -> &mut Vec<Entry> {
    &mut self.entries
  }
}

impl RootDirectory {
  pub fn root_descriptor(&self) -> spec::RootDirectoryRecord {
    spec::RootDirectoryRecord {
      extent_location: self.extent_lba.unwrap_or(0),
      data_length: self
        .entries
        .iter()
        .map(|e| e.descriptor().extent() as u32)
        .sum(),
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::DIRECTORY,
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 1,
    }
  }
}

#[derive(Default, Debug)]
pub struct Filesystem {
  pub root: RootDirectory,
}

impl Filesystem {
  pub(crate) fn assign_extent_lbas(&mut self, allocator: &mut super::lba::LbaAllocator) {
    self.root.assign_extent_lbas(allocator);
  }

  pub fn upsert_file(
    &mut self,
    destination: impl AsRef<Path>,
    source: impl AsRef<Path>,
  ) -> Result<(), super::error::Error> {
    let destination = destination.as_ref();
    let components = destination.components();

    let mut tail = Entry::File(FileEntry::new(
      // TODO(meowesque): Handle error more gracefully.
      destination
        .file_name()
        .expect("Must have a filename")
        .to_string_lossy()
        .to_string(),
      source,
    )?);

    for component in components.rev().skip(1) {
      tail = Entry::Directory(DirectoryEntry {
        extent_lba: None,
        name: component.as_os_str().to_string_lossy().to_string(),
        entries: vec![tail],
      });
    }

    self.root.upsert(tail);

    Ok(())
  }
}
