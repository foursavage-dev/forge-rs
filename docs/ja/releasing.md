# リリースガイド (Release Guide)

このドキュメントは Fish の標準リリースプロセスとチェックリストを説明します。

## リリースチェックリスト
1. **品質検証**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
2. **バージョン更新**:
   `Cargo.toml`、`Cargo.lock`、`vscode-extension/package.json` を更新。
3. **変更履歴の更新**:
   `CHANGELOG.md` に新機能と修正を記録。
4. **Git タグ作成**:
   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```
5. **配布物の公開**:
   - crates.io への crate 公開
   - VS Code `.vsix` のパッケージング
   - GitHub Releases へのバイナリ公開
