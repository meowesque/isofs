#!/usr/bin/env python3
"""
Path Table Verification Script for ISO 9660 files
Usage: python3 verify-path-table.py <iso-file>
"""

import sys
import struct

def read_pvd(f):
  """Read Primary Volume Descriptor and extract path table info"""
  f.seek(16 * 2048)  # Start at sector 16
  
  while True:
    sector_data = f.read(2048)
    if len(sector_data) < 2048:
      break
      
    # Check for volume descriptor
    if sector_data[0] == 1 and sector_data[1:6] == b'CD001':
      # Primary Volume Descriptor found
      # Path table size at offset 0x84 (LE) 
      pt_size = struct.unpack('<I', sector_data[0x84:0x88])[0]
      # Path table LBA at offset 0x8C (LE) and 0x94 (BE, stored as big-endian)
      pt_lba_le = struct.unpack('<I', sector_data[0x8C:0x90])[0]
      pt_lba_be = struct.unpack('>I', sector_data[0x94:0x98])[0]
      
      return pt_size, pt_lba_le, pt_lba_be
    elif sector_data[0] == 255:  # Volume descriptor set terminator
      break
  
  return None, None, None

def decode_path_table(f, lba, size, big_endian=False):
  """Decode path table entries"""
  f.seek(lba * 2048)
  data = f.read(size)
  
  entries = []
  offset = 0
  
  while offset < len(data) and offset < size:
    if offset + 8 > len(data):
      break
      
    dir_id_len = data[offset]
    if dir_id_len == 0:
      break
      
    ext_attr_len = data[offset + 1]
    
    if big_endian:
      extent_lba = struct.unpack('>I', data[offset + 2:offset + 6])[0]
      parent_dir = struct.unpack('>H', data[offset + 6:offset + 8])[0]
    else:
      extent_lba = struct.unpack('<I', data[offset + 2:offset + 6])[0]
      parent_dir = struct.unpack('<H', data[offset + 6:offset + 8])[0]
    
    # Read directory identifier
    dir_name = data[offset + 8:offset + 8 + dir_id_len].decode('ascii', errors='ignore')
    if dir_id_len == 1 and data[offset + 8] == 0:
      dir_name = "<ROOT>"
    
    entries.append({
      'dir_id_len': dir_id_len,
      'ext_attr_len': ext_attr_len,
      'extent_lba': extent_lba,
      'parent_dir': parent_dir,
      'dir_name': dir_name
    })
    
    # Move to next entry (with padding)
    entry_len = 8 + dir_id_len + (dir_id_len % 2)
    offset += entry_len
  
  return entries

def main():
  if len(sys.argv) != 2:
    print("Usage: python3 verify-path-table.py <iso-file>")
    sys.exit(1)
  
  iso_file = sys.argv[1]
  
  with open(iso_file, 'rb') as f:
    pt_size, pt_lba_le, pt_lba_be = read_pvd(f)
    
    if pt_size is None:
      print("Error: Could not find Primary Volume Descriptor")
      sys.exit(1)
    
    print(f"Path Table Info from PVD:")
    print(f"  Size: {pt_size} bytes")
    print(f"  Little-endian LBA: {pt_lba_le}")
    print(f"  Big-endian LBA: {pt_lba_be}")
    print()
    
    # Decode little-endian path table
    print("Little-Endian Path Table:")
    le_entries = decode_path_table(f, pt_lba_le, pt_size, False)
    for i, entry in enumerate(le_entries, 1):
      print(f"  {i:2d}: len={entry['dir_id_len']:2d} extent={entry['extent_lba']:3d} parent={entry['parent_dir']:2d} name='{entry['dir_name']}'")
    
    print()
    
    # Decode big-endian path table
    print("Big-Endian Path Table:")
    be_entries = decode_path_table(f, pt_lba_be, pt_size, True)
    for i, entry in enumerate(be_entries, 1):
      print(f"  {i:2d}: len={entry['dir_id_len']:2d} extent={entry['extent_lba']:3d} parent={entry['parent_dir']:2d} name='{entry['dir_name']}'")
    
    # Verify consistency
    print()
    print("Consistency Check:")
    if len(le_entries) == len(be_entries):
      print(f"SUCCESS Both tables have {len(le_entries)} entries")
      
      all_match = True
      for i, (le, be) in enumerate(zip(le_entries, be_entries)):
        if (le['dir_id_len'] != be['dir_id_len'] or 
          le['extent_lba'] != be['extent_lba'] or 
          le['parent_dir'] != be['parent_dir'] or 
          le['dir_name'] != be['dir_name']):
          print(f"ERROR Entry {i+1} mismatch")
          all_match = False
      
      if all_match:
        print("SUCCESS All entries match between LE and BE tables")
    else:
      print(f"ERROR Entry count mismatch: LE={len(le_entries)}, BE={len(be_entries)}")

if __name__ == '__main__':
  main()
