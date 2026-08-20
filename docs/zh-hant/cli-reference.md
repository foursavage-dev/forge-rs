# CLI 命令列參考

> 🌐 **翻譯與貢獻：** 想用您的母語翻譯或完善本文件？請參閱 [翻譯指南](TRANSLATION.md)。

Fish 命令列完整指令與參數指南。

## 基礎指令

| 指令 | 描述 |
| :--- | :--- |
| `fish init` | 在目前目錄下初始化 `fish.toml` 設定檔 |
| `fish new <name>` | 基於預設範本建立新專案或套件 |
| `fish build` | 編譯建置工作區中所有目標任務 |
| `fish check` | 執行快速語法與型別檢查 |
| `fish test` | 並行執行所有單元與整合測試 |
| `fish clean` | 清理建置產生物與本機快取 |

## AI 智慧指令

```bash
# 分析建置錯誤日誌
fish ai analyze --toolchain rust --stderr "<log_content>"

# 最佳化任務排程圖
fish ai optimize --workers 8

# 推薦需建置的目標套件
fish ai recommend
```

## 網路與分散式指令

```bash
# 啟動遠端快取伺服端
fish cache-server --listen 0.0.0.0:8080

# 啟動分散式 Worker 節點
fish worker --coordinator http://coordinator:9090
```

## 診斷與輔助工具

```bash
# 執行環境相依全面檢查
fish doctor --ai

# 查詢套件相依關係
fish query "deps(//packages/core)"

# 檔案變更即時監聽建置
fish watch
```
