pub struct LbaAllocator {
  sector_size: u32,
  next_lba: u32,
}

impl LbaAllocator {
  pub fn new(sector_size: u32, offset: u32) -> Self {
    Self {
      sector_size,
      next_lba: offset,
    }
  }

  pub fn allocate(&mut self, size: u32) -> u32 {
    let lba = self.next_lba;
    let sectors = (size + self.sector_size - 1) / self.sector_size;
    self.next_lba += sectors;
    lba
  }
}
