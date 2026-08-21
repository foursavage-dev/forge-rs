# Swift & Objective-C バックエンド

Fish は macOS、iOS、Linux 上の Swift および Objective-C プロジェクトをネイティブにサポートします。

## 自動検出
Fish は以下のファイルから Swift プロジェクトを自動検出します：
- `Package.swift` (Swift Package Manager)
- `*.xcodeproj` / `*.xcworkspace` (Xcode プロジェクト)

## サポートされるコマンド
```bash
fish build     # swift build による SwiftPM モジュールのビルド
fish test      # XCTest テストスイートの実行
fish check     # swiftc による構文および型チェック
```

## `fish.toml` の設定
```toml
backend = "swift"
jobs = 4

[cache]
enabled = true
```
