use crate::spec::*;

type Result<T> = std::result::Result<T, IsoSerializeError>;

#[derive(Debug, thiserror::Error)]
pub enum IsoSerializeError {
  #[error("Output buffer too small")]
  OutputBufferTooSmall { expected: usize, actual: usize },
}

pub trait IsoSerialize {
  fn extent(&self) -> usize;

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()>;

  fn serialize(&self, out: &mut [u8]) -> Result<usize> {
    let extent = self.extent();

    if out.len() < extent {
      return Err(IsoSerializeError::OutputBufferTooSmall {
        expected: extent,
        actual: out.len(),
      });
    }

    unsafe {
      self.serialize_unchecked(out)?;
    }

    Ok(extent)
  }
}

impl<const LENGTH: usize> IsoSerialize for ACharacters<LENGTH> {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl<const LENGTH: usize> IsoSerialize for DCharacters<LENGTH> {
  fn extent(&self) -> usize {
    LENGTH
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl<const LENGTH: usize> IsoSerialize for A1Characters<LENGTH> {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl<const LENGTH: usize> IsoSerialize for D1Characters<LENGTH> {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl<const LENGTH: usize> IsoSerialize for EscapeSequences<LENGTH> {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl IsoSerialize for VariadicEscapeSequences {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl IsoSerialize for JolietFileIdentifier {
  fn extent(&self) -> usize {
    self.0.len() * 2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    self.0.iter().enumerate().for_each(|(i, &c)| {
      out[i * 2..i * 2 + 2].copy_from_slice(&c.to_be_bytes());
    });

    Ok(())
  }
}

impl IsoSerialize for JolietDirectoryIdentifier {
  fn extent(&self) -> usize {
    self.0.len() * 2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    self.0.iter().enumerate().for_each(|(i, &c)| {
      out[i * 2..i * 2 + 2].copy_from_slice(&c.to_be_bytes());
    });

    Ok(())
  }
}

impl IsoSerialize for FileFlags {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.bits();
    Ok(())
  }
}

impl IsoSerialize for Permissions {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..2].copy_from_slice(&self.bits().to_le_bytes());
    Ok(())
  }
}

impl IsoSerialize for VolumeFlags {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.bits();
    Ok(())
  }
}

impl<const LENGTH: usize> IsoSerialize for FileIdentifier<LENGTH> {
  fn extent(&self) -> usize {
    self.0.iter().position(|&b| b == 0).unwrap_or(self.0.len())
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0[..self.extent()]);
    Ok(())
  }
}

impl<const LENGTH: usize> IsoSerialize for DirectoryIdentifier<LENGTH> {
  fn extent(&self) -> usize {
    self.0.iter().position(|&b| b == 0).unwrap_or(self.0.len())
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.extent()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl IsoSerialize for OwnerIdentification {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..2].copy_from_slice(&self.0.to_le_bytes());
    Ok(())
  }
}

impl IsoSerialize for GroupIdentification {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..2].copy_from_slice(&self.0.to_le_bytes());
    Ok(())
  }
}

impl IsoSerialize for RecordAttributes {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = (*self).into();
    Ok(())
  }
}

impl IsoSerialize for ExtendedAttributeRecordVersion {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = (*self).into();
    Ok(())
  }
}

impl IsoSerialize for StandardIdentifier {
  fn extent(&self) -> usize {
    5
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..5].copy_from_slice(self.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for VolumeDescriptorVersion {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = (*self).into();
    Ok(())
  }
}

impl IsoSerialize for FileStructureVersion {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = (*self).into();
    Ok(())
  }
}

impl IsoSerialize for DigitsYear {
  fn extent(&self) -> usize {
    4
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // TODO(meowesque): This is inefficient
    let s = format!("{:04}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for DigitsMonth {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // TODO(meowesque): This is inefficient
    let s = format!("{:02}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for DigitsDay {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // TODO(meowesque): This is inefficient
    let s = format!("{:02}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for DigitsHour {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // TODO(meowesque): This is inefficient
    let s = format!("{:02}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for DigitsMinute {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // TODO(meowesque): This is inefficient
    let s = format!("{:02}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for DigitsSecond {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // TODO(meowesque): This is inefficient
    let s = format!("{:02}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for DigitsHundreths {
  fn extent(&self) -> usize {
    2
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    let s = format!("{:02}", self.0);
    out[..s.len()].copy_from_slice(s.as_bytes());
    Ok(())
  }
}

impl IsoSerialize for NumericalYear {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0;
    Ok(())
  }
}

impl IsoSerialize for NumericalMonth {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0;
    Ok(())
  }
}

impl IsoSerialize for NumericalDay {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0;
    Ok(())
  }
}

impl IsoSerialize for NumericalHour {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0;
    Ok(())
  }
}

impl IsoSerialize for NumericalMinute {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0;
    Ok(())
  }
}

impl IsoSerialize for NumericalSecond {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0;
    Ok(())
  }
}

impl IsoSerialize for NumericalGmtOffset {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.0 as u8;
    Ok(())
  }
}

