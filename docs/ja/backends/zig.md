# Zig バックエンド

Fish は Zig ビルドスクリプトおよび C/C++ クロスコンパイルのための高速オーケストレーションを提供します。

## 自動検出
Fish は以下のファイルから Zig プロジェクトを自動検出します：
- `build.zig`
- `build.zig.zon`

## サポートされるコマンド
```bash
fish build     # zig build の実行
fish test      # zig build test の実行
fish check     # Zig 構文および AST の検証
```

## `fish.toml` の設定
```toml
backend = "zig"
jobs = 8
```
