#![no_std]
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
struct ElfHeaderLayout {
    magic: [u8; 4],
    endian: u8,
    header_version: u8,
    os_abi: u8,
    padding: u8,
    elf_type: u16,
    instruction_set: u16,
    elf_version: u32,
    entry_offset: u64,
    program_header_table_offset: u64,
    section_header_table_offset: u64,
    flags: u32,
    elf_header_size: u16,
    program_header_table_entry_size: u16,
    program_header_table_entry_count: u16,
    section_header_table_entry_size: u16,
    section_header_table_entry_count: u16,
    string_table_section_header_index: u16,
}
struct ProgramHeaderLayout {
    segment_type: u32,
    flags: u32,
    data_offset: u64,
    virtual_address: u64,
    physical_address: u64,
    size_in_file: u64,
    size_in_memory: u64,
    alignment: u64,
}