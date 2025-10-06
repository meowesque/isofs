# ISO 9660 + Joliet + El Torito Compliance Roadmap

## Phase 1: Core ISO 9660 Compliance

### Directory Structure Fixes
- [X] Add mandatory "." (current directory) entries to all directories
- [X] Add mandatory ".." (parent directory) entries to all directories
- [X] Fix data_length calculation to include "." and ".." entries
- [ ] Implement proper sector alignment and padding for directory records
- [ ] Ensure directory records don't span sector boundaries

### Path Tables
- [X] Generate path table entries for all directories
- [X] Serialize path tables (both little endian and big endian)
- [X] Calculate correct path table sizes
- [X] Link path tables to Primary Volume Descriptor
- [X] Implement PathTableRecord creation from directory structure

### File System Structure
- [X] Fix file identifier encoding (;1 version suffixes)
- [X] Implement proper file record ordering (directories first, then files)
- [X] Add support for deep directory hierarchies (>8 levels)
- [ ] Handle long filenames (>31 characters) with proper truncation

### Volume Descriptors
- [X] Complete Primary Volume Descriptor calculations
  - [X] Calculate volume_space_size from total sectors needed
  - [X] Set proper volume_set_size and volume_sequence_number  
  - [X] Calculate and set path_table_size correctly
  - [X] Set correct path table LBA locations
- [X] Add proper volume creation timestamps
- [ ] Implement volume set identifiers
- [ ] Add copyright and bibliographic file references

## Phase 2: Joliet Extension Support

### Unicode Support
- [ ] Implement UCS-2 encoding for filenames
- [ ] Add Supplementary Volume Descriptor (SVD) generation
- [ ] Support Unicode directory and file names up to 64 characters
- [ ] Handle Unicode normalization

### Joliet Directory Records
- [ ] Implement SupplementaryVolumeDescriptor serialization (struct exists)
- [ ] Generate parallel Joliet directory structure
- [ ] Implement Joliet-specific file identifier encoding (UCS-2)
- [ ] Support longer filenames without 8.3 restrictions
- [ ] Add Joliet path tables
- [ ] Add escape sequences for UCS-2 encoding

### Compatibility
- [ ] Ensure both ISO 9660 and Joliet structures coexist
- [ ] Maintain backward compatibility with ISO 9660-only readers
- [ ] Test with Windows, macOS, and Linux systems

## Phase 3: El Torito Bootable Media

### Boot Catalog
- [ ] Implement ElToritoBootRecordVolumeDescriptor serialization (struct exists)
- [ ] Generate Boot Catalog structure using existing El Torito types
- [ ] Support multiple boot entries with validation
- [ ] Add validation entry and section headers
- [ ] Implement boot catalog checksum calculation

### Boot Images
- [ ] Support floppy disk emulation (1.2MB, 1.44MB, 2.88MB)
- [ ] Support hard disk emulation
- [ ] Support "no emulation" mode for modern bootloaders
- [ ] Handle boot image loading and validation

### Platform Support
- [ ] Add x86 platform support
- [ ] Add EFI platform support
- [ ] Support multiple architectures in single image
- [ ] Implement proper boot indicator flags

## Phase 4: Advanced Features

### Performance & Optimization
- [ ] Implement streaming write operations
- [ ] Add multi-threading support for large ISOs
- [ ] Optimize memory usage for large file systems
- [ ] Add progress reporting for long operations

### Rock Ridge Extensions
- [ ] Add POSIX file attributes support
- [ ] Implement symbolic link support
- [ ] Support longer filenames (>255 characters)
- [ ] Add device file support

### Additional Standards
- [ ] ISO 9660:1999 (Level 3) support for >4GB files
- [ ] UDF bridge format support
- [ ] Apple ISO 9660 extensions (HFS+ bridge)

## Phase 5: Testing & Validation

### Compliance Testing
- [ ] Test against official ISO 9660 test suites
- [ ] Validate with multiple OS mount implementations
- [ ] Test bootability on physical and virtual hardware
- [ ] Verify Unicode handling across different systems

