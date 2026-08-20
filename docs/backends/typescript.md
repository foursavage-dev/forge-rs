# TypeScript/JavaScript Backend

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](TRANSLATION.md).

The TypeScript/JavaScript backend provides build orchestration for Node.js projects.

## Detection

The TypeScript/JavaScript backend is detected when a `package.json` file is present.

## Configuration

Configure the TypeScript/JavaScript backend via `fish.toml` in your project or workspace root:

```toml
[build]
backend = "ts"
jobs = 8

[pipelines.build]
inputs = ["src/**/*.{ts,tsx,js,jsx}", "package.json", "tsconfig.json"]
outputs = ["dist/**/*", "build/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*.{ts,js}", "src/**/*.{ts,js}"]
```

## Supported Package Managers

- **npm**: Default Node.js package manager
- **pnpm**: Fast, disk space efficient package manager
- **yarn**: Dependency management with workspaces
- **bun**: Fast JavaScript runtime and package manager

## Tasks Generated

### Build Task

```bash
npm run build
# or
pnpm run build
# or
yarn build
# or
bun run build
```

### Test Task

```bash
npm test
# or
pnpm test
# or
yarn test
# or
bun test
```

### Lint Task

```bash
npm run lint
```

## Dependency Extraction

The TypeScript/JavaScript backend extracts dependencies from:

- `package.json` dependencies and devDependencies
- `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`
- `node_modules` (for scanning)

## Fingerprinting

The TypeScript/JavaScript backend fingerprints:

- `package.json` content
- Lock file content
- Source files (excluding node_modules/)
- tsconfig.json configuration

## Monorepo Support

### Nx

```json
{
  "monorepo": true,
  "monorepoTool": "nx"
}
```

### Turborepo

```json
{
  "monorepo": true,
  "monorepoTool": "turborepo"
}
```

### Lerna

```json
{
  "monorepo": true,
  "monorepoTool": "lerna"
}
```

## Examples

### Basic TypeScript Project

```bash
cd my-typescript-project
fish build
```

### Project with pnpm

```bash
cd my-pnpm-project
fish build
```

### Nx Monorepo

```bash
cd my-nx-monorepo
fish build
```

## Performance Optimization

The TypeScript/JavaScript backend uses:

- **Package manager detection**: Uses fastest available manager
- **Lock file fingerprinting**: Efficient dependency tracking
- **Monorepo optimization**: Parallel package building

## Troubleshooting

### npm not found

Install Node.js: `curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -`

### pnpm not found

Install pnpm: `npm install -g pnpm`

### Build fails with TypeScript errors

Check tsconfig.json and ensure types are installed.

### Cache not working

Clear cache: `Fish cache prune` and rebuild.
