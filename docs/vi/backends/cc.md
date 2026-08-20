# C / C++ Backend

> 🌐 **Translations & Contributions:** [Translation Guidelines](TRANSLATION.md)

Fish cung cấp khả năng điều phối build tự động cho các dự án đa ngôn ngữ.

## Phát hiện dự án

Phát hiện dự án: `CMakeLists.txt`.

## Cấu hình trong fish.toml

```toml
[build]
backend = "c / c++"
jobs = 8

[pipelines.build]
inputs = ["src/**/*", "CMakeLists.txt"]
outputs = ["target/**/*"]
```

## Các tác vụ tự động sinh ra

- `fish build`: Các tác vụ tự động sinh ra (build)
- `fish test`: Các tác vụ tự động sinh ra (test)
- `fish check`: Các tác vụ tự động sinh ra (check)

## Trích xuất quan hệ phụ thuộc

- `CMakeLists.txt`
