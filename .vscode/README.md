# fish VS Code Extension

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

Basic VS Code extension for fish build system integration.

## Features

- Build project: `fish: Build Project`
- Run tests: `fish: Run Tests`
- Show build graph: `fish: Show Build Graph`
- Clean artifacts: `fish: Clean Build Artifacts`
- Run diagnostics: `fish: Run Diagnostics`
- Show affected packages: `fish: Show Affected Packages`

## Installation

1. Clone this repository
2. Open in VS Code
3. Press `F5` to launch extension development host
4. Or package with `vsce publish`

## Configuration

```json
{
  "fish.path": "fish",
  "fish.experimental": false,
  "fish.maxJobs": 4
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
- IntelliSense for fish.toml
- Build graph visualization
- Circular dependency detection
- Task completion for build targets
- Integration with build status bar