//! High-level interface for building ISO 9660 filesystems with optional Joliet extensions.

use crate::{
  lba::LbaAllocator,
  serialize::IsoSerialize,
  spec::{self, Identifier},
};

use super::prelude::*;
use std::{
  collections::{hash_map, HashMap},
  path::{self, Path},
  rc::Rc,
};

type ArrayStringU255 = arraystring::ArrayString<arraystring::typenum::U255>;

struct PathTable {
  size: u32,
  records: Vec<spec::PathTableRecord>,
}

impl PathTable {
  fn build_from_filesystem(fs: &Filesystem) -> Option<Self> {
    fn aggregate(
      dir: &DirectoryEntry,
      parent_ix: u16,
      records: &mut Vec<spec::PathTableRecord>,
    ) -> Option<()> {
      let record = dir.directory_record();
      let ix = records.len() as u16 + 1; // 1-based index
      let record = spec::PathTableRecord {
        directory_identifier_length: dir.name.len() as u8,
        extended_attribute_record_length: 0,
        extent_location: dir.data_lba?,
        parent_directory_number: parent_ix,
        // TODO(meowesque): Handle different identifier types (e.g., Joliet).
        directory_identifier: spec::Identifier::standard_directory(&dir.name)?,
      };

      records.push(record);

      dir.dirs.values().for_each(|subdir| {
        aggregate(subdir, ix, records);
      });

      Some(())
    }

    let records = {
      let mut records = vec![];

      records.push(spec::PathTableRecord {
        directory_identifier_length: 1,
        extended_attribute_record_length: 0,
        extent_location: fs.root.data_lba?,
        parent_directory_number: 1,
        directory_identifier: spec::Identifier::root_directory(),
      });

      for dir in fs.root.dirs.values() {
        aggregate(dir, 1, &mut records);
      }

      records
    };

    let size = records.iter().map(|x| x.extent() as u32).sum();

    Some(Self { size, records })
  }

  /// Allocates LBA for Type L path table.
  fn allocate_type_l_lba(&self, allocator: &mut LbaAllocator) -> u32 {
    allocator.allocate(self.size)
  }

  /// Allocates LBA for Type M path table.
  fn allocate_type_m_lba(&self, allocator: &mut LbaAllocator) -> u32 {
    allocator.allocate(self.size)
  }
}

/// Represents the content of a file, either from the filesystem or in-memory.
#[derive(Debug)]
enum FileEntryContentInner {
  /// File backed by an actual file on the host filesystem.
  File {
    metadata: std::fs::Metadata,
    handle: std::fs::File,
  },
  /// File with content stored directly in memory.
  InMemory(Vec<u8>),
}

/// Represents the content of a file, either from the filesystem or in-memory.
#[derive(Debug, Clone)]
pub struct FileEntryContent(Rc<FileEntryContentInner>);

impl FileEntryContent {
  pub(crate) fn extent(&self) -> u64 {
    match &*self.0 {
      FileEntryContentInner::File { metadata, .. } => metadata.len(),
      FileEntryContentInner::InMemory(vec) => vec.len() as u64,
    }
  }
}

impl TryFrom<std::fs::File> for FileEntryContent {
  type Error = std::io::Error;

  fn try_from(file: std::fs::File) -> std::io::Result<FileEntryContent> {
    Ok(FileEntryContent(Rc::new(FileEntryContentInner::File {
      metadata: file.metadata()?,
      handle: file,
    })))
  }
}

impl From<Vec<u8>> for FileEntryContent {
  fn from(vec: Vec<u8>) -> Self {
    FileEntryContent(Rc::new(FileEntryContentInner::InMemory(vec)))
  }
}

/// Represents a file in the filesystem.
#[derive(Debug, Clone)]
pub struct FileEntry {
  /// LBA of the start of the file's data.
  data_lba: Option<u32>,
  name: ArrayStringU255,
  content: FileEntryContent,
}

