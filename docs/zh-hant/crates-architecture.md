# 36 Crates 核心工作區架構 (`crates/`)

Fish 由 36 個高度模組化的 Rust Crates 組成，分層嚴謹且高內聚低耦合。

## 架構分層
1. **基礎層 (Foundation Tier)**:
   - `fish-core`: 專案探測、Manifest 解析、`fish.toml` 設定。
   - `fish-graph`: DAG 圖模型、無鎖拓撲排序、圖查詢代數。
   - `fish-executor`: OS 子行程執行、`@args.rsp` 回應參數檔案、中介軟體鏈。
2. **儲存與快取層 (Storage & Cache Tier)**:
   - `fish-cas`: ZSTD 壓縮與 FastCDC 分塊的 CAS 儲存。
   - `fish-cache`: 雙階段 Fingerprint 快取與 GC 清理。
   - `fish-remote-cache`: gRPC REAPI v2 客戶端與 TCP 串流快取。
3. **調度與執行層 (Scheduling Tier)**:
   - `fish-scheduler`: 關鍵路徑動態預測調度器、Chase-Lev 工作竊取隊列、GNU Jobserver。
   - `fish-worker`: 遠端 Worker 叢集執行與 Daemon IPC。
   - `fish-sandbox`: Linux eBPF 系統呼叫追蹤與 WASM 沙箱。
4. **11 種語言後端適配器**:
   - Rust, C++, Go, TS, Python, Docker, Java, .NET, Swift, Dart, Zig。
5. **安全與診斷工具層**:
   - `fish-security`, `fish-signing`, `fish-secrets`, `fish-flaky-detection`, `fish-notifications`, `fish-analytics`, `fish-templates`, `fish-docker-builder`, `fish-incremental`, `fish-multiplatform`, `fish-installer`。
6. **頂層 CLI 應用**:
   - `fish-cli`: 統一 CLI 入口、TUI 瀑布流儀表板與 `fish lsp` 語言服務端。
