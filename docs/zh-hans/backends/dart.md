# Dart & Flutter 后端

Fish 支持 Dart 命令行包和多平台 Flutter 应用的协调构建与高速缓存。

## 自动检测
Fish 通过以下文件自动识别 Dart/Flutter 项目：
- `pubspec.yaml`
- `pubspec.lock`

## 支持的命令
```bash
fish build     # 编译 Dart AOT 或 Flutter 构建产物
fish test      # 执行 dart test / flutter test
fish check     # 执行 dart analyze 静态分析
```

## `fish.toml` 配置
```toml
backend = "dart"
jobs = 4
```
