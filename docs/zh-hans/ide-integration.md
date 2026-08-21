# IDE 集成与开发者工具

Fish 为 VS Code、JetBrains 全家桶及 LSP 协议提供一流的开发者集成体验。

## VS Code 插件
从 `vscode-extension/` 安装官方扩展：
- **交互式 DAG 图形可视化**: 基于 Webview 的任务依赖拓扑图。
- **一键式任务执行**: 直接在侧边栏运行 build、test、check。
- **实时语法诊断**: 错误即时标记与代码补全。

## JetBrains 插件套件
位于 `jetbrains-plugin/`，支持 IntelliJ IDEA、CLion、RustRover、PyCharm、Rider：
- **Fish ToolWindow**: 任务管理树与执行器。
- **LSP 桥接**: `fish.toml` 配置智能导航与补全。

## Language Server Protocol (LSP)
Fish 内置 LSP 服务端：
```bash
fish lsp
```
支持 Neovim、Helix、Emacs 等任意支持 LSP 的编辑器。
