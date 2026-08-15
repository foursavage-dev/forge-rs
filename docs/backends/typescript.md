# TypeScript/JavaScript Backend

The TypeScript/JavaScript backend provides build orchestration for Node.js projects.

## Detection

The TypeScript/JavaScript backend is detected when a `package.json` file is present.

## Configuration

### forge.ts.json

```json
{
  "packageManager": "npm",
  "scripts": ["build", "test", "lint"],
  "includeDevDependencies": false,
  "monorepo": false
}
```

### Configuration Options

- `packageManager`: Package manager to use (npm, pnpm, yarn, bun)
- `scripts`: NPM scripts to run
- `includeDevDependencies`: Whether to include dev dependencies in fingerprint
- `monorepo`: Whether this is a monorepo (Nx, Turborepo, Lerna)

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
forge build
```

### Project with pnpm

```bash
cd my-pnpm-project
forge build
```

### Nx Monorepo

```bash
cd my-nx-monorepo
forge build
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

Clear cache: `forge cache prune` and rebuild.
