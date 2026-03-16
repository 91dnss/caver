# Changelog

## [0.3.0] - 2026-16-03

### Added
- `CaveInfo::symbol` — resolved symbol name as written into the ELF symbol table
- `ElfFile::find_caves()` — scan for existing NOP/zero sleds above a size threshold

### Changed
- Removed public inspection API (`sections`, `segments`, `symbols`, `is_stripped`)
  — `find_caves` is the only inspection surface remaining public

### Fixed
- AArch64 cave size alignment validation
- VMA overlap detection across segments
- Symbol name collision detection in `inject_many`