### Real-world Testing
- [ ] Test with large directory structures (>1000 files)
- [ ] Validate deep directory hierarchies
- [ ] Test mixed file types (binary, text, executables)
- [ ] Performance benchmarking against other ISO tools

### Compatibility Matrix
- [ ] Windows 95/98/ME/NT/2000/XP/Vista/7/8/10/11
- [ ] macOS (all versions with ISO support)
- [ ] Linux distributions (major filesystems)
- [ ] FreeBSD, OpenBSD, NetBSD
- [ ] CD/DVD burning software compatibility

## Implementation Priority

1. **Critical (Phase 1)**: **COMPLETED** - Core ISO 9660 compliance achieved
2. **High (Phase 2)**: Joliet Unicode support for modern compatibility  
3. **Medium (Phase 3)**: El Torito bootable media support
4. **Low (Phase 4-5)**: Advanced features and comprehensive testing

## Current Status

### Completed
- [x] **Core ISO 9660 structure types** (spec.rs - all major structs defined)
- [x] **Directory hierarchy creation and upserting logic**
- [x] **File content writing with LBA allocation**
- [x] **Primary Volume Descriptor serialization** (IsoSerialize impl exists)
- [x] **Supplementary Volume Descriptor serialization** (IsoSerialize impl exists)
- [x] **Directory Record serialization** for files and directories  
- [x] **Root Directory Record generation**
- [x] **Basic file system writing** (sectors, LBA allocation)
- [x] **Volume descriptor set terminator**
- [x] **File and directory entry management**
- [x] **Mandatory "." and ".." directory entries**
- [x] **Complete directory structure traversal and writing**
- [x] **Path table generation and serialization** (both LE and BE)
- [x] **Primary Volume Descriptor calculations** (volume size, path table locations)
- [x] **Working ISO 9660 filesystem generation** (verified mountable ISO)

### Partially Implemented
- [PARTIAL] **Timestamp handling** (chrono integration exists but some conversions missing)
- [PARTIAL] **File identifier encoding** (basic ;1 versioning exists, needs refinement for edge cases)

### Defined But Not Implemented
- [x] **Supplementary Volume Descriptor** (struct exists, serialization implemented)
- [x] **Path Table Records** (struct exists, generation and serialization completed)
- [TODO] **El Torito structures** (all structs exist, no serialization)
- [TODO] **Joliet Extensions** (types defined, no implementation)
- [TODO] **Extended Attribute Records** (struct exists, no usage)
- [TODO] **Rock Ridge Extensions** (not started)

### In Progress
- [WIP] **Long filename handling** (>31 characters truncation)
- [WIP] **Joliet Unicode filename support** (SVD serialization exists, need parallel directory structure)

### Recently Completed
- [x] **Directory structure layout** (all directories and subdirectories now appear correctly)
- [x] **"." and ".." entries implementation** (working as confirmed by libcdio)
- [x] **Directory hierarchy creation and file placement**
- [x] **Path table generation and serialization** (both little-endian and big-endian tables)
- [x] **Primary Volume Descriptor calculations** (volume size, path table locations, proper LBA allocation)
- [x] **Working ISO 9660 filesystem** (generates valid ISO files)
- [x] **IsoSerialize trait context parameter migration** (all implementations updated)

### Known Issues
- **Reader/parser functionality** is stubbed (parse.rs has skeleton only)
- **Long filename handling** needs refinement (>31 character truncation edge cases)
- **No validation of generated ISOs** against official test suites yet
- **Generated ISOs may not mount properly** - only subset of filesystem structure validated

### Not Started
- **Joliet Unicode filename support** (SVD serialization exists, need parallel directory structure)
- **El Torito bootable media** (structs exist, need serialization and boot catalog)
- **Multi-volume support**
- **UDF bridge format**
- **Comprehensive testing framework**
- **Rock Ridge POSIX extensions**

