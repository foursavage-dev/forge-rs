# C / C++ Backend

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](../TRANSLATION.md).

C/C++ Backend hỗ trợ CMake, Make, Clang, GCC và MSVC, đồng thời tự động sinh `compile_commands.json` cho Clangd.

## Phát hiện Dự án
Được phát hiện khi có `CMakeLists.txt` hoặc `Makefile`.

## Các Tác vụ
- **Configure**: `cmake -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON`
- **Build**: `cmake --build build --config Release`
- **Test**: `ctest --test-dir build`
