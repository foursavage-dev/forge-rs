# Swift & Objective-C 後端

Fish 為 macOS、iOS 和 Linux 上的 Swift 與 Objective-C 專案提供一流的原生支援。

## 自動偵測
Fish 透過以下檔案自動識別 Swift 專案：
- `Package.swift` (Swift Package Manager)
- `*.xcodeproj` / `*.xcworkspace` (Xcode 專案)

## 支援的命令
```bash
fish build     # 使用 swift build 編譯 SwiftPM 模組
fish test      # 執行 XCTest 測試套件
fish check     # 執行 swiftc 語法與型別檢查
```

## `fish.toml` 設定
```toml
backend = "swift"
jobs = 4

[cache]
enabled = true
```
