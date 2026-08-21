# TypeScript / JavaScript Backend

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](../TRANSLATION.md).

TypeScript/JavaScript Backend cung cấp khả năng điều phối biên dịch cho các dự án Node.js, Web và full-stack.

## Phát hiện Dự án (Detection)

Được tự động kích hoạt khi có tệp `package.json` trong thư mục dự án.

## Cấu hình (Configuration)

Cấu hình qua `fish.toml` tại thư mục gốc:

```toml
[build]
backend = "ts"
jobs = 8

[pipelines.build]
inputs = ["src/**/*.{ts,tsx,js,jsx}", "package.json", "tsconfig.json"]
outputs = ["dist/**/*", "build/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*.{ts,js}", "src/**/*.{ts,js}"]
```

## Trình Quản lý Gói Hỗ trợ (Package Managers)
- **npm**: Trình quản lý mặc định của Node.js
- **pnpm**: Trình quản lý gói tốc độ cao và tiết kiệm dung lượng ổ đĩa
- **yarn**: Hỗ trợ tốt cho monorepo workspaces
- **bun**: Runtime và trình quản lý gói JavaScript siêu tốc

## Các Tác vụ Được Tạo (Tasks Generated)

### Tác vụ Biên dịch (Build Task)
```bash
npm run build # hoặc pnpm / yarn / bun run build
```

### Tác vụ Kiểm thử (Test Task)
```bash
npm test # hoặc pnpm / yarn / bun test
```

### Tác vụ Lint (Lint Task)
```bash
npm run lint
```

## Trích xuất Phụ thuộc & Fingerprinting
- Phân tích `dependencies` và `devDependencies` trong `package.json`.
- Tính mã băm lockfile (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lockb`).
- Tự động loại trừ thư mục `node_modules/` và `dist/` khi băm dấu vân tay nguồn.
