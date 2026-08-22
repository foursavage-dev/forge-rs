# 更新日誌與版本歷史

此處記錄 Fish 專案的所有重要變更。

## [v0.3.0] - 2026-08-21
### 新增特性
- **IDE 擴充**: 官方 VS Code 擴充套件與 JetBrains 全家桶外掛。
- **LSP 協定支援**: `fish lsp` 語言服務端與即時診斷。
- **gRPC REAPI v2**: 分散式 Remote Execution API v2 動作執行。
- **eBPF 追蹤**: Linux 核心級動態相依發現。
- **Doctor AI**: 智慧環境自愈診斷 (`fish doctor --fix`)。
- **TUI 瀑布流儀表板**: 即時 CPU/RAM 與任務執行視覺化。

## [v0.2.0] - 2026-08-10
### 新增特性
- **Tri-Engine 架構**: Rust 2024 核心 + Go 協調器 + Python AI 引擎。
- **11 種語言後端**: Rust, Go, TS, Python, C++, Docker, Java, .NET, Swift, Dart, Zig。
- **BLAKE3 CAS 儲存**: 高效能 ZSTD 內容定址快取。
