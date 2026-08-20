# Hướng Dẫn Chuyển Đổi Sang Fish

Tài liệu này hướng dẫn cách chuyển đổi các cấu hình build hiện có từ Turborepo, Nx và Bazel sang tệp cấu hình chuẩn `fish.toml`.

---

## 1. Chuyển đổi từ Turborepo (`turbo.json`)

Cấu hình pipeline trong Turborepo được ánh xạ trực tiếp sang các section `[pipelines]` của Fish:

### Trước: `turbo.json`
```json
{
  "tasks": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**"]
    }
  }
}
```

### Sau: `fish.toml`
```toml
[build]
backend = "ts"
jobs = 8
reflink = true
semantic = true

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "package.json"]
outputs = ["dist/**"]
```

---

## 2. Chuyển đổi từ Nx (`nx.json`)
Chạy lệnh khởi tạo tự động của Fish để quét toàn bộ workspace:
```bash
fish init --force
fish doctor --fix
```
