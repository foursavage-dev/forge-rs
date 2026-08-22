# Zig 后端

Fish 为 Zig 构建脚本和 C/C++ 交叉编译工具链提供零开销的构建协调。

## 自动检测
Fish 通过以下文件自动识别 Zig 项目：
- `build.zig`
- `build.zig.zon`

## 支持的命令
```bash
fish build     # 执行 zig build
fish test      # 执行 zig build test 测试
fish check     # 校验 Zig 语法与 AST
```

## `fish.toml` 配置
```toml
backend = "zig"
jobs = 8
```
