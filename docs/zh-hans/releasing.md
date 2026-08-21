# 版本发布指南 (Release Guide)

本文档说明了 Fish 的标准发布流程与质量检查清单。

## 发布检查清单
1. **质量验证**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
2. **版本号升级**:
   更新 `Cargo.toml`、`Cargo.lock` 及 `vscode-extension/package.json`。
3. **更新更新日志**:
   在 `CHANGELOG.md` 中记录新增功能与修复。
4. **打 Git 标签**:
   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```
5. **产物分发**:
   - 发布 crates 到 crates.io
   - 打包 VS Code `.vsix`
   - 在 GitHub Releases 发布二进制文件
