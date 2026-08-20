# TypeScript / JavaScript バックエンド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](../TRANSLATION.md) をご覧ください。

TypeScript/JavaScript バックエンドは、Node.js、Web、およびフルスタックプロジェクト向けのビルドオーケストレーションを提供します。

## 自動検出 (Detection)

プロジェクトに `package.json` が存在する場合に自動検出されます。

## 設定 (Configuration)

`fish.toml` で設定します：

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

## サポートされているパッケージマネージャー
- **npm**: デフォルトの Node.js パッケージマネージャー
- **pnpm**: 高速でディスク効率の良いパッケージマネージャー
- **yarn**: ワークスペース対応の依存関係管理
- **bun**: 高速な JavaScript ランタイムおよびパッケージマネージャー

## 生成されるタスク (Tasks Generated)

### ビルドタスク (Build Task)
```bash
npm run build # または pnpm / yarn / bun run build
```

### テストタスク (Test Task)
```bash
npm test # または pnpm / yarn / bun test
```

### リントタスク (Lint Task)
```bash
npm run lint
```
