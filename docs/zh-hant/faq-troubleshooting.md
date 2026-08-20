# 常見問題與疑難排解

> 🌐 **翻譯與貢獻：** 想用您的母語翻譯或完善本文件？請參閱 [翻譯指南](TRANSLATION.md)。

## 常見問題

### 1. Fish 與 Cargo、Turborepo 或 Bazel 有何差別？
Fish 專為多語言大型單體儲存庫打造，兼具 Rust 原生極致執行效率、Python AI 智慧最佳化以及 Go 雲原生分散式網路，無需 Bazel 複雜的規則設定即可開箱即用。

### 2. Fish 支援哪些後端語言？
目前 Fish 官方支援 11 種主流語言與工具鏈：Rust、Go、TypeScript/Node.js、Python、C/C++、Docker、Java、.NET、Swift、Dart 以及 Zig。

### 3. 如何檢查目前機器的開發工具鏈？
執行以下指令即可：
```bash
fish doctor --ai
```

## 疑難排解

### Windows 分頁檔耗盡錯誤 (`os error 1455`)
- **原因:** 並行編譯過多大型巨集或重型依賴佔滿分頁檔。
- **解決方案:** 透過 `--jobs` 限制並行數：
```bash
fish build --jobs 4
```