impl IsoSerialize for DigitsDate {
  fn extent(&self) -> usize {
    self.year.extent()
      + self.month.extent()
      + self.day.extent()
      + self.hour.extent()
      + self.minute.extent()
      + self.second.extent()
      + self.hundreths.extent()
      + self.gmt_offset.extent()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    let mut offset = 0;

    self
      .year
      .serialize_unchecked(&mut out[offset..offset + self.year.extent()])?;
    offset += self.year.extent();

    self
      .month
      .serialize_unchecked(&mut out[offset..offset + self.month.extent()])?;
    offset += self.month.extent();

    self
      .day
      .serialize_unchecked(&mut out[offset..offset + self.day.extent()])?;
    offset += self.day.extent();

    self
      .hour
      .serialize_unchecked(&mut out[offset..offset + self.hour.extent()])?;
    offset += self.hour.extent();

    self
      .minute
      .serialize_unchecked(&mut out[offset..offset + self.minute.extent()])?;
    offset += self.minute.extent();

    self
      .second
      .serialize_unchecked(&mut out[offset..offset + self.second.extent()])?;
    offset += self.second.extent();

    self
      .hundreths
      .serialize_unchecked(&mut out[offset..offset + self.hundreths.extent()])?;
    offset += self.hundreths.extent();

    self
      .gmt_offset
      .serialize_unchecked(&mut out[offset..offset + self.gmt_offset.extent()])?;

    Ok(())
  }
}

impl IsoSerialize for NumericalDate {
  fn extent(&self) -> usize {
    self.years_since_1900.extent()
      + self.month.extent()
      + self.day.extent()
      + self.hour.extent()
      + self.minute.extent()
      + self.second.extent()
      + self.gmt_offset.extent()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    let mut offset = 0;

    self
      .years_since_1900
      .serialize_unchecked(&mut out[offset..offset + self.years_since_1900.extent()])?;
    offset += self.years_since_1900.extent();

    self
      .month
      .serialize_unchecked(&mut out[offset..offset + self.month.extent()])?;
    offset += self.month.extent();

    self
      .day
      .serialize_unchecked(&mut out[offset..offset + self.day.extent()])?;
    offset += self.day.extent();

    self
      .hour
      .serialize_unchecked(&mut out[offset..offset + self.hour.extent()])?;
    offset += self.hour.extent();

    self
      .minute
      .serialize_unchecked(&mut out[offset..offset + self.minute.extent()])?;
    offset += self.minute.extent();

    self
      .second
      .serialize_unchecked(&mut out[offset..offset + self.second.extent()])?;

    Ok(())
  }
}

