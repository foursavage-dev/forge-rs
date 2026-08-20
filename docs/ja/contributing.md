# Fish への貢献

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

Fish プロジェクトへの貢献に関心をお寄せいただきありがとうございます！

## 貢献ワークフロー

1. GitHub でリポジトリをフォーク: `https://github.com/requla11/fish`
2. ローカルにクローン:
   ```bash
   git clone https://github.com/<YOUR_USERNAME>/fish.git
   cd fish
   git checkout -b feat/my-feature dev
   ```
3. ガイドラインに沿って変更を実装:
   - コードおよびコミットメッセージは英語で記述。
   - テスト実行: `cargo test --workspace`。
   - フォーマット確認: `cargo fmt --all -- --check`。
   - Clippy 確認: `cargo clippy --workspace --all-targets --all-features -- -D warnings`。
4. 変更をコミットし、**`dev`** ブランチに向けて Pull Request を作成します。
