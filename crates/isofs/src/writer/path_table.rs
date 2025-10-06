use crate::{
  serialize::{Endianness, IsoSerialize},
  spec,
  writer::{fs, lba},
};

pub struct PathTable {
  l_size: u32,
  m_size: u32,
  records: Vec<spec::PathTableRecord<spec::NoExtension>>,
}

impl PathTable {
  pub fn from_filesystem(fs: &crate::writer::fs::Filesystem) -> Self {
    fn aggregate_records<D>(
      entry: &D,
      parent_index: u16,
      records: &mut Vec<spec::PathTableRecord<spec::NoExtension>>,
    ) where
      D: fs::EntryLike + fs::DirectoryLike,
    {
      let descriptor = entry.descriptor();

      let record_ix = records.len() as u16 + 1; // 1-based index

      let record = spec::PathTableRecord {
        directory_identifier_length: descriptor.file_identifier_length,
        extended_attribute_record_length: 0,
        extent_location: descriptor.extent_location,
        parent_directory_number: parent_index,
        // TODO(meowesque): 31 length? extensions? .unwrap()?
        directory_identifier: spec::DirectoryIdentifier::from_bytes_truncated(
          &descriptor.file_identifier.0,
        )
        .unwrap(),
      };

      records.push(record);

      for entry in entry.entries_iter() {
        if let fs::Entry::Directory(child) = entry {
          aggregate_records(child, record_ix, records);
        }
      }
    }

    let records = {
      let mut records = vec![];
      aggregate_records(&fs.root, 1, &mut records);
      records
    };

    let table_size = records.iter().map(|r| r.extent() as u32).sum::<u32>();

    Self {
      records,
      l_size: table_size,
      m_size: table_size,
    }
  }

  pub(crate) fn allocate_l_lba(&self, allocator: &mut lba::LbaAllocator) -> u32 {
    allocator.allocate(self.l_size)
  }

  pub(crate) fn allocate_m_lba(&self, allocator: &mut lba::LbaAllocator) -> u32 {
    allocator.allocate(self.m_size)
  }

  pub(crate) fn records_iter(
    &self,
  ) -> impl Iterator<Item = &spec::PathTableRecord<spec::NoExtension>> {
    self.records.iter()
  }

  pub(crate) fn size(&self) -> u32 {
    self.l_size
  }
}
