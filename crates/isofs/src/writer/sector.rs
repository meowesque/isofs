pub(crate) struct SectorWriter<Storage> {
  storage: Storage,
  sector_ix: u64,
  sector_size: u64,
  bytes_offset: u64,
}

impl<Storage> SectorWriter<Storage>
where
  Storage: std::io::Write + std::io::Seek,
{
  pub fn new(storage: Storage, sector_offset: u64, sector_size: u64) -> Self {
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
  pub fn write_aligned(&mut self, buf: &[u8]) -> std::io::Result<usize> {
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
