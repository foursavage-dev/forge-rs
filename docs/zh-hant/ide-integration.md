# IDE 整合與開發者工具

Fish 為 VS Code、JetBrains 全家桶及 LSP 協定提供一流的開發者整合體驗。

## VS Code 外掛
從 `vscode-extension/` 安裝官方擴充套件：
- **互動式 DAG 圖形視覺化**: 基於 Webview 的任務相依拓撲圖。
- **一鍵式任務執行**: 直接在側邊欄執行 build、test、check。
- **即時語法診斷**: 錯誤即時標記與代碼補全。

## JetBrains 外掛套件
位於 `jetbrains-plugin/`，支援 IntelliJ IDEA、CLion、RustRover、PyCharm、Rider：
- **Fish ToolWindow**: 任務管理樹與執行器。
- **LSP 橋接**: `fish.toml` 設定智能導航與補全。

## Language Server Protocol (LSP)
Fish 內建 LSP 伺服端：
```bash
fish lsp
```
支援 Neovim、Helix、Emacs 等任意支援 LSP 的編輯器。
