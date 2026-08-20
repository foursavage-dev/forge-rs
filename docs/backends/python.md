# Python Backend

> Ã°Å¸Å’Â **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../../TRANSLATION.md).

The Python backend provides build orchestration for Python projects.

## Detection

The Python backend is detected when a `pyproject.toml` or `setup.py` file is present.

## Configuration

Configure the Python backend via `fish.toml` in your project or workspace root:

```toml
[build]
backend = "py"
jobs = 4

[pipelines.build]
inputs = ["**/*.py", "pyproject.toml", "requirements.txt"]
outputs = ["dist/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*.py", "**/*.py"]
```

## Supported Package Managers

- **uv**: Fast Python package installer
- **poetry**: Modern Python packaging and dependency management
- **pip**: Standard Python package installer

## Tasks Generated

### Build Task

```bash
uv build
# or
poetry build
# or
pip install -e .
```

### Test Task

```bash
uv run pytest
# or
poetry run pytest
# or
pytest
```

### Lint Task

```bash
uv run ruff check
# or
poetry run ruff check
```

## Dependency Extraction

The Python backend extracts dependencies from:

- `pyproject.toml` dependencies
- `requirements.txt` files
- `setup.py` setup_requires
- Poetry lock file

## Fingerprinting

The Python backend fingerprints:

- `pyproject.toml` content
- `requirements.txt` content
- Source files (excluding venv/)
- Python version

## Examples

### Basic Python Project

```bash
cd my-python-project
Fish build
```

### Poetry Project

```bash
cd my-poetry-project
Fish build
```

### Project with Tests

```bash
cd my-python-project
Fish test
```

## Performance Optimization

The Python backend uses:

- **uv for speed**: Uses uv for fast dependency management
- **Virtual environment isolation**: Clean build environments
- **Dependency caching**: Efficient dependency resolution

## Troubleshooting

### Python not found

Install Python from python.org or use pyenv.

### uv not found

Install uv: `pip install uv`

### Poetry not found

Install Poetry: `curl -sSL https://install.python-poetry.org | python3 -`

### Cache not working

Clear cache: `Fish cache prune` and rebuild.
