# Python AI 預測引擎 (`py/`)

Fish 整合了基於 Python 3.11+ 編寫的智慧化預測與機器學習引擎。

## AI 核心功能
- **建置耗時預測器 (`py/src/fish_ai/predictor.py`)**: 根據歷史建置日誌與 AST 特徵預測任務時長。
- **不穩定性測試隔離 (`py/src/fish_ai/flaky.py`)**: 自動檢測並隔離非確定性 Flaky 測試。
- **推測式預熱編譯器 (`py/src/fish_ai/speculative.py`)**: 根據 Git 提交差異預測下游受影響目標並預熱編譯。
- **AST 智慧修復 (`py/src/fish_ai/remediation.py`)**: 自動診斷語法破損並生成修復補丁。

## 執行 Python 測試
```bash
cd py
python -m unittest discover tests
```
