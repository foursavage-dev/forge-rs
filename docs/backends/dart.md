# Dart & Flutter Backend

Fish orchestrates Dart CLI packages and multiplatform Flutter applications with high-speed caching.

## Detection
Fish automatically detects Dart and Flutter projects through:
- `pubspec.yaml`
- `pubspec.lock`

## Supported Commands
```bash
fish build     # Compiles Dart AOT or Flutter bundles
fish test      # Executes dart test / flutter test
fish check     # Runs dart analyze
```

## Configuration in `fish.toml`
```toml
backend = "dart"
jobs = 4
```
