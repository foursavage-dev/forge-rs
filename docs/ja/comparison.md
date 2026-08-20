# 比較マトリックス：Fish と他のビルドシステム

Fish は、現代のポリグロット Monorepo 向けに Rust 2024 で設計されたビルドオーケストレーションシステムです。以下は Bazel、Turborepo、Buck2 との包括的な機能比較表です：

| 機能 / 項目 | Fish | Turborepo | Bazel | Buck2 |
| :--- | :--- | :--- | :--- | :--- |
| **実装言語** | Rust 2024 | Go / Rust | Java / C++ | Rust |
| **対応言語** | ネイティブ 11+ 言語ツールチェーン | JS / TS 中心 | ポリグロット (Starlark) | ポリグロット (Starlark) |
| **設定モデル** | 統一 `fish.toml` / 自動検出 | `turbo.json` | `WORKSPACE` + `BUILD.bazel` | `BUCK` ファイル |
| **導入難易度** | 極めて低い（ゼロ構成） | 低い | 非常に高い | 高い |
| **ハッシュエンジン** | Blake3（超高速マルチスレッド） | SHA-256 | SHA-256 | Blake3 / SHA-256 |
| **CAS 圧縮** | Zstandard (Zstd) + CoW | Tar.gz / Gzip | Zstd / カスタム | Zstd / カスタム |
| **成果物マテリアライズ** | Reflink / Copy-on-Write (0ms) | ファイルコピー | シンボリック/ハードリンク | Reflink / CoW |
| **ブロック重複排除** | FastCDC (16KB - 256KB) | アーカイブ全体 | アーカイブ全体 | チャンク CAS |
| **VFS 解析速度** | インメモリ状態ツリー (<2ms) | ディスク走査 | Inotify / Watchman | Watchman / EdenFS |
| **セマンティック無効化**| AST 公開インターフェースハッシュ | ファイル全体のハッシュ | ヘッダーのみコンパイル | Header / rmeta |
| **AI 診断機能** | ネイティブ IPC + 原因解析 | なし | なし | なし |
| **ダッシュボード** | 組み込み Web GUI & TUI | Vercel Web App | サードパーティ製 | オープンソースコンソール |
