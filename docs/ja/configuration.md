# Fish 設定ガイド

> 🌐 **翻訳と貢献:** このドキュメントをあなたの言語で翻訳または改善したいですか？ [翻訳ガイドライン](TRANSLATION.md) をご覧ください。

このガイドでは、`fish.toml` を使用した Fish ワークスペースの設定方法を説明します。

---

## 設定ファイルの概要

Fish はワークスペースのルートにある `fish.toml` から設定を読み込みます。`fish.toml` が存在しない場合、Fish は適切なデフォルト設定を自動的に適用します。

```toml
[build]
backend = "rust"
jobs = 8
no_cache = false
sandbox = false
semantic = true
critical_path = true
ram_limit = 85

[cache]
dir = "~/.fish/cache"
reflink = true

[remote]
cache_url = "http://127.0.0.1:8080"
token = "secret-cache-token"

[daemon]
port = 9527

[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml", "Cargo.lock"]
outputs = ["target/release/**/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]
```

---

## トップレベルセクション

### `[build]` —— 実行設定

| キー | 型 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `backend` | 文字列 | Auto | プライマリツールチェーン (`rust`, `ts`, `go`, `cc`, `python`, `java`, `dotnet`, `docker`)。 |
| `jobs` | 整数 | `num_cpus` | 最大並列ワーカタスク数。 |
| `no_cache` | ブール値 | `false` | ローカルおよびリモートキャッシュ検索を無効化。 |
| `sandbox` | ブール値 | `false` | サンドボックス隔離環境でタスクを実行。 |
| `semantic` | ブール値 | `false` | AST 意味的変更検知を有効化。 |
| `critical_path` | ブール値 | `false` | 依存グラフのクリティカルパス上のタスクを優先。 |
| `ram_limit` | 整数 (1-100) | `85` | 利用可能メモリがこの割合を下回った場合に並列数を制御。 |
| `timeout` | 整数 | None | タスク実行タイムアウト（秒）。 |

---

### `[cache]` —— ローカルストレージ設定

| キー | 型 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `dir` | 文字列 | `~/.fish/cache` | ローカルコンテンツアドレス可能ストレージ (CAS) のパス。 |
| `reflink` | ブール値 | `true` | Copy-on-Write (CoW) またはハードリンクを使用して成果物を高速実体化。 |

---

### `[remote]` —— 分散キャッシュとリモート実行

| キー | 型 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `cache_url` | 文字列 | None | リモートキャッシュサーバーのアドレス (HTTP/gRPC)。 |
| `token` | 文字列 | None | リモート操作用の Bearer 認証トークン。 |
| `workers` | 文字列の配列 | `[]` | リモートワーカーノードのアドレスリスト（例: `["worker1:9000", "worker2:9000"]`）。 |
| `send_source` | ブール値 | `false` | 共有ストレージのないワーカーにソースコードのスナップショットを圧縮送信。 |

---

### `[daemon]` —— バックグラウンド IPC デーモン

| キー | 型 | デフォルト | 説明 |
| :--- | :--- | :--- | :--- |
| `port` | 整数 | `9527` | Fish バックグラウンドデーモンがリッスンするローカル TCP ポート。 |

---

### `[pipelines.<task>]` —— タスクパイプライン設定

パッケージ間のタスク依存関係とキャッシュ境界を設定します：

```toml
[pipelines.build]
depends_on = ["^build"]
inputs = ["src/**/*", "Cargo.toml"]
outputs = ["target/release/*"]

[pipelines.test]
depends_on = ["build"]
inputs = ["tests/**/*", "src/**/*"]

[pipelines.lint]
inputs = ["src/**/*.rs"]
```
