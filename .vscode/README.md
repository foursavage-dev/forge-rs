# Forge VS Code Extension

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../TRANSLATION.md).

Basic VS Code extension for Forge build system integration.

## Features

- Build project: `Forge: Build Project`
- Run tests: `Forge: Run Tests`
- Show build graph: `Forge: Show Build Graph`
- Clean artifacts: `Forge: Clean Build Artifacts`
- Run diagnostics: `Forge: Run Diagnostics`
- Show affected packages: `Forge: Show Affected Packages`

## Installation

1. Clone this repository
2. Open in VS Code
3. Press `F5` to launch extension development host
4. Or package with `vsce publish`

## Configuration

```json
{
  "forge.path": "forge",
  "forge.experimental": false,
  "forge.maxJobs": 4
}
```

## Development

```bash
npm install
npm run compile
npm run watch
```

## Future Enhancements

- Real-time build output parsing
- IntelliSense for forge.toml
- Build graph visualization
- Circular dependency detection
- Task completion for build targets
- Integration with build status bar