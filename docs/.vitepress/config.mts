import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "Fish",
  description: "Fast, Polyglot Monorepo Build Orchestration System in Rust 2024",
  base: "/fish/",
  cleanUrls: true,
  ignoreDeadLinks: false,

  head: [
    ['link', { rel: 'icon', href: '/fish/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#3eaf7c' }]
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/getting-started' },
          { text: 'Architecture', link: '/architecture' },
          { text: 'Crates', link: '/crates-architecture' },
          { text: 'Comparison', link: '/comparison' },
          { text: 'Benchmarks', link: '/benchmarks' },
          { text: 'Backends', link: '/backends/' },
          { text: 'IDE & Tools', link: '/ide-integration' },
          { text: 'AI Agents', link: '/ai-agents' },
          { text: 'CLI', link: '/cli-reference' },
          { text: 'Roadmap', link: '/ROADMAP' },
          { text: 'API', link: '/api/overview' }
        ],
        sidebar: [
          {
            text: 'Getting Started',
            items: [
              { text: 'Overview & Quick Start', link: '/getting-started' },
              { text: 'System Architecture', link: '/architecture' },
              { text: '36 Crates Workspace Architecture', link: '/crates-architecture' },
              { text: 'Distributed Go Services', link: '/go-services' },
              { text: 'Python AI Engine', link: '/python-ai-engine' },
              { text: 'Comparison Matrix', link: '/comparison' },
              { text: 'Performance Benchmarks', link: '/benchmarks' },
              { text: 'Migration Guides', link: '/migration' },
              { text: 'Configuration Reference', link: '/configuration' },
              { text: 'IDE Integration & LSP', link: '/ide-integration' },
              { text: 'Remote Execution & CAS', link: '/remote-execution' },
              { text: 'Starter Examples', link: '/examples-guide' },
              { text: 'CLI Reference', link: '/cli-reference' }
            ]
          },
          {
            text: 'Language Backends',
            items: [
              { text: 'All Backends', link: '/backends/' },
              { text: 'Rust', link: '/backends/rust' },
              { text: 'Go', link: '/backends/go' },
              { text: 'TypeScript / Node', link: '/backends/typescript' },
              { text: 'Python', link: '/backends/python' },
              { text: 'C / C++', link: '/backends/cc' },
              { text: 'Docker / OCI', link: '/backends/docker' },
              { text: 'Java / Kotlin', link: '/backends/java' },
              { text: 'Dotnet (.NET)', link: '/backends/dotnet' },
              { text: 'Swift / ObjC', link: '/backends/swift' },
              { text: 'Dart / Flutter', link: '/backends/dart' },
              { text: 'Zig', link: '/backends/zig' }
            ]
          },
          {
            text: 'Development & Ecosystem',
            items: [
              { text: 'API Overview', link: '/api/overview' },
              { text: 'AI Coding Agents & Rules', link: '/ai-agents' },
              { text: 'AI Agent Workflow', link: '/AI_AGENT_WORKFLOW' },
              { text: 'Development Guide', link: '/development' },
              { text: 'Contributing Guidelines', link: '/contributing' },
              { text: 'Release Guide', link: '/releasing' },
              { text: 'Changelog & History', link: '/changelog' },
              { text: 'Support & Community', link: '/support' },
              { text: 'Security Policy', link: '/security' },
              { text: 'Translation Guide', link: '/TRANSLATION' },
              { text: 'Roadmap', link: '/ROADMAP' },
              { text: 'FAQ & Troubleshooting', link: '/faq-troubleshooting' }
            ]
          }
        ]
      }
    },
    vi: {
      label: 'Tiếng Việt',
      lang: 'vi',
      link: '/vi/',
      themeConfig: {
        nav: [
          { text: 'Hướng dẫn', link: '/vi/getting-started' },
          { text: 'Kiến trúc', link: '/vi/architecture' },
          { text: 'Crates', link: '/vi/crates-architecture' },
          { text: 'So sánh', link: '/vi/comparison' },
          { text: 'Benchmark', link: '/vi/benchmarks' },
          { text: 'Backend', link: '/vi/backends/' },
          { text: 'IDE & Tools', link: '/vi/ide-integration' },
          { text: 'AI Agent', link: '/vi/ai-agents' },
          { text: 'CLI', link: '/vi/cli-reference' },
          { text: 'Lộ trình', link: '/vi/ROADMAP' }
        ],
        sidebar: [
          {
            text: 'Bắt đầu',
            items: [
              { text: 'Tổng quan & Bắt đầu nhanh', link: '/vi/getting-started' },
              { text: 'Kiến trúc hệ thống', link: '/vi/architecture' },
              { text: 'Kiến trúc 36 Crates Rust', link: '/vi/crates-architecture' },
              { text: 'Dịch vụ phân tán Go', link: '/vi/go-services' },
              { text: 'Bộ máy AI Python', link: '/vi/python-ai-engine' },
              { text: 'Bảng so sánh trực diện', link: '/vi/comparison' },
              { text: 'Đo lường hiệu năng Benchmark', link: '/vi/benchmarks' },
              { text: 'Hướng dẫn chuyển đổi', link: '/vi/migration' },
              { text: 'Cấu hình fish.toml', link: '/vi/configuration' },
              { text: 'Tích hợp IDE & LSP', link: '/vi/ide-integration' },
              { text: 'Thực thi từ xa & CAS', link: '/vi/remote-execution' },
              { text: 'Dự án mẫu Polyglot', link: '/vi/examples-guide' },
              { text: 'Tra cứu lệnh CLI', link: '/vi/cli-reference' }
            ]
          },
          {
            text: 'Backend Ngôn ngữ',
            items: [
              { text: 'Tất cả Backend', link: '/vi/backends/' },
              { text: 'Rust', link: '/vi/backends/rust' },
              { text: 'Go', link: '/vi/backends/go' },
              { text: 'TypeScript / Node', link: '/vi/backends/typescript' },
              { text: 'Python', link: '/vi/backends/python' },
              { text: 'C / C++', link: '/vi/backends/cc' },
              { text: 'Docker / OCI', link: '/vi/backends/docker' },
              { text: 'Java / Kotlin', link: '/vi/backends/java' },
              { text: 'Dotnet (.NET)', link: '/vi/backends/dotnet' },
              { text: 'Swift / ObjC', link: '/vi/backends/swift' },
              { text: 'Dart / Flutter', link: '/vi/backends/dart' },
              { text: 'Zig', link: '/vi/backends/zig' }
            ]
          },
          {
            text: 'Phát triển & Hệ sinh thái',
            items: [
              { text: 'Tổng quan API', link: '/vi/api/overview' },
              { text: 'Quy tắc AI Coding Agents', link: '/vi/ai-agents' },
              { text: 'Quy trình AI Agent', link: '/vi/AI_AGENT_WORKFLOW' },
              { text: 'Hướng dẫn phát triển', link: '/vi/development' },
              { text: 'Đóng góp dự án', link: '/vi/contributing' },
              { text: 'Quy trình phát hành Release', link: '/vi/releasing' },
              { text: 'Nhật ký thay đổi Changelog', link: '/vi/changelog' },
              { text: 'Kênh hỗ trợ', link: '/vi/support' },
              { text: 'Chính sách bảo mật', link: '/vi/security' },
              { text: 'Hướng dẫn dịch thuật', link: '/vi/TRANSLATION' },
              { text: 'Lộ trình phát triển', link: '/vi/ROADMAP' },
              { text: 'FAQ & Xử lý sự cố', link: '/vi/faq-troubleshooting' }
            ]
          }
        ]
      }
    },
    'zh-hans': {
      label: '简体中文',
      lang: 'zh-Hans',
      link: '/zh-hans/',
      themeConfig: {
        nav: [
          { text: '指南', link: '/zh-hans/getting-started' },
          { text: '架构', link: '/zh-hans/architecture' },
          { text: 'Crates', link: '/zh-hans/crates-architecture' },
          { text: '特性对比', link: '/zh-hans/comparison' },
          { text: '基准测试', link: '/zh-hans/benchmarks' },
          { text: '语言后端', link: '/zh-hans/backends/' },
          { text: 'IDE 集成', link: '/zh-hans/ide-integration' },
          { text: 'AI 规则', link: '/zh-hans/ai-agents' },
          { text: 'CLI 参考', link: '/zh-hans/cli-reference' }
        ],
        sidebar: [
          {
            text: '入门指南',
            items: [
              { text: '快速开始与概览', link: '/zh-hans/getting-started' },
              { text: '系统架构设计', link: '/zh-hans/architecture' },
              { text: '36 Crates 核心工作区架构', link: '/zh-hans/crates-architecture' },
              { text: 'Go 分布式服务系统', link: '/zh-hans/go-services' },
              { text: 'Python AI 预测引擎', link: '/zh-hans/python-ai-engine' },
              { text: '对比矩阵', link: '/zh-hans/comparison' },
              { text: '性能基准测试', link: '/zh-hans/benchmarks' },
              { text: '迁移指南', link: '/zh-hans/migration' },
              { text: '配置文件参考', link: '/zh-hans/configuration' },
              { text: 'IDE 集成与 LSP', link: '/zh-hans/ide-integration' },
              { text: '远程执行与 CAS', link: '/zh-hans/remote-execution' },
              { text: '多语言示例项目', link: '/zh-hans/examples-guide' },
              { text: 'CLI 命令参考', link: '/zh-hans/cli-reference' }
            ]
          },
          {
            text: '语言后端适配器',
            items: [
              { text: '所有后端', link: '/zh-hans/backends/' },
              { text: 'Rust', link: '/zh-hans/backends/rust' },
              { text: 'Go', link: '/zh-hans/backends/go' },
              { text: 'TypeScript / Node', link: '/zh-hans/backends/typescript' },
              { text: 'Python', link: '/zh-hans/backends/python' },
              { text: 'C / C++', link: '/zh-hans/backends/cc' },
              { text: 'Docker / OCI', link: '/zh-hans/backends/docker' },
              { text: 'Java / Kotlin', link: '/zh-hans/backends/java' },
              { text: 'Dotnet (.NET)', link: '/zh-hans/backends/dotnet' },
              { text: 'Swift / ObjC', link: '/zh-hans/backends/swift' },
              { text: 'Dart / Flutter', link: '/zh-hans/backends/dart' },
              { text: 'Zig', link: '/zh-hans/backends/zig' }
            ]
          },
          {
            text: '开发与生态',
            items: [
              { text: 'API 概览', link: '/zh-hans/api/overview' },
              { text: 'AI 编码智能体规则', link: '/zh-hans/ai-agents' },
              { text: 'AI Agent 工作流', link: '/zh-hans/AI_AGENT_WORKFLOW' },
              { text: '开发指南', link: '/zh-hans/development' },
              { text: '贡献指南', link: '/zh-hans/contributing' },
              { text: '版本发布指南', link: '/zh-hans/releasing' },
              { text: '更新日志 Changelog', link: '/zh-hans/changelog' },
              { text: '支持与社区', link: '/zh-hans/support' },
              { text: '安全策略', link: '/zh-hans/security' },
              { text: '多语言翻译指南', link: '/zh-hans/TRANSLATION' },
              { text: '项目路线图', link: '/zh-hans/ROADMAP' },
              { text: '常见问题排查', link: '/zh-hans/faq-troubleshooting' }
            ]
          }
        ]
      }
    },
    'zh-hant': {
      label: '繁體中文',
      lang: 'zh-Hant',
      link: '/zh-hant/',
      themeConfig: {
        nav: [
          { text: '指南', link: '/zh-hant/getting-started' },
          { text: '架構', link: '/zh-hant/architecture' },
          { text: 'Crates', link: '/zh-hant/crates-architecture' },
          { text: '特性對比', link: '/zh-hant/comparison' },
          { text: '基準測試', link: '/zh-hant/benchmarks' },
          { text: '語言後端', link: '/zh-hant/backends/' },
          { text: 'IDE 整合', link: '/zh-hant/ide-integration' },
          { text: 'AI 規則', link: '/zh-hant/ai-agents' },
          { text: 'CLI 參考', link: '/zh-hant/cli-reference' }
        ],
        sidebar: [
          {
            text: '入門指南',
            items: [
              { text: '快速開始與概覽', link: '/zh-hant/getting-started' },
              { text: '系統架構設計', link: '/zh-hant/architecture' },
              { text: '36 Crates 核心工作區架構', link: '/zh-hant/crates-architecture' },
              { text: 'Go 分散式服務系統', link: '/zh-hant/go-services' },
              { text: 'Python AI 預測引擎', link: '/zh-hant/python-ai-engine' },
              { text: '對比矩陣', link: '/zh-hant/comparison' },
              { text: '效能基準測試', link: '/zh-hant/benchmarks' },
              { text: '遷移指南', link: '/zh-hant/migration' },
              { text: '設定檔案參考', link: '/zh-hant/configuration' },
              { text: 'IDE 整合與 LSP', link: '/zh-hant/ide-integration' },
              { text: '遠端執行與 CAS', link: '/zh-hant/remote-execution' },
              { text: '多語言範例專案', link: '/zh-hant/examples-guide' },
              { text: 'CLI 命令參考', link: '/zh-hant/cli-reference' }
            ]
          },
          {
            text: '語言後端適配器',
            items: [
              { text: '所有後端', link: '/zh-hant/backends/' },
              { text: 'Rust', link: '/zh-hant/backends/rust' },
              { text: 'Go', link: '/zh-hant/backends/go' },
              { text: 'TypeScript / Node', link: '/zh-hant/backends/typescript' },
              { text: 'Python', link: '/zh-hant/backends/python' },
              { text: 'C / C++', link: '/zh-hant/backends/cc' },
              { text: 'Docker / OCI', link: '/zh-hant/backends/docker' },
              { text: 'Java / Kotlin', link: '/zh-hant/backends/java' },
              { text: 'Dotnet (.NET)', link: '/zh-hant/backends/dotnet' },
              { text: 'Swift / ObjC', link: '/zh-hant/backends/swift' },
              { text: 'Dart / Flutter', link: '/zh-hant/backends/dart' },
              { text: 'Zig', link: '/zh-hant/backends/zig' }
            ]
          },
          {
            text: '開發與生態',
            items: [
              { text: 'API 概覽', link: '/zh-hant/api/overview' },
              { text: 'AI 編碼智能體規則', link: '/zh-hant/ai-agents' },
              { text: 'AI Agent 工作流程', link: '/zh-hant/AI_AGENT_WORKFLOW' },
              { text: '開發指南', link: '/zh-hant/development' },
              { text: '貢獻指南', link: '/zh-hant/contributing' },
              { text: '版本發布指南', link: '/zh-hant/releasing' },
              { text: '更新日誌 Changelog', link: '/zh-hant/changelog' },
              { text: '支援與社群', link: '/zh-hant/support' },
              { text: '安全政策', link: '/zh-hant/security' },
              { text: '多語言翻譯指南', link: '/zh-hant/TRANSLATION' },
              { text: '項目路線圖', link: '/zh-hant/ROADMAP' },
              { text: '常見問題排查', link: '/zh-hant/faq-troubleshooting' }
            ]
          }
        ]
      }
    },
    ja: {
      label: '日本語',
      lang: 'ja',
      link: '/ja/',
      themeConfig: {
        nav: [
          { text: 'ガイド', link: '/ja/getting-started' },
          { text: 'アーキテクチャ', link: '/ja/architecture' },
          { text: 'Crates', link: '/ja/crates-architecture' },
          { text: '機能比較', link: '/ja/comparison' },
          { text: 'ベンチマーク', link: '/ja/benchmarks' },
          { text: 'バックエンド', link: '/ja/backends/' },
          { text: 'IDE 統合', link: '/ja/ide-integration' },
          { text: 'AI 規則', link: '/ja/ai-agents' },
          { text: 'CLI', link: '/ja/cli-reference' }
        ],
        sidebar: [
          {
            text: 'スタートガイド',
            items: [
              { text: '概要 & クイックスタート', link: '/ja/getting-started' },
              { text: 'システムアーキテクチャ', link: '/ja/architecture' },
              { text: '36 Crates ワークスペースアーキテクチャ', link: '/ja/crates-architecture' },
              { text: '分散 Go サービス群', link: '/ja/go-services' },
              { text: 'Python AI 予測エンジン', link: '/ja/python-ai-engine' },
              { text: '比較マトリックス', link: '/ja/comparison' },
              { text: 'パフォーマンスベンチマーク', link: '/ja/benchmarks' },
              { text: '移行ガイド', link: '/ja/migration' },
              { text: '設定ファイルリファレンス', link: '/ja/configuration' },
              { text: 'IDE 統合 & LSP', link: '/ja/ide-integration' },
              { text: 'リモート実行 & CAS', link: '/ja/remote-execution' },
              { text: 'ポリグロットサンプル', link: '/ja/examples-guide' },
              { text: 'CLI コマンドリファレンス', link: '/ja/cli-reference' }
            ]
          },
          {
            text: '言語バックエンド',
            items: [
              { text: '全バックエンド一覧', link: '/ja/backends/' },
              { text: 'Rust', link: '/ja/backends/rust' },
              { text: 'Go', link: '/ja/backends/go' },
              { text: 'TypeScript / Node', link: '/ja/backends/typescript' },
              { text: 'Python', link: '/ja/backends/python' },
              { text: 'C / C++', link: '/ja/backends/cc' },
              { text: 'Docker / OCI', link: '/ja/backends/docker' },
              { text: 'Java / Kotlin', link: '/ja/backends/java' },
              { text: 'Dotnet (.NET)', link: '/ja/backends/dotnet' },
              { text: 'Swift / ObjC', link: '/ja/backends/swift' },
              { text: 'Dart / Flutter', link: '/ja/backends/dart' },
              { text: 'Zig', link: '/ja/backends/zig' }
            ]
          },
          {
            text: '開発 & エコシステム',
            items: [
              { text: 'API 概要', link: '/ja/api/overview' },
              { text: 'AI エージェントルール', link: '/ja/ai-agents' },
              { text: 'AI Agent 開発ワークフロー', link: '/ja/AI_AGENT_WORKFLOW' },
              { text: '開発ガイド', link: '/ja/development' },
              { text: 'コントリビューションガイド', link: '/ja/contributing' },
              { text: 'リリースガイド', link: '/ja/releasing' },
              { text: '変更履歴 Changelog', link: '/ja/changelog' },
              { text: 'サポート & コミュニティ', link: '/ja/support' },
              { text: 'セキュリティポリシー', link: '/ja/security' },
              { text: '翻訳ガイドライン', link: '/ja/TRANSLATION' },
              { text: 'ロードマップ', link: '/ja/ROADMAP' },
              { text: 'FAQ & トラブルシューティング', link: '/ja/faq-troubleshooting' }
            ]
          }
        ]
      }
    }
  },

  themeConfig: {
    socialLinks: [
      { icon: 'github', link: 'https://github.com/requla11/fish' }
    ],
    search: {
      provider: 'local'
    }
  }
})
