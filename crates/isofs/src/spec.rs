//! UDF and ISO 9660 specification types including extensions such as Joliet and Rock Ridge.

pub trait Extension {
  type FileIdentifier: std::fmt::Debug;
  type DirectoryIdentifier: std::fmt::Debug;
}

/// No extensions; Standard ISO 9660 only.
#[derive(Debug)]
pub struct NoExtension;

impl Extension for NoExtension {
  type FileIdentifier = FileIdentifier<32>;
  type DirectoryIdentifier = DirectoryIdentifier<31>;
}

#[derive(Debug)]
pub enum JolietLevel {
  /// UCS-2 Level 1
  Level1,
  /// UCS-2 Level 2
  Level2,
  /// UCS-2 Level 3
  Level3,
}

/// Microsoft Joliet extension.
#[derive(Debug)]
pub struct JolietExtension {
  pub level: JolietLevel,
}

impl Extension for JolietExtension {
  type FileIdentifier = JolietFileIdentifier;
  type DirectoryIdentifier = JolietDirectoryIdentifier;
}

/// `[\s\!\"\%\&\'\(\)\*\+\,\-\.\/0-9A-Z\:\;\<\=\>\?\_A-Z0-9]`
#[derive(Debug)]
pub struct ACharacters<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

impl<const LENGTH: usize> ACharacters<LENGTH> {
  /// Convert from a byte slice, truncating or zero-padding as necessary.
  pub fn from_bytes_truncated(bytes: &[u8]) -> Self {
    // TODO(meowesque): Validate characters?
    let mut cs = [0u8; LENGTH];
    cs[..LENGTH.min(bytes.len())].copy_from_slice(&bytes[..LENGTH.min(bytes.len())]);
    Self(cs)
  }
}

/// `[0-9A-Z_]``
#[derive(Debug)]
pub struct DCharacters<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

impl<const LENGTH: usize> DCharacters<LENGTH> {
  const CHARACTER_SET: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_";

  /// Convert from a byte slice, truncating or zero-padding as necessary.
  pub fn from_bytes_truncated(bytes: &[u8]) -> Option<Self> {
    if !bytes.iter().all(|b| Self::CHARACTER_SET.contains(b)) {
      return None;
    }

    let mut cs = [0u8; LENGTH];

    cs[..LENGTH.min(bytes.len())].copy_from_slice(&bytes[..LENGTH.min(bytes.len())]);

    Some(Self(cs))
  }
}

#[derive(Debug)]
pub struct A1Characters<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

#[derive(Debug)]
pub struct D1Characters<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

/// Escape sequences conforming to ISO/IEC 2022, including the escape characters.
///
/// If all the bytes of the escape sequences are zero, it shall mean that the set
/// of a1-characters is identical to the set of a-characters.
#[derive(Debug)]
pub struct EscapeSequences<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

/// Escape sequences conforming to ISO/IEC 2022, excluding the escape characters.
#[derive(Debug)]
pub struct VariadicEscapeSequences(pub(crate) Vec<u8>);

#[derive(Debug)]
pub struct JolietFileIdentifier(pub(crate) [u16; 64]);

#[derive(Debug)]
pub struct JolietDirectoryIdentifier(pub(crate) [u16; 64]);

