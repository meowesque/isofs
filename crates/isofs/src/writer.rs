//! High-level interface for building ISO 9660 filesystems with optional Joliet extensions.

use crate::{
  serialize::{self, IsoSerialize},
  spec::{self, EscapeSequences, Identifier},
};

use super::prelude::*;
use std::{
  collections::{hash_map, HashMap},
  path::{self, Path},
  rc::Rc,
};

type ArrayStringU255 = arraystring::ArrayString<arraystring::typenum::U255>;

struct Context {
  compatibility_mode: spec::CompatibilityMode,
}

struct SectorWriter<Storage> {
  storage: Storage,
  sector_ix: u64,
  sector_size: u64,
  bytes_offset: u64,
}

impl<Storage> SectorWriter<Storage>
where
  Storage: std::io::Write + std::io::Seek,
{
  fn new(storage: Storage, sector_offset: u64, sector_size: u64) -> Self {
    Self {
      storage,
      sector_ix: sector_offset,
      sector_size,
      bytes_offset: 0,
    }
  }

  /// Write data to the current sector, padding with zeros if necessary.
  ///
  /// If the buffer is larger than the sector size, it will be truncated.
  fn write_aligned(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let buf = &buf[..buf.len().min(self.sector_size as usize)];

    if self.bytes_offset + buf.len() as u64 > self.sector_size {
      // Move to the next sector if we don't have enough space.

      self.sector_ix += 1;
      self.bytes_offset = 0;

      self
        .storage
        .seek(std::io::SeekFrom::Start(self.sector_ix * self.sector_size))?;
    } else {
      self.storage.seek(std::io::SeekFrom::Start(
        self.sector_ix * self.sector_size + self.bytes_offset,
      ))?;
    }

    log::info!(
      "Writing {} bytes at sector {}, offset {}",
      buf.len(),
      self.sector_ix,
      self.bytes_offset
    );

    let written = self.storage.write(buf)?;

    self.bytes_offset += written as u64;

    Ok(written)
  }
}

struct LbaAllocator {
  sector_size: u32,
  next_lba: u32,
}

impl LbaAllocator {
  fn new(sector_size: u32, offset: u32) -> Self {
    Self {
      sector_size,
      next_lba: offset,
    }
  }

  fn allocate(&mut self, size: u32) -> u32 {
    let lba = self.next_lba;
    let sectors = (size + self.sector_size - 1) / self.sector_size;
    self.next_lba += sectors;
    lba
  }
}

struct PathTable {
  size: u32,
  records: Vec<spec::PathTableRecord>,
}

