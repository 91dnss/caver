# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-16

### Added
- ELF64 code cave injection via new `PT_LOAD` segment (`inject`, `inject_many`)
- `PatchedElf` result type with `info()`, `infos()`, and `write()` methods
- `CaveOptions` builder API with `size()`, `name()`, `fill()` and `build()`
- `FillByte::ArchNop` and `FillByte::Zero` fill patterns
- Cave exported as `STT_FUNC` symbol (`caverfn_<name>`) for disassembler discovery
- Inspection API on `ElfFile`: `sections()`, `segments()`, `find_caves()`
- `is_executable()`, `is_writable()` on `SectionInfo` and `SegmentInfo`
- Architecture support: x86_64, AArch64, RISC-V 64
- Output validation after injection
- `SHN_XINDEX` fallback for `e_shstrndx` resolution
- `PT_PHDR` segment patching when relocating phdr table
```
