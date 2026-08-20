# Python バックエンド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](../TRANSLATION.md) をご覧ください。

Python バックエンドは、`pyproject.toml`, `setup.py`, `requirements.txt` および `uv` / `pytest` をサポートする Python プロジェクト向けのオーケストレーションを提供します。

## 自動検出
`pyproject.toml`, `setup.py`, `requirements.txt` のいずれかが存在する場合に自動検出されます。

## 設定 (`fish.toml`)
```toml
[build]
backend = "py"

[pipelines.build]
inputs = ["src/**/*.py", "pyproject.toml"]
outputs = ["dist/*"]

[pipelines.test]
inputs = ["tests/**/*.py", "src/**/*.py"]
```
