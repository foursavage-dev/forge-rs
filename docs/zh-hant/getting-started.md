# 快速開始使用 Fish

> 🌐 **翻譯與貢獻：** 想用您的母語翻譯或完善本文件？請參閱 [翻譯指南](TRANSLATION.md)。

本指南將協助您快速上手 Fish —— 一款高效能、快取優先的通用多語言建置編排系統。

## 安裝指南

### 單行指令快速安裝（推薦）

**Linux 與 macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/requla11/fish/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/requla11/fish/main/install.ps1 | iex
```

### 從原始碼編譯安裝

```bash
git clone https://github.com/requla11/fish.git
cd fish
cargo install --path crates/fish-cli
```

### 透過 Cargo 安裝

```bash
cargo install fish-cli --git https://github.com/requla11/fish
```

## 快速入門

### 建置 Rust 專案

```bash
cd my-rust-project
fish init
fish build
```

### 建置多語言專案 (Polyglot)

```bash
# 全域快速檢查
fish check

# 執行所有測試套件
fish test

# 清理建置快取與產生物
fish clean
```

## 體驗 TUI 終端介面

```bash
fish ui
```

## AI 故障診斷與排程最佳化

```bash
# 使用 AI 分析建置失敗原因
fish ai analyze --toolchain rust --stderr "error[E0308]: mismatched types"

# 最佳化 DAG 排程順序
fish ai optimize --workers 8

# 基於 Git Diff 智慧推薦建置目標
fish ai recommend
```
