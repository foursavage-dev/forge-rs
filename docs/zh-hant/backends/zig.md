# Zig 後端

Fish 為 Zig 建置腳本和 C/C++ 交叉編譯工具鏈提供零開銷的建置協調。

## 自動偵測
Fish 透過以下檔案自動識別 Zig 專案：
- `build.zig`
- `build.zig.zon`

## 支援的命令
```bash
fish build     # 執行 zig build
fish test      # 執行 zig build test 測試
fish check     # 校驗 Zig 語法與 AST
```

## `fish.toml` 設定
```toml
backend = "zig"
jobs = 8
```
