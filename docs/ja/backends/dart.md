# Dart & Flutter バックエンド

Fish は Dart CLI パッケージおよび Flutter マルチプラットフォームアプリの高速ビルドをサポートします。

## 自動検出
Fish は以下のファイルから Dart/Flutter プロジェクトを自動検出します：
- `pubspec.yaml`
- `pubspec.lock`

## サポートされるコマンド
```bash
fish build     # Dart AOT または Flutter アプリのビルド
fish test      # dart test / flutter test の実行
fish check     # dart analyze 静的解析の実行
```

## `fish.toml` の設定
```toml
backend = "dart"
jobs = 4
```
