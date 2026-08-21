---
layout: home

hero:
  name: "Fish"
  text: "高性能多语言构建编排与缓存加速系统"
  tagline: "基于代数 DAG 调度、确定性 CAS 缓存与分布式 Racing 执行，统一 11+ 工具链的构建体验。"
  image:
    src: /logo.svg
    alt: Fish Logo
  actions:
    - theme: brand
      text: 快速开始
      link: /zh-hans/getting-started
    - theme: alt
      text: 系统架构
      link: /zh-hans/architecture
    - theme: alt
      text: GitHub 源码
      link: https://github.com/requla11/fish

features:
  - icon: ⚡
    title: 极致并发与 Racing 调度
    details: 集成 GNU Jobserver、动态远程 Racing 与装箱队列调度，最大化 CPU 与网络利用率。
  - icon: 🎯
    title: 11+ 主流语言工具链
    details: 原生零配置自动检测 Rust、Go、TypeScript、Python、C/C++、Docker、Java、.NET、Swift、Dart 与 Zig。
  - icon: 🔒
    title: 确定性缓存与 CAS 存储
    details: 基于 Blake3 多层指纹与 ZSTD 压缩，实现毫秒级缓存命中与构建复用。
  - icon: 🌐
    title: 实时交互式 Web 控制台
    details: 实时 DAG 依赖图可视化、多语言遥测指标监控 monorepo 中所有工作负载。
  - icon: 🛡️
    title: 密码学 SBOM 与沙盒隔离
    details: 内置依赖漏洞扫描、Hermetic 密封沙盒与 Ed25519 产物签名保障供应链安全。
  - icon: 🚀
    title: 代数 DAG 查询引擎
    details: 使用表现力极强的代数表达式查询依赖关系（deps、rdeps、critical path）。
---
