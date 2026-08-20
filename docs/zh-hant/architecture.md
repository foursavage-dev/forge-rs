# Fish 系統架構設計

> 🌐 **翻譯與貢獻：** 想用您的母語翻譯或完善本文件？請參閱 [翻譯指南](TRANSLATION.md)。

Fish 採用專為現代化大型代碼庫（Monorepo）打造的 **三引擎架構 (Rust + Python + Go)**，兼具極致建置速度、雲原生分散式擴展以及 AI 智慧化分析。

## 三引擎架構概覽

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI 命令列                          │
│                      (crates/fish-cli)                      │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌──────────────────────────────┐ ┌────────────────────────────┐
│      Rust 執行核心 (75%)     │ │      Go 網路排程服務 (10%)  │
│  - fish-core, fish-graph     │ │  - fish-coordinator       │
│  - fish-executor, scheduler  │ │  - fish-worker-gateway    │
│  - fish-cache, fish-cas      │ │  - fish-network, migrator │
└──────────────┬───────────────┘ └────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Python AI 智慧服務 (15%)                │
│   - fish_ai_analyzer   - fish_optimizer                     │
│   - fish_analytics     - fish_recommender                   │
└─────────────────────────────────────────────────────────────┘
```

### 1. Rust 核心執行引擎 (75%)
- **`fish-core`**: 工作區自動探索、設定清單解析與微檔案過濾。
- **`fish-graph`**: 有向無環圖（DAG）、拓撲排序與代數相依查詢引擎。
- **`fish-executor`**: 程序控制、沙箱隔離與中介軟體管線。
- **`fish-scheduler`**: 基於 GNU Jobserver 的高並行工作竊取排程器。
- **`fish-cache` & `fish-cas`**: Blake3 多層指紋快取與 ZSTD 壓縮儲存。

### 2. Python AI 智慧層 (15%)
- **`fish_ai_analyzer`**: 建置失敗日誌分類、根因定位與修復建議。
- **`fish_optimizer`**: 關鍵路徑（Critical Path）計算與記憶體約束排程。
- **`fish_analytics`**: 建置耗時遙測聚合與瓶頸偵測。
- **`fish_recommender`**: 變更影響分析與不穩定測試（Flaky Tests）偵測。

### 3. Go 雲原生網路層 (10%)
- **`fish-coordinator`**: 節點註冊中心、心跳監控與分散式任務派送。
- **`fish-worker-gateway`**: 高效能反向代理與 Least-Loaded 負載平衡。
- **`fish-network`**: 連線池管理與 mTLS 傳輸安全。
- **`fish-db-migrator`**: 遙測資料庫版本遷移工具。
