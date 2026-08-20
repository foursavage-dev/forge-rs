# Fish CLI コマンドリファレンス

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

Fish コマンドラインインターフェイスのすべてのコマンドとオプションに関する完全なリファレンスです。

---

## グローバルオプション (Global Options)

- `--experimental`: 実験的機能を有効化。
- `-v, --verbose`: 詳細な診断ログ出力を有効化。
- `-j, --jobs <N>`: 最大並列ワーカースレッド数。
- `--no-cache`: ローカルおよびリモートキャッシュをバイパス。
- `--cache-dir <PATH>`: カスタムローカルキャッシュディレクトリを指定。
- `--explain`: ターゲットが再ビルドされた詳細な理由を出力。
- `--pgo-generate`: Profile-Guided Optimization 用のインストルメント付きバイナリを生成。
- `--pgo-use`: 収集した PGO プロファイルデータを使用して最適化ビルドを実行。

---

## 主要コマンド

### `fish init`
Fish 設定を初期化し、ワークスペースをスキャンして多言語タスク定義 (`fish.yaml`) を生成します。

```bash
fish init [--force]
```

---

### `fish ui`
リアルタイムの Web ダッシュボードと SVG DAG 依存グラフビジュアライザを起動します。5言語（英語、ベトナム語、簡体字中国語、繁体字中国語、日本語）に対応しています。

```bash
fish ui [--port <PORT>] [--open]
```

---

### `fish build`
ワークスペース内のパッケージのビルドタスクを実行します。

```bash
fish build [OPTIONS]
```

**主なフラグ:**
- `-p, --package <NAME>`: 特定のパッケージをビルド。
- `--explain`: パッケージが再ビルドされた理由を診断。
- `--profile [FILE]`: Chrome Trace JSON 形式のプロファイルデータを生成。
- `--sandbox`: 隔離されたサンドボックス環境で実行。
- `--ram-limit <PCT>`: メモリ使用率がしきい値を超えた場合に並列数を動的に制御。

---

### `fish check`
アーティファクトの完全なリンクを行わずに、型チェックと静的解析を実行します。

```bash
fish check [OPTIONS]
```

---

### `fish test`
ワークスペース内のパッケージ全体でテストスイートを実行します。

```bash
fish test [OPTIONS]
```

---

### `fish run`
指定した実行可能バイナリをビルドして実行します。

```bash
fish run -p <PACKAGE> --bin <BINARY> [-- <ARGS>...]
```

---

### `fish query <EXPR>`
ワークスペースの依存関係グラフに対して代数クエリを実行します。

```bash
fish query "<EXPRESSION>"
```

**サポートされている関数:**
- `deps(//pkg)`: `//pkg` のすべての推移的依存関係。
- `rdeps(//pkg)`: `//pkg` に依存するすべての逆依存関係。
- `allpaths(//from, //to)`: `//from` と `//to` の間のすべての経路。
- `somepath(//from, //to)`: `//from` と `//to` の間の最短経路。
- `filter('pattern', expr)`: キーワードまたは正規表現でパッケージを絞り込み。

**使用例:**
```bash
# fish-cli のビルドに必要なすべての依存関係を検索
fish query "deps(//fish-cli)"

# fish-graph の変更によって影響を受けるすべてのクレートを検索
fish query "rdeps(//fish-graph)"

# app と util の間の最短依存チェーンを検索
fish query "somepath(//app, //util)"
```

---

### `fish daemon`
高速なウォームグラフ解決を実現するバックグラウンドビルドデーモンを管理します。

```bash
# デーモンを起動
fish daemon start [--port 9527]

# デーモンの状態を確認
fish daemon status [--port 9527]

# デーモンを停止
fish daemon stop [--port 9527]
```

---

### `fish graph`
プロジェクトの依存関係グラフを出力またはエクスポートします。

```bash
fish graph [--format <tree|dot|json>]
```

---

### `fish affected`
指定した Git リビジョン以降に変更されたパッケージのみを特定してタスクを実行します。

```bash
fish affected --since <GIT_REF> [--mode <build|check|test>]
```

---

### `fish cache`
ローカルのコンテンツアドレス可能ストレージ (CAS) とフィンガープリントを管理します。

```bash
# キャッシュサイズとオブジェクト数を表示
fish cache stats

# 古いフィンガープリントと孤立した成果物を削除
fish cache prune

# CAS ストレージの検査
fish cache cas stats
fish cache cas list
```

---

### `fish doctor`
システムツールチェーン、コンパイラ、リンカー、依存関係の準備状況を診断します。

```bash
fish doctor [--fix] [--ai]
```

---

### `fish ci init` / `fish ci export`
各種 CI/CD プラットフォーム向けのワークフロー設定を自動生成します。

```bash
fish ci init --platform <github|gitlab|circleci|bitbucket|all>
```
