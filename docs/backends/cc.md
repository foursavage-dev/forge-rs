# C/C++ Backend Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../../TRANSLATION.md).

Forge coordinates C and C++ projects using modern compilers (GCC, Clang, MSVC) and CMake.

---

## Detection & Discovery

Forge detects C/C++ projects by looking for `CMakeLists.txt`, `Makefile`, `meson.build`, or root C/C++ header/source structures.

---

## Fast Linker Integration

Forge automatically queries your environment for modern high-speed linkers:
- **Linux:** Automatically uses `mold` or `ld.lld` via `-fuse-ld=mold`.
- **Windows:** Uses `lld-link` or MSVC linker.
- **macOS:** Uses `ld64.lld` or Apple `ld`.

---

## Response File Support

When compiling large C/C++ projects with thousands of source files, Forge writes argument vectors exceeding OS length limits into `@forge_args.rsp` response files automatically.