impl FileEntry {
  pub(crate) fn directory_record(&self) -> spec::DirectoryRecord {
    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.data_lba.unwrap_or(0),
      data_length: self.content.extent() as u32,
      recording_date: todo!(),
      file_flags: todo!(),
      file_unit_size: todo!(),
      interleave_gap_size: todo!(),
      volume_sequence_number: todo!(),
      file_identifier_length: todo!(),
      file_identifier: todo!(),
    }
  }

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
    self.data_lba = Some(allocator.allocate(self.directory_record().data_length));
  }
}

/// Represents a directory in the filesystem, which can contain files and subdirectories.
#[derive(Debug, Default)]
pub struct DirectoryEntry {
  /// LBA of the start of the directory's data.
  data_lba: Option<u32>,
  name: ArrayStringU255,
  dirs: HashMap<ArrayStringU255, DirectoryEntry>,
  files: HashMap<ArrayStringU255, FileEntry>,
}

impl DirectoryEntry {
  /// Converts this directory entry into a root directory.
  pub fn into_root_directory(self) -> RootDirectory {
    RootDirectory {
      data_lba: self.data_lba,
      dirs: self.dirs,
      files: self.files,
    }
  }

  pub fn merge(&mut self, other: DirectoryEntry, on_file_conflict: &OnFileConflict) {
    for (name, dir) in other.dirs {
      match self.dirs.entry(name) {
        hash_map::Entry::Vacant(vacant) => {
          vacant.insert(dir);
        }
        hash_map::Entry::Occupied(mut occupied) => {
          occupied.get_mut().merge(dir, on_file_conflict);
        }
      }
    }

    for (name, file) in other.files {
      if let Some(existing) = self.files.get_mut(&name) {
        match on_file_conflict {
          OnFileConflict::Overwrite => {
            let _ = std::mem::replace(existing, file);
          }
          OnFileConflict::Ignore => {
            // Do nothing, keep existing file.
          }
          OnFileConflict::Handler(handler) => {
            // TODO(meowesque): Avoid clone for efficiency.
            let replacement = handler(existing.clone(), file);
            let _ = std::mem::replace(existing, replacement);
          }
        }
      } else {
        self.files.insert(name, file);
      }
    }
  }

  pub(crate) fn directory_record(&self) -> spec::DirectoryRecord {
    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.data_lba.unwrap_or(0),
      data_length: self
        .files
        .values()
        .map(|x| x.directory_record().extent() as u32)
        .sum::<u32>()
        + self
          .dirs
          .values()
          .map(|x| x.directory_record().extent() as u32)
          .sum::<u32>()
          // Plus 2 entries for `.` and `..`
          + (2 * 34),
      recording_date: todo!(),
      file_flags: todo!(),
      file_unit_size: todo!(),
      interleave_gap_size: todo!(),
      volume_sequence_number: todo!(),
      file_identifier_length: todo!(),
      file_identifier: todo!(),
    }
  }

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
    self.data_lba = Some(allocator.allocate(self.directory_record().data_length));

    for dir in self.dirs.values_mut() {
      dir.allocate_lbas(allocator);
    }

    for file in self.files.values_mut() {
      file.allocate_lbas(allocator);
    }
  }
}

/// Exactly like a [DirectoryEntry], but represents the root of the filesystem.
#[derive(Debug, Default)]
pub struct RootDirectory {
  /// LBA of the start of the root directory's data.
  data_lba: Option<u32>,
  dirs: HashMap<ArrayStringU255, DirectoryEntry>,
  files: HashMap<ArrayStringU255, FileEntry>,
}

