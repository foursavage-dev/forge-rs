import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Fish',
  description: 'Fast, Polyglot Build Orchestration & Cache Acceleration System',
  base: '/fish/',
  lastUpdated: true,
  cleanUrls: true,
  ignoreDeadLinks: true,

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/getting-started' },
          { text: 'Architecture', link: '/architecture' },
          { text: 'CLI', link: '/cli-reference' },
          {
            text: 'Backends',
            items: [
              { text: 'Rust', link: '/backends/rust' },
              { text: 'Go', link: '/backends/go' },
              { text: 'TypeScript', link: '/backends/typescript' },
              { text: 'Python', link: '/backends/python' },
              { text: 'C / C++', link: '/backends/cc' },
              { text: 'Docker', link: '/backends/docker' },
              { text: 'Java', link: '/backends/java' },
              { text: 'Dotnet', link: '/backends/dotnet' }
            ]
          },
          { text: 'Config', link: '/configuration' },
          { text: 'FAQ', link: '/faq-troubleshooting' },
          { text: 'Translations', link: '/TRANSLATION' }
        ],
        sidebar: [
          {
            text: 'Getting Started',
            items: [
              { text: 'Introduction & Setup', link: '/getting-started' },
              { text: 'Configuration Reference', link: '/configuration' },
              { text: 'CLI Reference', link: '/cli-reference' },
              { text: 'Translation Guidelines', link: '/TRANSLATION' },
              { text: 'FAQ & Troubleshooting', link: '/faq-troubleshooting' }
            ]
          },
          {
            text: 'Architecture & Internals',
            items: [
              { text: 'System Architecture', link: '/architecture' },
              { text: 'AI Agent Workflow', link: '/AI_AGENT_WORKFLOW' },
              { text: 'API Overview', link: '/api/overview' }
            ]
          },
          {
            text: 'Language Backends',
            items: [
              { text: 'Rust', link: '/backends/rust' },
              { text: 'Go', link: '/backends/go' },
              { text: 'TypeScript / Node.js', link: '/backends/typescript' },
              { text: 'Python', link: '/backends/python' },
              { text: 'C / C++', link: '/backends/cc' },
              { text: 'Docker Containers', link: '/backends/docker' },
              { text: 'Java', link: '/backends/java' },
              { text: 'Dotnet (.NET)', link: '/backends/dotnet' }
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
          { text: 'Bắt đầu', link: '/vi/getting-started' },
          { text: 'Kiến trúc', link: '/vi/architecture' },
          { text: 'Lệnh CLI', link: '/vi/cli-reference' },
          {
            text: 'Backends',
            items: [
              { text: 'Rust', link: '/vi/backends/rust' },
              { text: 'Go', link: '/vi/backends/go' },
              { text: 'TypeScript', link: '/vi/backends/typescript' },
              { text: 'Python', link: '/vi/backends/python' },
              { text: 'C / C++', link: '/vi/backends/cc' },
              { text: 'Docker', link: '/vi/backends/docker' },
              { text: 'Java', link: '/vi/backends/java' },
              { text: 'Dotnet', link: '/vi/backends/dotnet' }
            ]
          },
          { text: 'Cấu hình', link: '/vi/configuration' },
          { text: 'Hỏi đáp', link: '/vi/faq-troubleshooting' },
          { text: 'Dịch thuật', link: '/vi/TRANSLATION' }
        ],
        sidebar: [
          {
            text: 'Hướng Dẫn Bắt Đầu',
            items: [
              { text: 'Cài đặt & Khởi chạy', link: '/vi/getting-started' },
              { text: 'Tài liệu cấu hình', link: '/vi/configuration' },
              { text: 'Danh sách lệnh CLI', link: '/vi/cli-reference' },
              { text: 'Hướng dẫn dịch thuật', link: '/vi/TRANSLATION' },
              { text: 'Hỏi đáp & Xử lý lỗi', link: '/vi/faq-troubleshooting' }
            ]
          },
          {
            text: 'Kiến Trúc & Lõi Hệ Thống',
            items: [
              { text: 'Kiến trúc tổng quan', link: '/vi/architecture' },
              { text: 'Quy trình AI Agent', link: '/vi/AI_AGENT_WORKFLOW' },
              { text: 'Tổng quan API', link: '/vi/api/overview' }
            ]
          },
          {
            text: 'Language Backends',
            items: [
              { text: 'Rust', link: '/vi/backends/rust' },
              { text: 'Go', link: '/vi/backends/go' },
              { text: 'TypeScript / Node.js', link: '/vi/backends/typescript' },
              { text: 'Python', link: '/vi/backends/python' },
              { text: 'C / C++', link: '/vi/backends/cc' },
              { text: 'Docker Containers', link: '/vi/backends/docker' },
              { text: 'Java', link: '/vi/backends/java' },
              { text: 'Dotnet (.NET)', link: '/vi/backends/dotnet' }
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
          { text: '命令行', link: '/zh-hans/cli-reference' },
          {
            text: '后端支持',
            items: [
              { text: 'Rust', link: '/zh-hans/backends/rust' },
              { text: 'Go', link: '/zh-hans/backends/go' },
              { text: 'TypeScript', link: '/zh-hans/backends/typescript' },
              { text: 'Python', link: '/zh-hans/backends/python' },
              { text: 'C / C++', link: '/zh-hans/backends/cc' },
              { text: 'Docker', link: '/zh-hans/backends/docker' },
              { text: 'Java', link: '/zh-hans/backends/java' },
              { text: 'Dotnet', link: '/zh-hans/backends/dotnet' }
            ]
          },
          { text: '配置', link: '/zh-hans/configuration' },
          { text: '常见问题', link: '/zh-hans/faq-troubleshooting' },
          { text: '翻译指南', link: '/zh-hans/TRANSLATION' }
        ],
        sidebar: [
          {
            text: '入门指南',
            items: [
              { text: '快速开始与安装', link: '/zh-hans/getting-started' },
              { text: '配置参考', link: '/zh-hans/configuration' },
              { text: 'CLI 命令大全', link: '/zh-hans/cli-reference' },
              { text: '翻译与贡献指南', link: '/zh-hans/TRANSLATION' },
              { text: '常见问题与排错', link: '/zh-hans/faq-troubleshooting' }
            ]
          },
          {
            text: '系统架构',
            items: [
              { text: '架构详解', link: '/zh-hans/architecture' },
              { text: 'AI Agent 工作流', link: '/zh-hans/AI_AGENT_WORKFLOW' },
              { text: 'API 概览', link: '/zh-hans/api/overview' }
            ]
          },
          {
            text: '语言后端',
            items: [
              { text: 'Rust', link: '/zh-hans/backends/rust' },
              { text: 'Go', link: '/zh-hans/backends/go' },
              { text: 'TypeScript / Node.js', link: '/zh-hans/backends/typescript' },
              { text: 'Python', link: '/zh-hans/backends/python' },
              { text: 'C / C++', link: '/zh-hans/backends/cc' },
              { text: 'Docker 容器', link: '/zh-hans/backends/docker' },
              { text: 'Java', link: '/zh-hans/backends/java' },
              { text: 'Dotnet (.NET)', link: '/zh-hans/backends/dotnet' }
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
          { text: '命令行', link: '/zh-hant/cli-reference' },
          {
            text: '後端支援',
            items: [
              { text: 'Rust', link: '/zh-hant/backends/rust' },
              { text: 'Go', link: '/zh-hant/backends/go' },
              { text: 'TypeScript', link: '/zh-hant/backends/typescript' },
              { text: 'Python', link: '/zh-hant/backends/python' },
              { text: 'C / C++', link: '/zh-hant/backends/cc' },
              { text: 'Docker', link: '/zh-hant/backends/docker' },
              { text: 'Java', link: '/zh-hant/backends/java' },
              { text: 'Dotnet', link: '/zh-hant/backends/dotnet' }
            ]
          },
          { text: '配置', link: '/zh-hant/configuration' },
          { text: '常見問題', link: '/zh-hant/faq-troubleshooting' },
          { text: '翻譯指南', link: '/zh-hant/TRANSLATION' }
        ],
        sidebar: [
          {
            text: '入門指南',
            items: [
              { text: '快速開始與安裝', link: '/zh-hant/getting-started' },
              { text: '配置參考', link: '/zh-hant/configuration' },
              { text: 'CLI 命令大全', link: '/zh-hant/cli-reference' },
              { text: '翻譯與貢獻指南', link: '/zh-hant/TRANSLATION' },
              { text: '常見問題與排錯', link: '/zh-hant/faq-troubleshooting' }
            ]
          },
          {
            text: '系統架構',
            items: [
              { text: '架構詳解', link: '/zh-hant/architecture' },
              { text: 'AI Agent 工作流', link: '/zh-hant/AI_AGENT_WORKFLOW' },
              { text: 'API 概覽', link: '/zh-hant/api/overview' }
            ]
          },
          {
            text: '語言後端',
            items: [
              { text: 'Rust', link: '/zh-hant/backends/rust' },
              { text: 'Go', link: '/zh-hant/backends/go' },
              { text: 'TypeScript / Node.js', link: '/zh-hant/backends/typescript' },
              { text: 'Python', link: '/zh-hant/backends/python' },
              { text: 'C / C++', link: '/zh-hant/backends/cc' },
              { text: 'Docker 容器', link: '/zh-hant/backends/docker' },
              { text: 'Java', link: '/zh-hant/backends/java' },
              { text: 'Dotnet (.NET)', link: '/zh-hant/backends/dotnet' }
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
          { text: 'CLI', link: '/ja/cli-reference' },
          {
            text: 'バックエンド',
            items: [
              { text: 'Rust', link: '/ja/backends/rust' },
              { text: 'Go', link: '/ja/backends/go' },
              { text: 'TypeScript', link: '/ja/backends/typescript' },
              { text: 'Python', link: '/ja/backends/python' },
              { text: 'C / C++', link: '/ja/backends/cc' },
              { text: 'Docker', link: '/ja/backends/docker' },
              { text: 'Java', link: '/ja/backends/java' },
              { text: 'Dotnet', link: '/ja/backends/dotnet' }
            ]
          },
          { text: '設定', link: '/ja/configuration' },
          { text: 'FAQ', link: '/ja/faq-troubleshooting' },
          { text: '翻訳ガイド', link: '/ja/TRANSLATION' }
        ],
        sidebar: [
          {
            text: 'スタートガイド',
            items: [
              { text: '導入とセットアップ', link: '/ja/getting-started' },
              { text: '設定リファレンス', link: '/ja/configuration' },
              { text: 'CLI コマンドリファレンス', link: '/ja/cli-reference' },
              { text: '翻訳と貢献ガイド', link: '/ja/TRANSLATION' },
              { text: 'FAQ とトラブルシューティング', link: '/ja/faq-troubleshooting' }
            ]
          },
          {
            text: 'アーキテクチャと内部構造',
            items: [
              { text: 'システムアーキテクチャ', link: '/ja/architecture' },
              { text: 'AI Agent ワークフロー', link: '/ja/AI_AGENT_WORKFLOW' },
              { text: 'API 概要', link: '/ja/api/overview' }
            ]
          },
          {
            text: '言語バックエンド',
            items: [
              { text: 'Rust', link: '/ja/backends/rust' },
              { text: 'Go', link: '/ja/backends/go' },
              { text: 'TypeScript / Node.js', link: '/ja/backends/typescript' },
              { text: 'Python', link: '/ja/backends/python' },
              { text: 'C / C++', link: '/ja/backends/cc' },
              { text: 'Docker コンテナ', link: '/ja/backends/docker' },
              { text: 'Java', link: '/ja/backends/java' },
              { text: 'Dotnet (.NET)', link: '/ja/backends/dotnet' }
            ]
          }
        ]
      }
    }
  },

  themeConfig: {
    siteTitle: 'Fish',
    logo: '/logo.svg',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/requla11/fish' }
    ],
    search: {
      provider: 'local'
    },
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026 requla11'
    }
  }
})
