# Dart & Flutter 後端

Fish 支援 Dart 命令列套件與多平台 Flutter 應用的協調建置與高速快取。

## 自動偵測
Fish 透過以下檔案自動識別 Dart/Flutter 專案：
- `pubspec.yaml`
- `pubspec.lock`

## 支援的命令
```bash
fish build     # 編譯 Dart AOT 或 Flutter 建置產物
fish test      # 執行 dart test / flutter test
fish check     # 執行 dart analyze 靜態分析
```

## `fish.toml` 設定
```toml
backend = "dart"
jobs = 4
```
