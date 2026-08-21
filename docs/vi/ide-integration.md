# Tích Hợp IDE & Công Cụ Lập Trình

Fish cung cấp bộ công cụ lập trình viên chính thức cho VS Code, JetBrains IDEs và giao thức Language Server Protocol (LSP).

## Extension VS Code
Cài đặt extension chính thức trong thư mục `vscode-extension/`:
- **Trực quan hóa đồ thị DAG**: Xem sơ đồ phụ thuộc tác vụ trực quan dạng webview.
- **Chạy tác vụ 1-Click**: Build, test, check trực tiếp từ thanh sidebar.
- **Chẩn đoán thời gian thực**: Báo lỗi inline và gợi ý cú pháp qua LSP.

## Plugin Bộ JetBrains
Nằm trong `jetbrains-plugin/` cho IntelliJ IDEA, CLion, RustRover, PyCharm và Rider:
- **Fish ToolWindow**: Quản lý tác vụ theo cây danh mục trực quan.
- **Cầu nối LSP**: Điều hướng và tự động hoàn thiện cú pháp `fish.toml`.

## Language Server Protocol (LSP)
Fish tích hợp sẵn máy chủ LSP:
```bash
fish lsp
```
Dễ dàng kết nối bất kỳ trình soạn thảo nào (Neovim, Helix, Emacs, Sublime) với `fish lsp`.
