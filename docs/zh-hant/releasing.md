# 版本發布指南 (Release Guide)

本文檔說明了 Fish 的標準發布流程與品質檢查清單。

## 發布檢查清單
1. **品質驗證**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
2. **版本號升級**:
   更新 `Cargo.toml`、`Cargo.lock` 及 `vscode-extension/package.json`。
3. **更新更新日誌**:
   在 `CHANGELOG.md` 中記錄新增功能與修復。
4. **打 Git 標籤**:
   ```bash
   git tag -a v0.4.0 -m "Release v0.4.0"
   git push origin v0.4.0
   ```
5. **產物分發**:
   - 發布 crates 到 crates.io
   - 打包 VS Code `.vsix`
   - 在 GitHub Releases 發布二進位檔案
