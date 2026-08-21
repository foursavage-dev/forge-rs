# Swift & Objective-C 后端

Fish 为 macOS、iOS 和 Linux 上的 Swift 与 Objective-C 项目提供一流的原生支持。

## 自动检测
Fish 通过以下文件自动识别 Swift 项目：
- `Package.swift` (Swift Package Manager)
- `*.xcodeproj` / `*.xcworkspace` (Xcode 项目)

## 支持的命令
```bash
fish build     # 使用 swift build 编译 SwiftPM 模块
fish test      # 执行 XCTest 测试套件
fish check     # 执行 swiftc 语法和类型检查
```

## `fish.toml` 配置
```toml
backend = "swift"
jobs = 4

[cache]
enabled = true
```
