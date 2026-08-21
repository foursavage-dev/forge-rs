# Python AI & Predictive Engine (`py/`)

Fish integrates a predictive machine learning engine written in Python 3.11+.

## AI Capabilities
- **Build Time Predictor (`py/src/fish_ai/predictor.py`)**: Estimates package build duration using historical logs and AST token features.
- **Flaky Test Quarantine (`py/src/fish_ai/flaky.py`)**: Automatically detects non-deterministic tests and isolates them into quarantine queues.
- **Speculative Pre-Warmer (`py/src/fish_ai/speculative.py`)**: Predicts which downstream packages will be affected based on git commit diffs.
- **AST Remediation (`py/src/fish_ai/remediation.py`)**: Suggests automated fixes for broken syntax and mismatched imports.

## Running Python AI Tests
```bash
cd py
python -m unittest discover tests
```
