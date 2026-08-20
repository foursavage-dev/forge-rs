# TypeScript Backend

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish cung cấp khả năng điều phối build tự động cho các dự án đa ngôn ngữ.

## Phát hiện dự án

Phát hiện dự án: `package.json`.

## Cấu hình trong fish.toml

```toml
[build]
backend = "typescript"
jobs = 8

[pipelines.build]
inputs = ["src/**/*", "package.json"]
outputs = ["target/**/*"]
```

## Các tác vụ tự động sinh ra

- `fish build`: Các tác vụ tự động sinh ra (build)
- `fish test`: Các tác vụ tự động sinh ra (test)
- `fish check`: Các tác vụ tự động sinh ra (check)

## Trích xuất quan hệ phụ thuộc

- `package.json`
