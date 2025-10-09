# isofs

![Crates.io Version](https://img.shields.io/crates/v/isofs)

`isofs` is a library for manipulating `.iso` files (ie. ISO 9660) and UDF filesystems. 

```rs
use isofs::writer::*;

fn main() -> Result<(), isofs::error::Error> {
  let mut writer = IsoWriter::new(IsoWriterOptions::compatibility());

  writer.upsert_filesystem(
    Filesystem::capture("Documents", "~/Documents")?,
    &OnFileConflict::Overwrite,
  )?;

  writer.finalize(std::fs::File::create("my-documents.iso")?)?;

  Ok(())
}
```

## Feature Flags

* `chrono` Enables conversion with [chrono](https://crates.io/crates/chrono) types.
* `time` Enables conversion with [time](https://crates.io/crates/time) types. 

## References

* [Rock Ridge](https://people.freebsd.org/~emaste/rrip112.pdf)
* [Joliet](https://pismotec.com/cfs/jolspec.html)
* [El Torito](https://pdos.csail.mit.edu/6.828/2014/readings/boot-cdrom.pdf)
* [ECMA-119](https://ecma-international.org/wp-content/uploads/ECMA-119_5th_edition_december_2024.pdf)
* ISO/IEC 9660:2023 
* [OSDev Wiki](https://wiki.osdev.org/ISO_9660)
* [isofs](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/fs/isofs)

## Acknowledgements

* [libcdio](https://github.com/libcdio/libcdio)

## License

Copyright © 2025 Maxine DeAndrade. All rights reserved.

This work is licensed under the terms of the MIT license.  
For a copy, see <https://opensource.org/licenses/MIT>.
