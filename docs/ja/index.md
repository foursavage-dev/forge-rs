---
layout: home

hero:
  name: "Fish"
  text: "超高速ポリグロット・ビルドオーケストレーションシステム"
  tagline: "代数DAGスケジューリング、決定論的CASキャッシュ、分散レーシング実行により、11以上のツールチェーンをシームレスに統合。"
  image:
    src: /logo.svg
    alt: Fish Logo
  actions:
    - theme: brand
      text: スタートガイド
      link: /ja/getting-started
    - theme: alt
      text: システムアーキテクチャ
      link: /ja/architecture
    - theme: alt
      text: GitHubで見る
      link: https://github.com/requla11/fish

features:
  - icon: ⚡
    title: 究極の並行処理とレーシング
    details: GNU Jobserver 統合、動的リモートレーシング、ビンパッキングワーカーキューにより、CPU とネットワークの使用効率を最大化。
  - icon: 🎯
    title: 11以上の言語ツールチェーン
    details: Rust、Go、TypeScript、Python、C/C++、Docker、Java、.NET、Swift、Dart、Zig をゼロ構成で自動検出。
  - icon: 🔒
    title: 決定論的キャッシュとCASストレージ
    details: Blake3 多層フィンガープリントと ZSTD 圧縮により、ミリ秒未満のキャッシュヒットとビルド再利用を実現。
  - icon: 🌐
    title: リアルタイム Web ダッシュボード
    details: リアルタイムの DAG 依存関係グラフの可視化と多言語テレメトリにより、すべてのワークロードを監視。
  - icon: 🛡️
    title: 暗号化 SBOM とサンドボックス
    details: 依存関係の脆弱性スキャン、ハーメチックサンドボックス、Ed25519 アーティファクト署名を標準搭載。
  - icon: 🚀
    title: 代数 DAG クエリエンジン
    details: 表現力豊かな代数式（deps、rdeps、somepath）を使用して依存関係を瞬時に検索。
---