impl IsoSerialize for PrimaryVolumeDescriptor {
  fn extent(&self) -> usize {
    2048
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = VolumeDescriptorType::Primary.into();
    out[1..6].copy_from_slice(self.standard_identifier.as_bytes());
    out[6] = self.version.into();
    out[7] = 0;
    self
      .system_identifier
      .serialize_unchecked(&mut out[8..40])?;
    self
      .volume_identifier
      .serialize_unchecked(&mut out[40..72])?;
    out[72..80].fill(0);

    out[80..84].copy_from_slice(&self.volume_space_size.to_le_bytes());
    out[84..88].copy_from_slice(&self.volume_space_size.to_be_bytes());

    out[88..120].fill(0);

    out[120..122].copy_from_slice(&self.volume_set_size.to_le_bytes());
    out[122..124].copy_from_slice(&self.volume_set_size.to_be_bytes());

    out[124..126].copy_from_slice(&self.volume_sequence_number.to_le_bytes());
    out[126..128].copy_from_slice(&self.volume_sequence_number.to_be_bytes());

    out[128..130].copy_from_slice(&self.logical_block_size.to_le_bytes());
    out[130..132].copy_from_slice(&self.logical_block_size.to_be_bytes());

    out[132..136].copy_from_slice(&self.path_table_size.to_le_bytes());
    out[136..140].copy_from_slice(&self.path_table_size.to_be_bytes());

    out[140..144].copy_from_slice(&self.type_l_path_table_location.to_le_bytes());

    out[144..148].copy_from_slice(&self.optional_type_l_path_table_location.to_le_bytes());

    out[148..152].copy_from_slice(&self.type_m_path_table_location.to_be_bytes());

    out[152..156].copy_from_slice(&self.optional_type_m_path_table_location.to_be_bytes());

    self
      .root_directory_record
      .serialize_unchecked(&mut out[156..190])?;

    self
      .volume_set_identifier
      .serialize_unchecked(&mut out[190..318])?;
    self
      .publisher_identifier
      .serialize_unchecked(&mut out[318..446])?;

    // TODO(meowesque): If the first btye is set to 5f, the remaining bytes of
    // TODO(meowesque): this field shall specify an identifier for a file containing
    // TODO(meowesque): the identification of the data preparer. This file shall be
    // TODO(meowesque): described in the root directory. The file name shall not contain
    // TODO(meowesque): contain more than 8 d-characters and the file name extension shall
    // TODO(meowesque): not contain more than 3 d-characters.
    self
      .data_preparer_identifier
      .serialize_unchecked(&mut out[446..574])?;

    // TODO(meowesque): If the first btye is set to 5f, the remaining bytes of
    // TODO(meowesque): this field shall specify an identifier for a file containing
    // TODO(meowesque): the identification of the data preparer. This file shall be
    // TODO(meowesque): described in the root directory. The file name shall not contain
    // TODO(meowesque): contain more than 8 d-characters and the file name extension shall
    // TODO(meowesque): not contain more than 3 d-characters.
    self
      .application_identifier
      .serialize_unchecked(&mut out[574..702])?;

    // TODO(meowesque): This field shall specify an identification for
    // TODO(meowesque): a file described by the root directory and
    // TODO(meowesque): containing the copyright statement for those volumes
    // TODO(meowesque): of the volume set the sequence numbers of which are
    // TODO(meowesque): less than, or equal to, the assigned volume set size
    // TODO(meowesque): of the volume. IF all bytes of this field are set
    // TODO(meowesque): to filler, it shall mean that no such file is identified.
    // TODO(meowesque): The file name shall not contain contain more than 8
    // TODO(meowesque): d-characters and the file name extension shall not contain
    // TODO(meowesque): more than 3 d-characters.
    self
      .copyright_file_identifier
      .serialize_unchecked(&mut out[702..739])?;
    self
      .abstract_file_identifier
      .serialize_unchecked(&mut out[739..776])?;
    self
      .bibliographic_file_identifier
      .serialize_unchecked(&mut out[776..813])?;

    self.creation_date.serialize_unchecked(&mut out[813..830])?;
    self
      .modification_date
      .serialize_unchecked(&mut out[830..847])?;
    self
      .expiration_date
      .serialize_unchecked(&mut out[847..864])?;
    self
      .effective_date
      .serialize_unchecked(&mut out[864..881])?;

    out[881] = self.file_structure_version.into();
    out[882] = 0;
    out[883..1395].copy_from_slice(&self.application_use);
    out[1395..2048].fill(0);

    Ok(())
  }
}

impl IsoSerialize for SupplementaryVolumeDescriptor {
  fn extent(&self) -> usize {
    2048
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = VolumeDescriptorType::Supplementary.into();
    out[1..6].copy_from_slice(self.standard_identifier.as_bytes());
    out[6] = self.version.into();
    out[7] = self.volume_flags.bits();
    self
      .system_identifier
      .serialize_unchecked(&mut out[8..40])?;
    self
      .volume_identifier
      .serialize_unchecked(&mut out[40..72])?;
    out[72..80].fill(0);
    out[80..84].copy_from_slice(&self.volume_space_size.to_le_bytes());
    out[84..88].copy_from_slice(&self.volume_space_size.to_be_bytes());
    out[88..120].fill(0); // TODO(meowesque): Add escape sequences
    out[120..122].copy_from_slice(&self.volume_set_size.to_le_bytes());
    out[122..124].copy_from_slice(&self.volume_set_size.to_be_bytes());
    out[124..126].copy_from_slice(&self.volume_sequence_number.to_le_bytes());
    out[126..128].copy_from_slice(&self.volume_sequence_number.to_be_bytes());
    out[128..130].copy_from_slice(&self.logical_block_size.to_le_bytes());
    out[130..132].copy_from_slice(&self.logical_block_size.to_be_bytes());
    out[132..136].copy_from_slice(&self.path_table_size.to_le_bytes());
    out[136..140].copy_from_slice(&self.path_table_size.to_be_bytes());
    out[140..144].copy_from_slice(&self.type_l_path_table_location.to_le_bytes());
    out[144..148].copy_from_slice(&self.optional_type_l_path_table_location.to_le_bytes());
    out[148..152].copy_from_slice(&self.type_m_path_table_location.to_le_bytes()); // TODO(meowesque): Check if the endianness is correct
    out[152..156].copy_from_slice(&self.optional_type_m_path_table_location.to_le_bytes());
    self
      .root_directory_record
      .serialize_unchecked(&mut out[156..190])?;
    self
      .volume_set_identifier
      .serialize_unchecked(&mut out[190..318])?;
    self
      .publisher_identifier
      .serialize_unchecked(&mut out[318..446])?;
    self
      .data_preparer_identifier
      .serialize_unchecked(&mut out[446..574])?;
    self
      .application_identifier
      .serialize_unchecked(&mut out[574..702])?;
    self
      .copyright_file_identifier
      .serialize_unchecked(&mut out[702..739])?;
    self
      .abstract_file_identifier
      .serialize_unchecked(&mut out[739..776])?;
    self
      .bibliographic_file_identifier
      .serialize_unchecked(&mut out[776..813])?;
    self.creation_date.serialize_unchecked(&mut out[813..830])?;
    self
      .modification_date
      .serialize_unchecked(&mut out[830..847])?;
    self
      .expiration_date
      .serialize_unchecked(&mut out[847..864])?;
    self
      .effective_date
      .serialize_unchecked(&mut out[864..881])?;
    out[881] = self.file_structure_version.into();
    out[882] = 0;
    out[883..1395].copy_from_slice(&self.application_use);
    out[1395..2048].fill(0);

    Ok(())
  }
}

