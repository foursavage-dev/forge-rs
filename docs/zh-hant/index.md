---
layout: home

hero:
  name: "Fish"
  text: "高性能多語言構建編排與快取加速系統"
  tagline: "基於代數 DAG 調度、確定性 CAS 快取與分散式 Racing 執行，統一 11+ 工具鏈的構建體驗。"
  image:
    src: /logo.svg
    alt: Fish Logo
  actions:
    - theme: brand
      text: 快速開始
      link: /zh-hant/getting-started
    - theme: alt
      text: 系統架構
      link: /zh-hant/architecture
    - theme: alt
      text: GitHub 源碼
      link: https://github.com/requla11/fish

features:
  - icon: ⚡
    title: 極致並發與 Racing 調度
    details: 整合 GNU Jobserver、動態遠端 Racing 與裝箱佇列調度，最大化 CPU 與網路利用率。
  - icon: 🎯
    title: 11+ 主流語言工具鏈
    details: 原生零配置自動偵測 Rust、Go、TypeScript、Python、C/C++、Docker、Java、.NET、Swift、Dart 與 Zig。
  - icon: 🔒
    title: 確定性快取與 CAS 儲存
    details: 基於 Blake3 多層指紋與 ZSTD 壓縮，實現毫秒級快取命中與構建復用。
  - icon: 🌐
    title: 即時互動式 Web 控制台
    details: 即時 DAG 依賴圖視覺化、多語言遙測指標監控 monorepo 中所有工作負載。
  - icon: 🛡️
    title: 密碼學 SBOM 與沙盒隔離
    details: 內建依賴漏洞掃描、Hermetic 密封沙盒與 Ed25519 產物簽章保障供應鏈安全。
  - icon: 🚀
    title: 代數 DAG 查詢引擎
    details: 使用表現力極強的代數表達式查詢依賴關係（deps、rdeps、critical path）。
---