impl RootDirectory {
  /// Creates a new root directory by scaffolding from the given path.
  pub fn scaffold(components: impl DoubleEndedIterator<Item = impl AsRef<str>>) -> Self {
    // TODO(meowesque): See if we can avoid using a DoubleEndedIterator here.

    let mut tail: Option<DirectoryEntry> = None;

    for part in components.rev() {
      tail = Some(match tail {
        None => DirectoryEntry {
          data_lba: None,
          name: ArrayStringU255::from(part.as_ref()),
          dirs: HashMap::new(),
          files: HashMap::new(),
        },
        Some(tail) => DirectoryEntry {
          data_lba: None,
          name: ArrayStringU255::from(part.as_ref()),
          dirs: HashMap::from([(ArrayStringU255::from(tail.name), tail)]),
          files: HashMap::new(),
        },
      });
    }

    tail
      .map(|tail| Self::from_directory(tail, false))
      .unwrap_or_default()
  }

  pub fn insert_file(
    &mut self,
    path: impl AsRef<Path>,
    content: FileEntryContent,
    on_file_conflict: &OnFileConflict,
  ) -> Result<()> {
    let path = path.as_ref();

    let Some(file_name) = path.file_name() else {
      return Err(Error::NotAFile(path.to_path_buf()));
    };

    let mut scaffold = path
      .parent()
      .map(|parent| {
        RootDirectory::scaffold(
          parent
            .components()
            .map(|comp| comp.as_os_str().to_string_lossy()),
        )
      })
      .unwrap_or_default();

    scaffold.files.insert(
      file_name.to_string_lossy().as_ref().into(),
      FileEntry {
        data_lba: None,
        name: file_name.to_string_lossy().as_ref().into(),
        content,
      },
    );

    // Since there's only file within the scaffold, we can
    // merge it, as there will only be at most one conflict.
    self.merge(scaffold, on_file_conflict);

    Ok(())
  }

  /// Creates a new root directory from a directory.
  ///
  /// * If `emplace` is true, the contents of `dir` will be placed at the root, otherwise `dir` will become a subdirectory.
  pub fn from_directory(dir: DirectoryEntry, emplace: bool) -> Self {
    match emplace {
      true => Self {
        data_lba: dir.data_lba,
        dirs: dir.dirs,
        files: dir.files,
      },
      false => Self {
        data_lba: None,
        dirs: HashMap::from([(dir.name.clone(), dir)]),
        files: HashMap::new(),
      },
    }
  }

  /// Merges another root directory into this one, resolving file conflicts according to `on_file_conflict`.
  pub fn merge(&mut self, other: RootDirectory, on_file_conflict: &OnFileConflict) {
    for (name, dir) in other.dirs {
      match self.dirs.entry(name) {
        hash_map::Entry::Vacant(vacant) => {
          vacant.insert(dir);
        }
        hash_map::Entry::Occupied(mut occupied) => {
          occupied.get_mut().merge(dir, on_file_conflict);
        }
      }
    }

    for (name, file) in other.files {
      if let Some(existing) = self.files.get_mut(&name) {
        match on_file_conflict {
          OnFileConflict::Overwrite => {
            let _ = std::mem::replace(existing, file);
          }
          OnFileConflict::Ignore => {
            // Do nothing, keep existing file.
          }
          OnFileConflict::Handler(handler) => {
            // TODO(meowesque): Avoid clone for efficiency.
            let replacement = handler(existing.clone(), file);
            let _ = std::mem::replace(existing, replacement);
          }
        }
      } else {
        self.files.insert(name, file);
      }
    }
  }

  pub(crate) fn root_directory_record(&self) -> spec::RootDirectoryRecord {
    spec::RootDirectoryRecord {
      extent_location: self.data_lba.unwrap_or(0),
      data_length: self
        .files
        .values()
        .map(|x| x.directory_record().extent() as u32)
        .sum::<u32>()
        + self
          .dirs
          .values()
          .map(|x| x.directory_record().extent() as u32)
          .sum::<u32>()
          // Plus 2 entries for `.` and `..`
          + (2 * 34),
      recording_date: todo!(),
      file_flags: todo!(),
      file_unit_size: todo!(),
      interleave_gap_size: todo!(),
      volume_sequence_number: todo!(),
    }
  }

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
    self.data_lba = Some(allocator.allocate(self.root_directory_record().data_length));

