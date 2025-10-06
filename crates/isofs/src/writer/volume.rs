use crate::spec;

pub struct VolumeContext {
  pub sector_size: u32,
  pub standard_identifier: spec::StandardIdentifier,
}

pub trait VolumeLike {
  type Descriptor;

  fn volume_id(&self) -> &str;
  fn descriptor(&self, context: &VolumeContext) -> Self::Descriptor;
}

pub struct PrimaryVolume {
  pub volume_id: String,
  pub publisher: Option<String>,
  pub preparer: Option<String>,
  pub filesystem: super::fs::Filesystem,
}

impl VolumeLike for PrimaryVolume {
  type Descriptor = spec::PrimaryVolumeDescriptor;

  fn volume_id(&self) -> &str {
    &self.volume_id
  }

  fn descriptor(&self, context: &VolumeContext) -> Self::Descriptor {
    spec::PrimaryVolumeDescriptor {
      standard_identifier: context.standard_identifier,
      version: spec::VolumeDescriptorVersion::Standard,
      system_identifier: spec::ACharacters::from_bytes_truncated(b"LINUX"),
      volume_identifier: spec::DCharacters::from_bytes_truncated(self.volume_id().as_bytes())
        .unwrap(),
      volume_space_size: 0,
      volume_set_size: 0,
      volume_sequence_number: 0,
      logical_block_size: context.sector_size as u16,
      path_table_size: 0,
      type_l_path_table_location: self.filesystem.root.extent_lba.unwrap_or(0),
      optional_type_l_path_table_location: 0,
      type_m_path_table_location: 0,
      optional_type_m_path_table_location: 0,
      root_directory_record: self.filesystem.root.root_descriptor(),
      volume_set_identifier: spec::DCharacters::from_bytes_truncated(b"ABC").unwrap(),
      publisher_identifier: spec::ACharacters::from_bytes_truncated(b"xyz"),
      data_preparer_identifier: spec::ACharacters::from_bytes_truncated(b"def"),
      application_identifier: spec::ACharacters::from_bytes_truncated(b"ghi"),
      copyright_file_identifier: spec::DCharacters::from_bytes_truncated(b"JKL").unwrap(),
      abstract_file_identifier: spec::DCharacters::from_bytes_truncated(b"MNO").unwrap(),
      bibliographic_file_identifier: spec::DCharacters::from_bytes_truncated(b"PQR").unwrap(),
      creation_date: chrono::Utc::now().into(),
      modification_date: chrono::Utc::now().into(),
      expiration_date: chrono::Utc::now().into(),
      effective_date: chrono::Utc::now().into(),
      file_structure_version: spec::FileStructureVersion::Standard,
      application_use: [0; 512],
    }
  }
}

impl Into<Volume> for PrimaryVolume {
  fn into(self) -> Volume {
    Volume::Primary(self)
  }
}

pub enum Volume {
  Primary(PrimaryVolume),
}
