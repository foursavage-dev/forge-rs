# C / C++ 後端

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

C/C++ 後端支持 CMake, Make, Clang, GCC 和 MSVC，并可自动生成用于 Clangd 的 `compile_commands.json`。

## 自动检测
当存在 `CMakeLists.txt` 或 `Makefile` 时自动启用。

## 任务流
- **配置阶段**: `cmake -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON`
- **构建阶段**: `cmake --build build --config Release`
- **测试阶段**: `ctest --test-dir build`
