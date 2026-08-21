# Fish VS Code Extension

VS Code extension for Fish build orchestration system with IDE integration, DAG visualization, and task runner capabilities.

## Features

### Core Functionality
- **Dependency Graph Visualization**: Interactive tree view showing workspace structure and package dependencies
- **Task Runner**: Quick access to common Fish build commands (build, test, clean)
- **Workspace Diagnostics**: Real-time diagnostics and error reporting
- **LSP Bridge**: Language Server Protocol integration for enhanced editor support

### Commands
- `Fish: Run Build` - Execute Fish build command
- `Fish: Run Tests` - Execute Fish test command  
- `Fish: Clean Build` - Clean build artifacts
- `Fish: Refresh DAG Graph` - Refresh dependency graph visualization
- `Fish: Show Build Diagnostics` - Display build diagnostics
- `Fish: Toggle Watch Mode` - Enable/disable file watching

### Configuration
The extension supports the following settings in VS Code settings:

```json
{
  "fish.executablePath": "fish",
  "fish.autoRefresh": true,
  "fish.showBuildNotifications": true
}
```

## Installation

### Development Installation
1. Clone this repository
2. Navigate to the `vscode-extension` directory
3. Install dependencies: `npm install`
4. Compile TypeScript: `npm run compile`
5. Press F5 in VS Code to launch extension in development mode

### Production Installation
1. Package the extension: `vsce package`
2. Install the `.vsix` file in VS Code

## Development

### Project Structure
```
vscode-extension/
├── src/
│   ├── extension.ts      # Main extension entry point
│   ├── dagProvider.ts    # Dependency graph tree provider
│   ├── taskProvider.ts   # Build tasks tree provider
│   ├── diagnosticsProvider.ts # Diagnostics collection
│   └── lspClient.ts      # LSP bridge implementation
├── package.json          # Extension manifest
├── tsconfig.json         # TypeScript configuration
└── README.md            # This file
```

### Building
```bash
npm install
npm run compile
```

### Testing
```bash
npm test
```

## Future Enhancements

This is a basic implementation. Future versions will include:

- **Full LSP Integration**: Complete Language Server Protocol support for code completion, navigation, and refactoring
- **Interactive DAG Visualization**: Web-based DAG preview with zoom, filtering, and interactive exploration
- **Build Status Integration**: Real-time build progress and status updates
- **JetBrains Plugin**: Native IntelliJ/CLion/Rider integration
- **Advanced Diagnostics**: Deep integration with Fish's query engine and build analysis

## Requirements

- VS Code 1.75.0 or higher
- Fish build system installed and available in PATH
- Node.js 18+ for development

## License

MIT License - See main project license file for details.