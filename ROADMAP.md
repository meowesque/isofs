# ISO 9660 + Joliet + El Torito Compliance Roadmap

## Phase 1: Core ISO 9660 Compliance

### Directory Structure Fixes
- [X] Add mandatory "." (current directory) entries to all directories
- [X] Add mandatory ".." (parent directory) entries to all directories
- [X] Fix data_length calculation to include "." and ".." entries
- [ ] Implement proper sector alignment and padding for directory records
- [ ] Ensure directory records don't span sector boundaries

### Path Tables
- [ ] Generate path table entries for all directories  
- [ ] Serialize path tables (both little endian and big endian)
- [ ] Calculate correct path table sizes
- [ ] Link path tables to Primary Volume Descriptor
- [ ] Implement PathTableRecord creation from directory structure

### File System Structure
- [ ] Fix file identifier encoding (;1 version suffixes)
- [ ] Implement proper file record ordering (directories first, then files)
- [ ] Add support for deep directory hierarchies (>8 levels)
- [ ] Handle long filenames (>31 characters) with proper truncation

### Volume Descriptors
- [ ] Complete Primary Volume Descriptor calculations
  - [ ] Calculate volume_space_size from total sectors needed
  - [ ] Set proper volume_set_size and volume_sequence_number  
  - [ ] Calculate and set path_table_size correctly
  - [ ] Set correct path table LBA locations
- [ ] Add proper volume creation timestamps
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

1. **Critical (Phase 1)**: Fix basic directory structure and "." ".." entries
2. **High (Phase 1-2)**: Complete ISO 9660 + Joliet for broad compatibility  
3. **Medium (Phase 3)**: El Torito for bootable media creation
4. **Low (Phase 4-5)**: Advanced features and comprehensive testing

## Current Status

### Completed
- [x] Basic ISO 9660 structure types (spec.rs - all major structs defined)
- [x] Directory hierarchy creation and upserting logic
- [x] File content writing with LBA allocation
- [x] Primary Volume Descriptor serialization (IsoSerialize impl exists)
- [x] Directory Record serialization for files and directories  
- [x] Root Directory Record generation
- [x] Basic file system writing (sectors, LBA allocation)
- [x] Volume descriptor set terminator
- [x] File and directory entry management

### Partially Implemented
- [⚠️] Primary Volume Descriptor creation (basic fields, but missing calculated values)
  - Missing: volume_space_size, volume_set_size, volume_sequence_number calculations
  - Missing: proper path table locations and sizes
- [⚠️] Timestamp handling (chrono integration exists but some conversions missing)
- [⚠️] File identifier encoding (basic ;1 versioning exists, needs refinement)

### Defined But Not Implemented
- [📝] Supplementary Volume Descriptor (struct exists, no serialization)
- [📝] Path Table Records (struct exists, no generation/serialization)
- [📝] El Torito structures (all structs exist, no serialization)
- [📝] Joliet Extensions (types defined, no implementation)
- [📝] Extended Attribute Records (struct exists, no usage)
- [📝] Rock Ridge Extensions (not started)

### In Progress
- [ ] Directory structure fixes (current issue: only deepest paths showing)
- [ ] "." and ".." entries implementation

### Known Issues
- Directory layout only shows deepest paths in mount tests
- Missing mandatory "." and ".." directory entries  
- Incorrect data_length calculations in directory records
- Path tables not generated or serialized
- PrimaryVolumeDescriptor has placeholder values instead of calculated ones
- No volume space size calculation
- Reader/parser functionality is stubbed (parse.rs has skeleton only)

### Not Started
- Joliet Unicode filename support
- El Torito bootable media
- Path table generation and writing
- Multi-volume support
- UDF bridge format
- Comprehensive testing framework