    for dir in self.dirs.values_mut() {
      dir.allocate_lbas(allocator);
    }

    for file in self.files.values_mut() {
      file.allocate_lbas(allocator);
    }
  }
}

/// Represents a generic filesystem to be included in the ISO image.
#[derive(Debug, Default)]
pub struct Filesystem {
  root: RootDirectory,
}

impl Filesystem {
  /// Captures the file or directory at `path` and inserts it into the filesystem at `destination`.
  /// * If `destination` is the root (i.e., `/`), the captured entry will become the root directory.
  pub fn capture(destination: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Self> {
    // TODO(meowesque): Handle destination properly.
    let mut root = RootDirectory::default();

    for entry in walkdir::WalkDir::new(path) {
      let entry = entry?;

      if entry.file_type().is_file() {
        let file = std::fs::File::open(entry.path())?;
        let content = FileEntryContent::try_from(file)?;

        root.insert_file(
          destination.as_ref().join(entry.path()),
          content,
          &OnFileConflict::Overwrite,
        )?;
      }
    }

    Ok(Self { root })
  }

  /// Merge another filesystem into this one, resolving file conflicts according to `on_file_conflict`.
  pub fn merge(&mut self, other: Filesystem, on_file_conflict: &OnFileConflict) -> Result<()> {
    self.root.merge(other.root, on_file_conflict);
    Ok(())
  }

  /// Inserts a file into the filesystem at the specified `path`.
  ///
  /// * If a file already exists at that path, the behavior is determined by `on_file_conflict`.
  pub fn insert_file(
    &mut self,
    path: impl AsRef<Path>,
    content: FileEntryContent,
    on_file_conflict: &OnFileConflict,
  ) -> Result<()> {
    self.root.insert_file(path, content, on_file_conflict)
  }

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator) {
    self.root.allocate_lbas(allocator);
  }
}

/// Behavior when a file is already present in the filesystem.
pub enum OnFileConflict {
  /// Replace the existing file with the new one.
  Overwrite,
  /// Ignore the new file and keep the existing one.
  Ignore,
  /// Custom handler that takes the existing and new file entries and returns the one to keep.
  Handler(Rc<dyn Fn(FileEntry, FileEntry) -> FileEntry>),
}

#[derive(Debug, Default)]
struct BootRecord {}

#[derive(Debug, Clone)]
pub struct IsoBuilderOptions {
  pub joliet: bool,
  pub sector_size: u32,
}

impl Default for IsoBuilderOptions {
  fn default() -> Self {
    Self {
      joliet: true,
      sector_size: 2048,
    }
  }
}

#[derive(Debug)]
pub struct IsoBuilder {
  options: IsoBuilderOptions,
  filesystem: Filesystem,
  boot_record: Option<BootRecord>,
}

impl IsoBuilder {
  pub fn new(options: IsoBuilderOptions) -> Self {
    Self {
      options,
      filesystem: Default::default(),
      boot_record: Default::default(),
    }
  }

  pub fn options(&self) -> &IsoBuilderOptions {
    &self.options
  }

  /// Inserts or updates the filesystem to be used in the ISO image.
  /// If a filesystem is already present, it will be merged according
  /// to the specified `on_file_conflict` behavior.
  pub fn upsert_filesystem(
    &mut self,
    filesystem: Filesystem,
    on_file_conflict: &OnFileConflict,
  ) -> Result<()> {
    self.filesystem.merge(filesystem, on_file_conflict)
  }