impl PathTable {
  fn build_from_filesystem(fs: &Filesystem, context: &Context) -> Option<Self> {
    fn aggregate(
      dir: &DirectoryEntry,
      parent_ix: u16,
      records: &mut Vec<spec::PathTableRecord>,
    ) -> Option<()> {
      let ix = records.len() as u16 + 1; // 1-based index

      let record = spec::PathTableRecord {
        directory_identifier_length: dir.name.len() as u8,
        extended_attribute_record_length: 0,
        extent_location: dir.data_lba?,
        parent_directory_number: parent_ix,
        // TODO(meowesque): Handle different identifier types (e.g., Joliet).
        directory_identifier: spec::Identifier::standard_directory_identifier(&dir.name)?,
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
  pub(crate) fn directory_record(&self, context: &Context) -> spec::DirectoryRecord {
    let file_identifier = spec::Identifier::standard_file_identifier(&self.name)
      // TODO(meowesque): Handle different identifier types (e.g., Joliet).
      .expect("File name should be valid");

    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.data_lba.unwrap_or(0),
      data_length: self.content.extent() as u32,
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::empty(),
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 0,
      file_identifier_length: file_identifier.extent() as u8,
      // TODO(meowesque): Handle different identifier types (e.g., Joliet).
      file_identifier,
    }
  }

  fn allocate_lbas(&mut self, allocator: &mut LbaAllocator, context: &Context) {
    self.data_lba = Some(allocator.allocate(self.directory_record(context).data_length));
  }
}

/// Represents a directory in the filesystem, which can contain files and subdirectories.
#[derive(Debug, Default, Clone)]
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

  pub(crate) fn directory_record(&self, context: &Context) -> spec::DirectoryRecord {
    let file_identifier = spec::Identifier::standard_directory_identifier(&self.name)
      // TODO(meowesque): Handle different identifier types (e.g., Joliet).
      .expect("Directory name should be valid");

    spec::DirectoryRecord {
      extended_attribute_length: 0,
      extent_location: self.data_lba.unwrap_or(0),
      data_length: self
        .files
        .values()
        .map(|x| x.directory_record(context).extent() as u32)
        .sum::<u32>()
        + self
          .dirs
          .values()
          .map(|x| x.directory_record(context).extent() as u32)
          .sum::<u32>()
          // Plus 2 entries for `.` and `..`
          + (2 * 34),
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::DIRECTORY,
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 0,
      file_identifier_length: file_identifier.extent() as u8,
      // TODO(meowesque): Handle different identifier types (e.g., Joliet).
      file_identifier,
    }
  }

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator, context: &Context) {
    self.data_lba = Some(allocator.allocate(self.directory_record(context).data_length));

    for dir in self.dirs.values_mut() {
      dir.allocate_lbas(allocator, context);
    }

    for file in self.files.values_mut() {
      file.allocate_lbas(allocator, context);
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
  pub fn scaffold(
    components: impl DoubleEndedIterator<Item = impl AsRef<str>>,
    dirs: HashMap<ArrayStringU255, DirectoryEntry>,
    files: HashMap<ArrayStringU255, FileEntry>,
  ) -> Self {
    // TODO(meowesque): See if we can avoid using a DoubleEndedIterator here.

    let mut tail: Option<DirectoryEntry> = None;

    for part in components.rev() {
      tail = Some(match tail {
        None => DirectoryEntry {
          data_lba: None,
          name: ArrayStringU255::from(part.as_ref()),
          // TODO(meowesque): Avoid clone for efficiency.
          dirs: dirs.clone(),
          files: files.clone(),
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
          HashMap::new(),
          HashMap::from([(
            file_name.to_string_lossy().as_ref().into(),
            FileEntry {
              data_lba: None,
              name: file_name.to_string_lossy().as_ref().into(),
              content,
            },
          )]),
        )
      })
      .unwrap_or_default();

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

  pub(crate) fn root_directory_record(&self, context: &Context) -> spec::RootDirectoryRecord {
    spec::RootDirectoryRecord {
      extent_location: self.data_lba.unwrap_or(0),
      data_length: self
        .files
        .values()
        .map(|x| x.directory_record(context).extent() as u32)
        .sum::<u32>()
        + self
          .dirs
          .values()
          .map(|x| x.directory_record(context).extent() as u32)
          .sum::<u32>()
          // Plus 2 entries for `.` and `..`
          + (2 * 34),
      recording_date: chrono::Utc::now().into(),
      file_flags: spec::FileFlags::DIRECTORY,
      file_unit_size: 0,
      interleave_gap_size: 0,
      volume_sequence_number: 0,
    }
  }

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator, context: &Context) {
    self.data_lba = Some(allocator.allocate(self.root_directory_record(context).data_length));

    for dir in self.dirs.values_mut() {
      dir.allocate_lbas(allocator, context);
    }

    for file in self.files.values_mut() {
      file.allocate_lbas(allocator, context);
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

    for entry in walkdir::WalkDir::new(&path) {
      let entry = entry?;

      if entry.file_type().is_file() {
        let file = std::fs::File::open(entry.path())?;
        let content = FileEntryContent::try_from(file)?;

        root.insert_file(
          destination
            .as_ref()
            .join(entry.path().strip_prefix(path.as_ref()).unwrap()),
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

  pub(crate) fn allocate_lbas(&mut self, allocator: &mut LbaAllocator, context: &Context) {
    self.root.allocate_lbas(allocator, context);
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
pub struct IsoWriterOptions {
  pub joliet: bool,
  pub sector_size: u32,
}

impl IsoWriterOptions {
  pub fn compatibility() -> Self {
    Self {
      joliet: false,
      sector_size: 2048,
    }
  }
}

impl Default for IsoWriterOptions {
  fn default() -> Self {
    Self {
      joliet: true,
      sector_size: 2048,
    }
  }
}

#[derive(Debug)]
pub struct IsoWriter {
  options: IsoWriterOptions,
  filesystem: Filesystem,
  boot_record: Option<BootRecord>,
}

impl IsoWriter {
  pub fn new(options: IsoWriterOptions) -> Self {
    Self {
      options,
      filesystem: Default::default(),
      boot_record: Default::default(),
    }
  }

  pub fn options(&self) -> &IsoWriterOptions {
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
  pub fn finalize<W: std::io::Write + std::io::Seek>(mut self, mut writer: W) -> Result<()> {
    let context = Context {
      compatibility_mode: if self.options.joliet {
        spec::CompatibilityMode::Joliet(spec::JolietLevel::Level3)
      } else {
        spec::CompatibilityMode::Standard
      },
    };

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

    self.filesystem.allocate_lbas(&mut lba_allocator, &context);

    // 2. Allocate LBAs for the path table(s).

    let path_table = PathTable::build_from_filesystem(&self.filesystem, &context)
      .expect("Failed to build path table");

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
      volume_space_size: lba_allocator.next_lba,
      volume_set_size: 0,
      volume_sequence_number: 0,
      logical_block_size: self.options.sector_size as u16,
      path_table_size: path_table.size,
      type_l_path_table_location: path_table_type_l_lba,
      optional_type_l_path_table_location: path_table_type_l_lba,
      type_m_path_table_location: path_table_type_m_lba,
      optional_type_m_path_table_location: path_table_type_m_lba,
      root_directory_record: self.filesystem.root.root_directory_record(&context),
      volume_set_identifier: spec::Identifier::volume_set_identifier("ISOFS").unwrap(),
      publisher_identifier: spec::Identifier::publisher_identifier("ISOFS").unwrap(),
      data_preparer_identifier: spec::Identifier::data_preparer_identifier("ISOFS").unwrap(),
      application_identifier: spec::Identifier::application_identifier("ISOFS").unwrap(),
      copyright_file_identifier: spec::Identifier::copyright_file_identifier("ISOFS").unwrap(),
      abstract_file_identifier: spec::Identifier::abstract_file_identifier("ISOFS").unwrap(),
      bibliographic_file_identifier: spec::Identifier::bibliographic_file_identifier("ISOFS")
        .unwrap(),
      creation_date: chrono::Utc::now().into(),
      modification_date: chrono::Utc::now().into(),
      expiration_date: chrono::Utc::now().into(),
      effective_date: chrono::Utc::now().into(),
      file_structure_version: spec::FileStructureVersion::Standard,
      application_use: [0; 512],
    };

    /*
       let supplementary_volume_descriptor = spec::SupplementaryVolumeDescriptor {
         standard_identifier: spec::StandardIdentifier::Cd001,
         version: spec::VolumeDescriptorVersion::Standard,
         volume_flags: spec::VolumeFlags::empty(),
         system_identifier: spec::Identifier::system_identifier("LINUX").unwrap(),
         volume_identifier: spec::Identifier::volume_identifier("ISOFS").unwrap(),
         volume_space_size: 0,
         escape_sequences: EscapeSequences::joliet_level_3(),
         volume_set_size: 0,
         volume_sequence_number: 0,
         logical_block_size: self.options.sector_size as u16,
         path_table_size: path_table.joliet_size,
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
    */

    // TODO(meowesque): Add supplementary volume descriptor if Joliet is enabled.

    {
      let mut descriptor_bytes = [0u8; 2048];

      // 3.1. Write Primary Volume Descriptor

      primary_volume_descriptor.serialize(&mut (), &mut descriptor_bytes)?;
      writer.seek(std::io::SeekFrom::Start(
        16 * self.options.sector_size as u64,
      ))?;
      writer.write_all(&descriptor_bytes)?;

      // 3.2. Write Supplementary Volume Descriptor (if Joliet is enabled)

      if self.options.joliet {
        // TODO(meowesque)
      }

      // 3.3. Write Boot Record (if present)

      if let Some(_boot_record) = &self.boot_record {
        // TODO(meowesque)
      }

      // 3.4. Write Volume Descriptor Set Terminator

      spec::VolumeDescriptorSetTerminator.serialize(&mut (), &mut descriptor_bytes)?;
      writer.seek(std::io::SeekFrom::Start(
        (16 + 1 + self.options.joliet as u64 + self.boot_record.is_some() as u64)
          * self.options.sector_size as u64,
      ))?;
      writer.write_all(&descriptor_bytes)?;
    }

    // 4. Write Path Table(s)

    // 4.1 Write Type L Path Table

    {
      let mut record_bytes = vec![];
      let mut sector_writer = SectorWriter::new(
        &mut writer,
        path_table_type_l_lba as u64,
        self.options.sector_size as u64,
      );

      for record in &path_table.records {
        log::debug!("Writing (type L) path table record: {:?}", record);

        record_bytes.resize(record.extent(), 0);
        record.serialize(&mut serialize::Endianness::Little, &mut record_bytes)?;
        sector_writer.write_aligned(&record_bytes)?;
        record_bytes.clear();
      }
    }

    // 4.2 Write Type M Path Table

    {
      let mut record_bytes = vec![];
      let mut sector_writer = SectorWriter::new(
        &mut writer,
        path_table_type_m_lba as u64,
        self.options.sector_size as u64,
      );

      for record in &path_table.records {
        log::debug!("Writing (type M) path table record: {:?}", record);

        record_bytes.resize(record.extent(), 0);
        record.serialize(&mut serialize::Endianness::Big, &mut record_bytes)?;
        sector_writer.write_aligned(&record_bytes)?;
        record_bytes.clear();
      }
    }

    // 5. Write Directory Records and File Data

    fn write_file_entry<W: std::io::Write + std::io::Seek>(
      writer: &mut W,
      file: &FileEntry,
      options: &IsoWriterOptions,
    ) -> Result<()> {
      let Some(lba) = file.data_lba else {
        unreachable!("File LBA should have been allocated by now");
      };

      log::debug!("Writing file content: {:?}", file);

      writer.seek(std::io::SeekFrom::Start(
        lba as u64 * options.sector_size as u64,
      ))?;

      match file.content.0.as_ref() {
        FileEntryContentInner::File { handle, .. } => {
          std::io::copy(&mut std::io::BufReader::new(handle), writer)?;
        }
        FileEntryContentInner::InMemory(data) => writer.write_all(data)?,
      }

      Ok(())
    }

    fn write_directory_entry<W: std::io::Write + std::io::Seek>(
      writer: &mut W,
      parent_dir_extent_location: u32,
      parent_dir_data_length: u32,
      dir: &DirectoryEntry,
      options: &IsoWriterOptions,
      context: &Context,
    ) -> Result<()> {
      let Some(lba) = dir.data_lba else {
        unreachable!("Directory LBA should have been allocated by now");
      };
      let mut sector_writer =
        SectorWriter::new(&mut *writer, lba as u64, options.sector_size as u64);
      let dir_record = dir.directory_record(context);
      let mut buf = vec![];

      log::debug!("Writing directory record: {:?}", dir_record);

      {
        let dot_entry = spec::DirectoryRecord {
          extended_attribute_length: 0,
          extent_location: lba,
          data_length: dir_record.data_length,
          recording_date: chrono::Utc::now().into(),
          file_flags: spec::FileFlags::DIRECTORY,
          file_unit_size: 0,
          interleave_gap_size: 0,
          volume_sequence_number: 1,
          file_identifier_length: 1,
          file_identifier: spec::Identifier::current_directory(),
        };

        buf.resize(dot_entry.extent(), 0);
        dot_entry.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      {
        let dotdot_entry = spec::DirectoryRecord {
          extended_attribute_length: 0,
          extent_location: parent_dir_extent_location,
          data_length: parent_dir_data_length,
          recording_date: chrono::Utc::now().into(),
          file_flags: spec::FileFlags::DIRECTORY,
          file_unit_size: 0,
          interleave_gap_size: 0,
          volume_sequence_number: 1,
          file_identifier_length: 1,
          file_identifier: spec::Identifier::parent_directory(),
        };

        buf.resize(dotdot_entry.extent(), 0);
        dotdot_entry.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      for subdir in dir.dirs.values() {
        let subdir_record = subdir.directory_record(context);

        log::debug!("Writing directory record: {:?}", subdir_record);

        buf.resize(subdir_record.extent(), 0);
        subdir_record.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      for file in dir.files.values() {
        let file_record = file.directory_record(context);

        log::debug!("Writing file record: {:?}", file_record);

        buf.resize(file_record.extent(), 0);
        file_record.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      for subdir in dir.dirs.values() {
        write_directory_entry(
          &mut *writer,
          lba,
          dir_record.data_length,
          subdir,
          options,
          context,
        )?;
      }

      for file in dir.files.values() {
        write_file_entry(&mut *writer, file, options)?;
      }

      Ok(())
    }

    fn write_root_directory<W: std::io::Write + std::io::Seek>(
      writer: &mut W,
      root: &RootDirectory,
      options: &IsoWriterOptions,
      context: &Context,
    ) -> Result<()> {
      let Some(lba) = root.data_lba else {
        unreachable!("Directory LBA should have been allocated by now");
      };
      let mut sector_writer =
        SectorWriter::new(&mut *writer, lba as u64, options.sector_size as u64);
      let root_record = root.root_directory_record(context);
      let mut buf = vec![];

      log::debug!("Writing root directory record: {:?}", root_record);

      {
        let dot_entry = spec::DirectoryRecord {
          extended_attribute_length: 0,
          extent_location: lba,
          data_length: root_record.data_length,
          recording_date: chrono::Utc::now().into(),
          file_flags: spec::FileFlags::DIRECTORY,
          file_unit_size: 0,
          interleave_gap_size: 0,
          volume_sequence_number: 1,
          file_identifier_length: 1,
          file_identifier: spec::Identifier::current_directory(),
        };

        buf.resize(dot_entry.extent(), 0);
        dot_entry.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      {
        let dotdot_entry = spec::DirectoryRecord {
          extended_attribute_length: 0,
          extent_location: root_record.extent_location,
          data_length: root_record.data_length,
          recording_date: chrono::Utc::now().into(),
          file_flags: spec::FileFlags::DIRECTORY,
          file_unit_size: 0,
          interleave_gap_size: 0,
          volume_sequence_number: 1,
          file_identifier_length: 1,
          file_identifier: spec::Identifier::parent_directory(),
        };

        buf.resize(dotdot_entry.extent(), 0);
        dotdot_entry.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      for subdir in root.dirs.values() {
        let subdir_record = subdir.directory_record(context);

        log::debug!("Writing root directory record: {:?}", subdir_record);

        buf.resize(subdir_record.extent(), 0);
        subdir_record.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      for file in root.files.values() {
        let file_record = file.directory_record(context);

        log::debug!("Writing root file record: {:?}", file_record);

        buf.resize(file_record.extent(), 0);
        file_record.serialize(&mut (), &mut buf)?;
        sector_writer.write_aligned(&buf)?;
        buf.clear();
      }

      for dir in root.dirs.values() {
        write_directory_entry(
          &mut *writer,
          root_record.extent_location,
          root_record.data_length,
          dir,
          options,
          context,
        )?;
      }

      for file in root.files.values() {
        write_file_entry(writer, file, options)?;
      }

      Ok(())
    }

    write_root_directory(
      &mut writer,
      &self.filesystem.root,
      &self.options,
      &context,
    )?;

    // 6. Done!

    Ok(())
  }
}
