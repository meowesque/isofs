#[derive(Debug)]
#[repr(transparent)]
pub struct IsoPath(str);

impl IsoPath {
  pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
    unsafe { &*(s.as_ref() as *const str as *const IsoPath) }
  }

  /// Returns the components of this path as an iterator.
  pub fn components<'a>(&'a self) -> Components<'a> {
    Components { path: &self.0 }
  }
}

impl AsRef<IsoPath> for str {
  fn as_ref(&self) -> &IsoPath {
    IsoPath::new(self)
  }
}

pub struct Components<'a> {
  path: &'a str,
}

impl<'a> Iterator for Components<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    if self.path.is_empty() {
      return None;
    }

    if let Some(pos) = self.path.find(['/', '\\']) {
      let part = &self.path[..pos];
      self.path = &self.path[pos + 1..];
      Some(part)
    } else {
      let part = self.path;
      self.path = "";
      Some(part)
    }
  }
}