impl IsoSerialize for VolumePartitionDescriptor {
  fn extent(&self) -> usize {
    2048
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = VolumeDescriptorType::Partition.into();
    out[1..6].copy_from_slice(self.standard_identifier.as_bytes());
    out[6] = self.version.into();
    out[7] = 0;
    self
      .system_identifier
      .serialize_unchecked(&mut out[8..40])?;
    self
      .volume_partition_identifier
      .serialize_unchecked(&mut out[40..72])?;
    out[72..76].copy_from_slice(&self.volume_partition_location.to_le_bytes());
    out[76..80].copy_from_slice(&self.volume_partition_location.to_be_bytes());
    out[80..84].copy_from_slice(&self.volume_partition_size.to_le_bytes());
    out[84..88].copy_from_slice(&self.volume_partition_size.to_be_bytes());
    out[88..2048].fill(0);

    Ok(())
  }
}

impl IsoSerialize for VolumeDescriptorSetTerminator {
  fn extent(&self) -> usize {
    2048
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = VolumeDescriptorType::Terminator.into();
    out[1..6].copy_from_slice(b"CD001"); // TODO(meowesque): This might be different depending on the serialization context
    out[6] = VolumeDescriptorVersion::Standard.into();
    out[7..2048].fill(0);

    Ok(())
  }
}

impl<Ext: Extension> IsoSerialize for DirectoryRecord<Ext>
where
  Ext::FileIdentifier: IsoSerialize,
{
  fn extent(&self) -> usize {
    33 + self.file_identifier_length as usize + (self.file_identifier_length % 2 == 0) as usize
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.extent() as u8;
    out[1] = self.extended_attribute_length;
    out[2..6].copy_from_slice(&self.extent_location.to_le_bytes());
    out[6..10].copy_from_slice(&self.extent_location.to_be_bytes());
    out[10..14].copy_from_slice(&self.data_length.to_le_bytes());
    out[14..18].copy_from_slice(&self.data_length.to_be_bytes());
    self.recording_date.serialize_unchecked(&mut out[18..25])?;
    self.file_flags.serialize_unchecked(&mut out[25..26])?;
    out[26] = self.file_unit_size;
    out[27] = self.interleave_gap_size;
    out[28..30].copy_from_slice(&self.volume_sequence_number.to_le_bytes());
    out[30..32].copy_from_slice(&self.volume_sequence_number.to_be_bytes());
    // TODO(meowesque): Why does this fix identifiers being misread ?
    out[32] = self.file_identifier_length;

    // TODO(meowesque): Check if this is right ?
    self
      .file_identifier
      .serialize_unchecked(&mut out[33..33 + self.file_identifier_length as usize])?;

    // TODO(meowesque): Check if this is right ?
    if self.file_identifier_length % 2 == 0 {
      out[33 + self.file_identifier_length as usize] = 0;
    }

    Ok(())
  }
}