  /// Builds the ISO image according to the current configuration.
  pub fn build<W: std::io::Write + std::io::Seek>(&mut self, writer: W) -> Result<()> {
    // 1. Allocate LBAs for the main volume's filesystem.

    let mut lba_allocator = LbaAllocator::new(
      self.options.sector_size,
      // Descriptors start at LBA 16. System area is LBA 0..=15.
      16 /* System Area */
        + /* Primary Volume Descriptor */ 1
        + /* Supplementary Volume Descriptor */ self.options.joliet as u32
        + self.boot_record.is_some() as u32
        + /* Volume set terminator */ 1,
    );

    self.filesystem.allocate_lbas(&mut lba_allocator);

    // 2. Allocate LBAs for the path table(s).

    let path_table =
      PathTable::build_from_filesystem(&self.filesystem).expect("Failed to build path table");

    let path_table_type_l_lba = path_table.allocate_type_l_lba(&mut lba_allocator);
    let path_table_type_m_lba = path_table.allocate_type_m_lba(&mut lba_allocator);

    // 3. Write out the various volume descriptors.

    let primary_volume_descriptor = spec::PrimaryVolumeDescriptor {
      standard_identifier: spec::StandardIdentifier::Cd001,
      version: spec::VolumeDescriptorVersion::Standard,
      // TODO(meowesque): Allow configuration
      system_identifier: spec::Identifier::system_identifier("LINUX").unwrap(),
      // TODO(meowesque): Allow configuration
      volume_identifier: spec::Identifier::volume_identifier("ISOFS").unwrap(),
      // TODO(meowesque): Calculate actual size
      volume_space_size: 0,
      volume_set_size: 0,
      volume_sequence_number: 0,
      logical_block_size: self.options.sector_size as u16,
      path_table_size: path_table.size,
      type_l_path_table_location: path_table_type_l_lba,
      optional_type_l_path_table_location: path_table_type_l_lba,
      type_m_path_table_location: path_table_type_m_lba,
      optional_type_m_path_table_location: path_table_type_m_lba,
      root_directory_record: self.filesystem.root.root_directory_record(),
      volume_set_identifier: spec::Identifier::volume_set_identifier("ISOFS").unwrap(),
      publisher_identifier: spec::Identifier::publisher_identifier("ISOFS").unwrap(),
      data_preparer_identifier: spec::Identifier::data_preparer_identifier("ISOFS").unwrap(),
      application_identifier: spec::Identifier::application_identifier("ISOFS").unwrap(),
      copyright_file_identifier: spec::Identifier::copyright_file_identifier("ISOFS").unwrap(),
      abstract_file_identifier: spec::Identifier::abstract_file_identifier("ISOFS").unwrap(),
      bibliographic_file_identifier: spec::Identifier::bibliographic_file_identifier("ISOFS").unwrap(),
      creation_date: chrono::Utc::now().into(),
      modification_date: chrono::Utc::now().into(),
      expiration_date: chrono::Utc::now().into(),
      effective_date: chrono::Utc::now().into(),
      file_structure_version: spec::FileStructureVersion::Standard,
      application_use: [0; 512],
    };

    let supplementary_volume_descriptor = spec::SupplementaryVolumeDescriptor {
      standard_identifier: spec::StandardIdentifier::Cd001,
      version: spec::VolumeDescriptorVersion::Standard,
      volume_flags: spec::VolumeFlags::empty(),
      system_identifier: todo!(),
      volume_identifier: todo!(),
      volume_space_size: todo!(),
      escape_sequences: todo!(),
      volume_set_size: todo!(),
      volume_sequence_number: todo!(),
      logical_block_size: todo!(),
      path_table_size: todo!(),
      type_l_path_table_location: todo!(),
      optional_type_l_path_table_location: todo!(),
      type_m_path_table_location: todo!(),
      optional_type_m_path_table_location: todo!(),
      root_directory_record: todo!(),
      volume_set_identifier: todo!(),
      publisher_identifier: todo!(),
      data_preparer_identifier: todo!(),
      application_identifier: todo!(),
      copyright_file_identifier: todo!(),
      abstract_file_identifier: todo!(),
      bibliographic_file_identifier: todo!(),
      creation_date: todo!(),
      modification_date: todo!(),
      expiration_date: todo!(),
      effective_date: todo!(),
      file_structure_version: todo!(),
      application_use: todo!(),
    };

    todo!()
  }
}
