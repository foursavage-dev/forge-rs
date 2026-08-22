# Python AI 预测引擎 (`py/`)

Fish 集成了基于 Python 3.11+ 编写的智能化预测与机器学习引擎。

## AI 核心功能
- **构建耗时预测器 (`py/src/fish_ai/predictor.py`)**: 根据历史构建日志与 AST 特征预测任务时长。
- **不稳定性测试隔离 (`py/src/fish_ai/flaky.py`)**: 自动检测并隔离非确定性 Flaky 测试。
- **推测式预热编译器 (`py/src/fish_ai/speculative.py`)**: 根据 Git 提交差异预测下游受影响目标并预热编译。
- **AST 智能修复 (`py/src/fish_ai/remediation.py`)**: 自动诊断语法破损并生成修复补丁。

## 运行 Python 测试
```bash
cd py
python -m unittest discover tests
```