impl IsoSerialize for RootDirectoryRecord {
  fn extent(&self) -> usize {
    34
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.extent() as u8;
    out[1] = 0;

    out[2..6].copy_from_slice(&self.extent_location.to_le_bytes());
    out[6..10].copy_from_slice(&self.extent_location.to_be_bytes());

    out[10..14].copy_from_slice(&self.data_length.to_le_bytes());
    out[14..18].copy_from_slice(&self.data_length.to_be_bytes());

    self.recording_date.serialize_unchecked(&mut out[18..25])?;
    self.file_flags.serialize_unchecked(&mut out[25..26])?;
    out[26] = self.file_unit_size;
    out[27] = self.interleave_gap_size;
    out[28..30].copy_from_slice(&self.volume_sequence_number.to_le_bytes());
    out[30..32].copy_from_slice(&self.volume_sequence_number.to_be_bytes());
    out[32] = 0; // NOTE(meowesque): This might be one
    out[33] = 0;

    Ok(())
  }
}

impl IsoSerialize for ElToritoManufacturerId {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.0.len()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl IsoSerialize for ElToritoBootMediaTypeExt {
  fn extent(&self) -> usize {
    1
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    // NOTE(meowesque): Thank you Rémy (:

    out[0] = (self.emulation_type as u8)
      | (self.continuation_entry_follows as u8) << 5
      | (self.contains_atapi_driver as u8) << 6
      | (self.contains_scsi_drivers as u8) << 7;

    Ok(())
  }
}

impl IsoSerialize for ElToritoSectionId {
  fn extent(&self) -> usize {
    self.0.len()
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[..self.0.len()].copy_from_slice(&self.0);
    Ok(())
  }
}

impl IsoSerialize for ElToritoInitialSectionEntry {
  fn extent(&self) -> usize {
    32
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.boot_indicator.into();
    out[1] = self.boot_media_type.into();
    out[2..=3].copy_from_slice(&self.load_segment.to_le_bytes());
    out[4] = self.system_type;
    out[5] = 0;
    out[6..=7].copy_from_slice(&self.sector_count.to_le_bytes());
    out[8..=0x0b].copy_from_slice(&self.virtual_disk_location.to_le_bytes());
    out[0x0c..=0x1f].fill(0);

    Ok(())
  }
}

impl IsoSerialize for ElToritoSectionHeaderEntry {
  fn extent(&self) -> usize {
    32
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.header_indicator as u8;
    out[1] = self.platform_id.into();
    out[2..=3].copy_from_slice(&self.succeeding_section_entries.to_le_bytes());
    self.section_id.serialize_unchecked(&mut out[4..=0x1f])?;

    Ok(())
  }
}

impl IsoSerialize for ElToritoValidationEntry {
  fn extent(&self) -> usize {
    32
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.header_id.into();
    out[1] = self.platform_id.into();
    out[2..=3].fill(0);
    self
      .manufacturer_id
      .serialize_unchecked(&mut out[4..=0x1b])?;
    out[0x1c..=0x1d].copy_from_slice(&self.checksum.to_le_bytes());
    out[0x1e] = 0x55;
    out[0x1f] = 0xAA;

    Ok(())
  }
}

impl IsoSerialize for ElToritoSectionEntry {
  fn extent(&self) -> usize {
    32
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = self.boot_indicator.into();
    self.boot_media_type.serialize_unchecked(&mut out[1..2])?;
    out[2..=3].copy_from_slice(&self.load_segment.to_le_bytes());
    out[4] = self.system_type;
    out[5] = 0;
    out[6..=7].copy_from_slice(&self.sector_count.to_le_bytes());
    out[8..=0x0b].copy_from_slice(&self.virtual_disk_location.to_le_bytes());
    out[0x0c] = self.selection_criteria_type.into();
    out[0x0d..=0x1f].copy_from_slice(&self.vendor_selection_criteria);

    Ok(())
  }
}

impl IsoSerialize for ElToritoSectionEntryExtension {
  fn extent(&self) -> usize {
    32
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = 44;
    out[1] = self.extension_record_follows_indicator.bits();
    out[2..=0x1F].copy_from_slice(&self.vendor_unique_selection_criteria);

    Ok(())
  }
}

impl IsoSerialize for ElToritoBootRecordVolumeDescriptor {
  fn extent(&self) -> usize {
    2048
  }

  unsafe fn serialize_unchecked(&self, out: &mut [u8]) -> Result<()> {
    out[0] = 0;
    out[1..=5].copy_from_slice(self.standard_identifier.as_bytes());
    out[6] = self.version.into();
    out[7..=26].copy_from_slice(b"EL TORITO SPECIFICATION");
    out[27..=46].fill(0);
    out[47..=0x4a].copy_from_slice(&self.boot_catalog_pointer.to_le_bytes());
    out[0x4a..=0x7ff].fill(0);

    Ok(())
  }
}
