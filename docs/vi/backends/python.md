# Python Backend

> 🌐 **Bản dịch & Đóng góp:** Bạn muốn dịch hoặc cải thiện tài liệu này bằng ngôn ngữ của mình? Xem [Hướng dẫn Dịch thuật](../TRANSLATION.md).

Python Backend cung cấp khả năng điều phối cho các dự án Python, hỗ trợ `pyproject.toml`, `setup.py`, `requirements.txt` và trình quản lý `uv` / `pytest`.

## Phát hiện Dự án
Được phát hiện khi có `pyproject.toml`, `setup.py` hoặc `requirements.txt`.

## Cấu hình (`fish.toml`)
```toml
[build]
backend = "py"

[pipelines.build]
inputs = ["src/**/*.py", "pyproject.toml"]
outputs = ["dist/*"]

[pipelines.test]
inputs = ["tests/**/*.py", "src/**/*.py"]
```

## Các Tác vụ
- **Build**: `python -m build` (hoặc `uv build`)
- **Test**: `pytest`
- **Type Check**: `mypy .`