bitflags::bitflags! {
  #[derive(Debug)]
  pub struct FileFlags: u8 {
    const EXISTENCE = 1 << 0;
    const DIRECTORY = 1 << 1;
    const ASSOCIATED_FILE = 1 << 2;
    const RECORD = 1 << 3;
    const PROTECTION = 1 << 4;
    const RESERVED_5 = 1 << 5;
    const RESERVED_6 = 1 << 6;
    const MULTI_EXTENT = 1 << 7;
  }

  #[derive(Debug)]
  pub struct Permissions: u16 {
    const SYSTEM_READ = 1 << 0;
    /// "Shall be set to 1."
    const PERMISSION_1 = 1 << 1;
    const SYSTEM_EXECUTE = 1 << 2;
    /// "Shall be set to 1."
    const PERMISSION_3 = 1 << 3;
    const USER_READ = 1 << 4;
    /// "Shall be set to 1."
    const PERMISSION_5 = 1 << 5;
    const USER_EXECUTE = 1 << 6;
    /// "Shall be set to 1."
    const PERMISSION_7 = 1 << 7;
    const OTHER_READ = 1 << 8;
    /// "Shall be set to 1."
    const PERMISSION_9 = 1 << 9;
    const OTHER_EXECUTE = 1 << 10;
    /// "Shall be set to 1."
    const PERMISSION_11 = 1 << 11;
    const ALL_READ = 1 << 12;
    /// "Shall be set to 1."
    const PERMISSION_13 = 1 << 13;
    const ALL_EXECUTE = 1 << 14;
    /// "Shall be set to 1."
    const PERMISSION_15 = 1 << 15;
  }

  #[derive(Debug)]
  pub struct VolumeFlags: u8 {
    /// If zero, shall mean that the escape sequences field specifies only
    /// escape sequences registered by ISO/IEC 2375.
    ///
    /// If one, shall mean that the escape sequences field includes
    /// atleast one escape sequence not registered according to ISO/IEC 2375.
    const UNREGISTERED_ESCAPE_SEQUENCES = 1 << 0;
    const RESERVED_1 = 1 << 1;
    const RESERVED_2 = 1 << 2;
    const RESERVED_3 = 1 << 3;
    const RESERVED_4 = 1 << 4;
    const RESERVED_5 = 1 << 5;
    const RESERVED_6 = 1 << 6;
    const RESERVED_7 = 1 << 7;
  }
}

#[derive(Debug)]
pub struct FileIdentifier<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

impl<const LENGTH: usize> FileIdentifier<LENGTH> {
  /// Convert from a byte slice, truncating or zero-padding as necessary.
  pub fn from_bytes_truncated(bytes: &[u8]) -> Self {
    // TODO(meowesque): Validate characters?
    let mut cs = [0u8; LENGTH];
    cs[..LENGTH.min(bytes.len())].copy_from_slice(&bytes[..LENGTH.min(bytes.len())]);
    Self(cs)
  }
}

/// `DCharacters`/`D1Characters`.
#[derive(Debug)]
pub struct DirectoryIdentifier<const LENGTH: usize>(pub(crate) [u8; LENGTH]);

/// TODO(meowesque): Define this better?
#[derive(Debug)]
pub struct OwnerIdentification(pub(crate) u16);

/// TODO(meowesque): Define this better?
#[derive(Debug)]
pub struct GroupIdentification(pub(crate) u16);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum RecordFormat {
  StructureNotSpecified = 0,
  FixedLengthRecords = 1,
  VariableLengthRecordsMsb = 2,
  VariableLengthRecordsLsb = 3,
  Other(u8),
}

