# Python AI 予測エンジン (`py/`)

Fish には Python 3.11+ で構築されたインテリジェントな予測エンジンが組み込まれています。

## AI 機能
- **ビルド時間予測 (`py/src/fish_ai/predictor.py`)**: 過去のログと AST 特徴からビルド時間を予測。
- **Flaky テストの隔離 (`py/src/fish_ai/flaky.py`)**: 非決定論的なテストを自動検知して隔離。
- **投機的プリウォーマー (`py/src/fish_ai/speculative.py`)**: Git 差分から影響を受けるターゲットを予測し事前コンパイル。
- **AST 自動修復 (`py/src/fish_ai/remediation.py`)**: 構文エラーやインポート漏れの自動修復パッチを生成。

## Python テストの実行
```bash
cd py
python -m unittest discover tests
```
