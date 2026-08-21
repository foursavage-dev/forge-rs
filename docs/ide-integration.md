# IDE Integration & Tooling

Fish provides first-class developer tooling across VS Code, JetBrains IDEs, and Language Server Protocol (LSP).

## VS Code Extension
Install the official Fish VS Code extension from `vscode-extension/`:
- **Interactive DAG Graph Visualizer**: Webview-powered dependency visualization.
- **One-Click Task Execution**: Run build, test, and check directly from the editor sidebar.
- **Real-time Diagnostics**: Inline error reporting and LSP code completion.

## JetBrains Plugin Suite
Located in `jetbrains-plugin/` for IntelliJ IDEA, CLion, RustRover, PyCharm, and Rider:
- **Fish ToolWindow**: Interactive task management tree.
- **LSP Bridge**: Code navigation and completion for `fish.toml`.

## Language Server Protocol (LSP)
Fish includes a built-in LSP server:
```bash
fish lsp
```
Connect any LSP-compliant editor (Neovim, Helix, Emacs, Sublime) to `fish lsp` for real-time diagnostics and autocomplete.