impl Into<u8> for RecordFormat {
  fn into(self) -> u8 {
    match self {
      RecordFormat::StructureNotSpecified => 0,
      RecordFormat::FixedLengthRecords => 1,
      RecordFormat::VariableLengthRecordsMsb => 2,
      RecordFormat::VariableLengthRecordsLsb => 3,
      RecordFormat::Other(v) => v,
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum RecordAttributes {
  PreceededByLfcFollowedByCrc = 0,
  /// First byte of the record shall be interpreted as specified in ISO/IEC 1539-1 for vertical spacing.
  FirstByteInterpretedByIso15391 = 1,
  ContainsNecessaryControlInformation = 2,
  Other(u8),
}

impl Into<u8> for RecordAttributes {
  fn into(self) -> u8 {
    match self {
      RecordAttributes::PreceededByLfcFollowedByCrc => 0,
      RecordAttributes::FirstByteInterpretedByIso15391 => 1,
      RecordAttributes::ContainsNecessaryControlInformation => 2,
      RecordAttributes::Other(v) => v,
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExtendedAttributeRecordVersion {
  Standard = 1,
  Other(u8),
}

impl Into<u8> for ExtendedAttributeRecordVersion {
  fn into(self) -> u8 {
    match self {
      ExtendedAttributeRecordVersion::Standard => 1,
      ExtendedAttributeRecordVersion::Other(v) => v,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum StandardIdentifier {
  /// Standard ISO 9660 identifier; "CD001"
  Cd001,
  /// Denotes the beginning of the extended descriptor section; "BEA01"
  Bea01,
  /// Indicates that this volume contains a UDF filesystem; "NSR02"
  Nsr02,
  /// Indicates that this volume contains a UDF filesystem; "NSR03"
  Nsr03,
  /// Includes information concerning boot loader location and entry point address; "BOOT2"
  Boot2,
  /// Indicates the end of the extended descriptor section; "TEA01"
  Tea01,
  /// Any other identifier not covered by the above variants.
  Other([u8; 5]),
}

impl StandardIdentifier {
  pub fn as_bytes(&self) -> &[u8; 5] {
    match self {
      StandardIdentifier::Cd001 => b"CD001",
      StandardIdentifier::Bea01 => b"BEA01",
      StandardIdentifier::Nsr02 => b"NSR02",
      StandardIdentifier::Nsr03 => b"NSR03",
      StandardIdentifier::Boot2 => b"BOOT2",
      StandardIdentifier::Tea01 => b"TEA01",
      StandardIdentifier::Other(v) => v,
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VolumeDescriptorType {
  BootRecord = 0,
  Primary = 1,
  Supplementary = 2,
  Partition = 3,
  Other(u8),
  Terminator = 255,
}

impl Into<u8> for VolumeDescriptorType {
  fn into(self) -> u8 {
    match self {
      VolumeDescriptorType::BootRecord => 0,
      VolumeDescriptorType::Primary => 1,
      VolumeDescriptorType::Supplementary => 2,
      VolumeDescriptorType::Partition => 3,
      VolumeDescriptorType::Other(v) => v,
      VolumeDescriptorType::Terminator => 255,
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum VolumeDescriptorVersion {
  Standard = 1,
  Other(u8),
}

impl Into<u8> for VolumeDescriptorVersion {
  fn into(self) -> u8 {
    match self {
      VolumeDescriptorVersion::Standard => 1,
      VolumeDescriptorVersion::Other(v) => v,
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FileStructureVersion {
  Standard = 1,
  Other(u8),
}

impl Into<u8> for FileStructureVersion {
  fn into(self) -> u8 {
    match self {
      FileStructureVersion::Standard => 1,
      FileStructureVersion::Other(v) => v,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DigitsYear(pub(crate) u16);

#[derive(Debug, Clone)]
pub struct DigitsMonth(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct DigitsDay(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct DigitsHour(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct DigitsMinute(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct DigitsHundreths(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct DigitsSecond(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalYear(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalMonth(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalDay(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalHour(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalMinute(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalSecond(pub(crate) u8);

#[derive(Debug, Clone)]
pub struct NumericalGmtOffset(pub(crate) i8);

#[derive(Debug, Clone)]
pub struct DigitsDate {
  pub year: DigitsYear,
  pub month: DigitsMonth,
  pub day: DigitsDay,
  pub hour: DigitsHour,
  pub minute: DigitsMinute,
  pub second: DigitsSecond,
  pub hundreths: DigitsHundreths,
  pub gmt_offset: NumericalGmtOffset,
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> From<chrono::DateTime<Tz>> for DigitsDate {
  fn from(dt: chrono::DateTime<Tz>) -> Self {
    use chrono::{Datelike, Timelike};

    Self {
      year: DigitsYear(dt.year() as u16),
      month: DigitsMonth(dt.month() as u8),
      day: DigitsDay(dt.day() as u8),
      hour: DigitsHour(dt.hour() as u8),
      minute: DigitsMinute(dt.minute() as u8),
      second: DigitsSecond(dt.second() as u8),
      hundreths: DigitsHundreths((dt.timestamp_subsec_millis() / 10) as u8),
      // TODO(meowesque): Calculate this.
      gmt_offset: NumericalGmtOffset(0),
    }
  }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> Into<chrono::DateTime<Tz>> for DigitsDate {
  fn into(self) -> chrono::DateTime<Tz> {
    todo!()
  }
}

#[derive(Debug, Clone)]
pub struct NumericalDate {
  pub years_since_1900: NumericalYear,
  pub month: NumericalMonth,
  pub day: NumericalDay,
  pub hour: NumericalHour,
  pub minute: NumericalMinute,
  pub second: NumericalSecond,
  pub gmt_offset: NumericalGmtOffset,
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> From<chrono::DateTime<Tz>> for NumericalDate {
  fn from(dt: chrono::DateTime<Tz>) -> Self {
    use chrono::{Datelike, Timelike};

    Self {
      years_since_1900: NumericalYear((dt.year().max(1900) - 1900) as u8),
      month: NumericalMonth(dt.month() as u8),
      day: NumericalDay(dt.day() as u8),
      hour: NumericalHour(dt.hour() as u8),
      minute: NumericalMinute(dt.minute() as u8),
      second: NumericalSecond(dt.second() as u8),
      // TODO(meowesque): Calculate this.
      gmt_offset: NumericalGmtOffset(0),
    }
  }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> Into<chrono::DateTime<Tz>> for NumericalDate {
  fn into(self) -> chrono::DateTime<Tz> {
    todo!()
  }
}

#[derive(Debug)]
pub struct PrimaryVolumeDescriptor {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub system_identifier: ACharacters<32>,
  pub volume_identifier: DCharacters<32>,
  pub volume_space_size: u32,
  pub volume_set_size: u16,
  pub volume_sequence_number: u16,
  pub logical_block_size: u16,
  pub path_table_size: u32,
  pub type_l_path_table_location: u32,
  pub optional_type_l_path_table_location: u32,
  pub type_m_path_table_location: u32,
  pub optional_type_m_path_table_location: u32,
  pub root_directory_record: RootDirectoryRecord,
  pub volume_set_identifier: DCharacters<128>,
  pub publisher_identifier: ACharacters<128>,
  pub data_preparer_identifier: ACharacters<128>,
  pub application_identifier: ACharacters<128>,
  pub copyright_file_identifier: DCharacters<37>, // Separator 1 and 2
  pub abstract_file_identifier: DCharacters<37>,  // Separator 1 and 2
  pub bibliographic_file_identifier: DCharacters<37>, // Separator 1 and 2
  pub creation_date: DigitsDate,
  pub modification_date: DigitsDate,
  pub expiration_date: DigitsDate,
  pub effective_date: DigitsDate,
  pub file_structure_version: FileStructureVersion,
  pub application_use: [u8; 512],
}

#[derive(Debug)]
pub struct SupplementaryVolumeDescriptor {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub volume_flags: VolumeFlags,
  pub system_identifier: A1Characters<32>,
  pub volume_identifier: D1Characters<32>,
  pub volume_space_size: u32,
  pub escape_sequences: EscapeSequences<32>,
  pub volume_set_size: u16,
  pub volume_sequence_number: u16,
  pub logical_block_size: u16,
  pub path_table_size: u32,
  pub type_l_path_table_location: u32,
  pub optional_type_l_path_table_location: u32,
  pub type_m_path_table_location: u32,
  pub optional_type_m_path_table_location: u32,
  pub root_directory_record: RootDirectoryRecord,
  pub volume_set_identifier: D1Characters<128>,
  pub publisher_identifier: A1Characters<128>,
  pub data_preparer_identifier: A1Characters<128>,
  pub application_identifier: A1Characters<128>,
  pub copyright_file_identifier: D1Characters<37>, // Separator 1 and 2
  pub abstract_file_identifier: D1Characters<37>,  // Separator 1 and 2
  pub bibliographic_file_identifier: D1Characters<37>, // Separator 1 and 2
  pub creation_date: DigitsDate,
  pub modification_date: DigitsDate,
  pub expiration_date: DigitsDate,
  pub effective_date: DigitsDate,
  pub file_structure_version: FileStructureVersion,
  pub application_use: [u8; 512],
}

#[derive(Debug)]
pub struct VolumePartitionDescriptor {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub system_identifier: ACharacters<32>,
  pub volume_partition_identifier: DCharacters<32>,
  pub volume_partition_location: u32,
  pub volume_partition_size: u32,
}

#[derive(Debug)]
pub struct VolumeDescriptorSetTerminator;

#[derive(Debug)]
pub struct DirectoryRecord<Ext: Extension> {
  pub extended_attribute_length: u8,
  pub extent_location: u32,
  pub data_length: u32,
  pub recording_date: NumericalDate,
  pub file_flags: FileFlags,
  pub file_unit_size: u8,
  pub interleave_gap_size: u8,
  pub volume_sequence_number: u16,
  pub file_identifier_length: u8,
  pub file_identifier: Ext::FileIdentifier,
}

/// Root directory record as found in `SupplementaryVolumeDescriptor` and
/// `PrimaryVolumeDescriptor`. Like `DirectoryRecord` but without the `length`
/// and `extended_attribute_length` fields.
#[derive(Debug)]
pub struct RootDirectoryRecord {
  pub extent_location: u32,
  pub data_length: u32,
  pub recording_date: NumericalDate,
  pub file_flags: FileFlags,
  pub file_unit_size: u8,
  pub interleave_gap_size: u8,
  pub volume_sequence_number: u16,
}

#[derive(Debug)]
pub struct PathTableRecord<Ext: Extension> {
  pub directory_identifier_length: u8,
  pub extent_location: u32,
  pub parent_directory_number: u16,
  pub directory_identifier: Ext::DirectoryIdentifier,
}

#[derive(Debug)]
pub struct ExtendedAttributeRecord {
  pub owner_identification: OwnerIdentification,
  pub group_identification: GroupIdentification,
  pub permissions: Permissions,
  pub file_creation_date: DigitsDate,
  pub file_modification_date: DigitsDate,
  pub file_expiration_date: DigitsDate,
  pub file_effective_date: DigitsDate,
  pub record_format: RecordFormat,
  pub record_attributes: RecordAttributes,
  pub extended_attribute_record_version: ExtendedAttributeRecordVersion,
  pub application_use: Vec<u8>,
  pub escape_sequences: VariadicEscapeSequences,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElToritoHeaderId {
  Standard = 1,
  Other(u8),
}

impl Into<u8> for ElToritoHeaderId {
  fn into(self) -> u8 {
    match self {
      ElToritoHeaderId::Standard => 1,
      ElToritoHeaderId::Other(v) => v,
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElToritoPlatformId {
  X86 = 0,
  PowerPc = 1,
  Mac = 2,
  Other(u8),
}

impl Into<u8> for ElToritoPlatformId {
  fn into(self) -> u8 {
    match self {
      ElToritoPlatformId::X86 => 0,
      ElToritoPlatformId::PowerPc => 1,
      ElToritoPlatformId::Mac => 2,
      ElToritoPlatformId::Other(v) => v,
    }
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElToritoBootIndicator {
  Bootable = 0x88,
  NonBootable = 0x00,
  Other(u8),
}

impl Into<u8> for ElToritoBootIndicator {
  fn into(self) -> u8 {
    match self {
      ElToritoBootIndicator::Bootable => 0x88,
      ElToritoBootIndicator::NonBootable => 0x00,
      ElToritoBootIndicator::Other(v) => v,
    }
  }
}

#[derive(Debug)]
pub struct ElToritoManufacturerId(pub(crate) [u8; 16]);

bitflags::bitflags! {
  #[derive(Debug)]
  pub struct ElToritoExtensionRecordFollowsIndicator: u8 {
    const EXTENSION_RECORD_FOLLOWS = 1 << 5;
  }
}

// TODO(meowesque): Implement
#[derive(Debug, Clone, Copy)]
pub struct ElToritoBootMediaType(pub(crate) u8);

impl Into<u8> for ElToritoBootMediaType {
  fn into(self) -> u8 {
    self.0
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ElToritoEmulationType {
  NoEmulation = 0,
  Floppy12M = 1,
  Floppy144M = 2,
  Floppy288M = 3,
  HardDisk = 4,
}

impl Into<u8> for ElToritoEmulationType {
  fn into(self) -> u8 {
    match self {
      ElToritoEmulationType::NoEmulation => 0,
      ElToritoEmulationType::Floppy12M => 1,
      ElToritoEmulationType::Floppy144M => 2,
      ElToritoEmulationType::Floppy288M => 3,
      ElToritoEmulationType::HardDisk => 4,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ElToritoBootMediaTypeExt {
  pub emulation_type: ElToritoEmulationType,
  pub continuation_entry_follows: bool,
  pub contains_atapi_driver: bool,
  pub contains_scsi_drivers: bool,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ElToritoHeaderIndicator {
  MoreHeadersFollow = 90,
  FinalHeader = 91,
}

#[derive(Debug, Clone, Copy)]
pub struct ElToritoSectionId(pub(crate) [u8; 16]);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ElToritoSelectionCriteriaType {
  NoSelectionCriteria = 0,
  LanguageAndVersionInformation = 1,
  Other(u8),
}

impl Into<u8> for ElToritoSelectionCriteriaType {
  fn into(self) -> u8 {
    match self {
      ElToritoSelectionCriteriaType::NoSelectionCriteria => 0,
      ElToritoSelectionCriteriaType::LanguageAndVersionInformation => 1,
      ElToritoSelectionCriteriaType::Other(v) => v,
    }
  }
}

#[derive(Debug)]
pub struct ElToritoInitialSectionEntry {
  pub boot_indicator: ElToritoBootIndicator,
  pub boot_media_type: ElToritoBootMediaType,
  pub load_segment: u16,
  pub system_type: u8,
  pub sector_count: u16,
  pub virtual_disk_location: u32,
}

#[derive(Debug)]
pub struct ElToritoSectionHeaderEntry {
  pub header_indicator: ElToritoHeaderIndicator,
  pub platform_id: ElToritoPlatformId,
  pub succeeding_section_entries: u16,
  pub section_id: ElToritoSectionId,
}

#[derive(Debug)]
pub struct ElToritoValidationEntry {
  pub header_id: ElToritoHeaderId,
  pub platform_id: ElToritoPlatformId,
  pub manufacturer_id: ElToritoManufacturerId,
  pub checksum: u16,
}

#[derive(Debug)]
pub struct ElToritoSectionEntry {
  pub boot_indicator: ElToritoBootIndicator,
  pub boot_media_type: ElToritoBootMediaTypeExt,
  pub load_segment: u16,
  pub system_type: u8,
  pub sector_count: u16,
  pub virtual_disk_location: u32,
  pub selection_criteria_type: ElToritoSelectionCriteriaType,
  pub vendor_selection_criteria: [u8; 18],
}

#[derive(Debug)]
pub struct ElToritoSectionEntryExtension {
  pub extension_record_follows_indicator: ElToritoExtensionRecordFollowsIndicator,
  pub vendor_unique_selection_criteria: [u8; 29],
}

#[derive(Debug)]
pub struct ElToritoBootRecordVolumeDescriptor {
  pub standard_identifier: StandardIdentifier,
  pub version: VolumeDescriptorVersion,
  pub boot_catalog_pointer: u32,
}
