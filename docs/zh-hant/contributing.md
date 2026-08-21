# 参与 Fish 貢獻

> 🌐 **翻译与貢獻：** 想用您的语言翻译或改进本文档？请参阅我们的 [翻译指南](TRANSLATION.md)。

感谢您对 Fish 项目的关注与支持！

## 貢獻流程

1. 在 GitHub 上 Fork 仓库：`https://github.com/requla11/fish`
2. 克隆您的 Fork 分支到本地：
   ```bash
   git clone https://github.com/<YOUR_USERNAME>/fish.git
   cd fish
   git checkout -b feat/my-feature dev
   ```
3. 遵循项目规范进行修改：
   - 标识符与提交记录必须使用英文。
   - 运行测试：`cargo test --workspace`。
   - 格式校验：`cargo fmt --all -- --check`。
   - 代码检查：`cargo clippy --workspace --all-targets --all-features -- -D warnings`。
4. 提交更改并发起针对 **`dev`** 分支的 Pull Request。
