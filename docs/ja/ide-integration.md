# IDE 統合と開発ツール

Fish は VS Code、JetBrains IDE、および Language Server Protocol (LSP) 向けの包括的な開発ツールを提供します。

## VS Code 拡張機能
`vscode-extension/` からインストール可能な公式拡張機能：
- **対話型 DAG グラフ表示**: Webview による依存関係の可視化。
- **ワンクリックタスク実行**: サイドバーから build / test / check を直接実行。
- **リアルタイム診断**: インラインエラー表示およびコード補完。

## JetBrains プラグイン
`jetbrains-plugin/` にて IntelliJ IDEA、CLion、RustRover、PyCharm、Rider をサポート：
- **Fish ToolWindow**: 直感的なタスク管理ツールウィンドウ。
- **LSP ブリッジ**: `fish.toml` の自動補完および構文検証。

## Language Server Protocol (LSP)
Fish には内蔵 LSP サーバーが含まれています：
```bash
fish lsp
```
Neovim、Helix、Emacs など任意の LSP クライアントと接続可能です。
