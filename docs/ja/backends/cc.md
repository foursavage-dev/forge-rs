# C / C++ バックエンド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](../TRANSLATION.md) をご覧ください。

C/C++ バックエンドは CMake、Make、Clang、GCC、MSVC をサポートし、Clangd 用の `compile_commands.json` を自動生成します。

## 自動検出
`CMakeLists.txt` または `Makefile` が存在する場合に検出されます。

## タスク
- **設定**: `cmake -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON`
- **ビルド**: `cmake --build build --config Release`
- **テスト**: `ctest --test-dir build`
