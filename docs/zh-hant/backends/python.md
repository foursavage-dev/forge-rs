# Python 後端

> 🌐 **翻译与贡献：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](../TRANSLATION.md)。

Python 後端为 Python 项目提供构建编排，支持 `pyproject.toml`, `setup.py`, `requirements.txt` 以及 `uv` / `pytest`。

## 自动检测
当存在 `pyproject.toml`, `setup.py` 或 `requirements.txt` 时自动启用。

## 配置 (`fish.toml`)
```toml
[build]
backend = "py"

[pipelines.build]
inputs = ["src/**/*.py", "pyproject.toml"]
outputs = ["dist/*"]

[pipelines.test]
inputs = ["tests/**/*.py", "src/**/*.py"]
```

## 生成任务
- **构建**: `python -m build` (或 `uv build`)
- **测试**: `pytest`
- **类型检查**: `mypy .`
