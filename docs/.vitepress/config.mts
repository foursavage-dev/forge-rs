import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Fish',
  description: 'Fast, Polyglot Build Orchestration & Cache Acceleration System',
  base: '/fish/',
  lastUpdated: true,
  cleanUrls: true,
  ignoreDeadLinks: false,

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
          { text: 'Roadmap', link: '/ROADMAP' },
          { text: 'FAQ', link: '/faq-troubleshooting' },
          { text: 'Dev Guide', link: '/development' },
          { text: 'Translations', link: '/TRANSLATION' }
        ],
        sidebar: [
          {
            text: 'Getting Started',
            items: [
              { text: 'Introduction & Setup', link: '/getting-started' },
              { text: 'Configuration Reference', link: '/configuration' },
              { text: 'CLI Reference', link: '/cli-reference' },
              { text: 'Roadmap', link: '/ROADMAP' },
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
          },
          {
            text: 'Development & Community',
            items: [
              { text: 'Development Guide', link: '/development' },
              { text: 'Contributing', link: '/contributing' },
              { text: 'Support & Community', link: '/support' },
              { text: 'Security Policy', link: '/security' }
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
          { text: 'Lộ trình', link: '/vi/ROADMAP' },
          { text: 'Hỏi đáp', link: '/vi/faq-troubleshooting' },
          { text: 'Phát triển', link: '/vi/development' },
          { text: 'Dịch thuật', link: '/vi/TRANSLATION' }
        ],
        sidebar: [
          {
            text: 'Bắt đầu',
            items: [
              { text: 'Giới thiệu & Cài đặt', link: '/vi/getting-started' },
              { text: 'Tài liệu cấu hình', link: '/vi/configuration' },
              { text: 'Danh mục lệnh CLI', link: '/vi/cli-reference' },
              { text: 'Lộ trình phát triển', link: '/vi/ROADMAP' },
              { text: 'Hướng dẫn dịch thuật', link: '/vi/TRANSLATION' },
              { text: 'Hỏi đáp & Xử lý sự cố', link: '/vi/faq-troubleshooting' }
            ]
          },
          {
            text: 'Kiến trúc & Hệ thống',
            items: [
              { text: 'Kiến trúc hệ thống', link: '/vi/architecture' },
              { text: 'Quy trình AI Agent', link: '/vi/AI_AGENT_WORKFLOW' },
              { text: 'Tổng quan API', link: '/vi/api/overview' }
            ]
          },
          {
            text: 'Backend ngôn ngữ',
            items: [
              { text: 'Rust', link: '/vi/backends/rust' },
              { text: 'Go', link: '/vi/backends/go' },
              { text: 'TypeScript / Node.js', link: '/vi/backends/typescript' },
              { text: 'Python', link: '/vi/backends/python' },
              { text: 'C / C++', link: '/vi/backends/cc' },
              { text: 'Docker', link: '/vi/backends/docker' },
              { text: 'Java', link: '/vi/backends/java' },
              { text: 'Dotnet (.NET)', link: '/vi/backends/dotnet' }
            ]
          },
          {
            text: 'Phát triển & Cộng đồng',
            items: [
              { text: 'Hướng dẫn phát triển', link: '/vi/development' },
              { text: 'Đóng góp dự án', link: '/vi/contributing' },
              { text: 'Hỗ trợ & Cộng đồng', link: '/vi/support' },
              { text: 'Chính sách bảo mật', link: '/vi/security' }
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
          { text: '快速入门', link: '/zh-hans/getting-started' },
          { text: '系统架构', link: '/zh-hans/architecture' },
          { text: '命令参考', link: '/zh-hans/cli-reference' },
          {
            text: '语言后端',
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
          { text: '配置参考', link: '/zh-hans/configuration' },
          { text: '路线图', link: '/zh-hans/ROADMAP' },
          { text: '常见问题', link: '/zh-hans/faq-troubleshooting' },
          { text: '开发指南', link: '/zh-hans/development' },
          { text: '翻译指南', link: '/zh-hans/TRANSLATION' }
        ],
        sidebar: [
          {
            text: '新手指南',
            items: [
              { text: '简介与快速安装', link: '/zh-hans/getting-started' },
              { text: '配置选项指南', link: '/zh-hans/configuration' },
              { text: 'CLI 命令大全', link: '/zh-hans/cli-reference' },
              { text: '项目发展路线图', link: '/zh-hans/ROADMAP' },
              { text: '文档翻译指南', link: '/zh-hans/TRANSLATION' },
              { text: '常见问题与排错', link: '/zh-hans/faq-troubleshooting' }
            ]
          },
          {
            text: '架构与内核',
            items: [
              { text: '系统架构概览', link: '/zh-hans/architecture' },
              { text: 'AI Agent 工作流', link: '/zh-hans/AI_AGENT_WORKFLOW' },
              { text: 'API 接口文档', link: '/zh-hans/api/overview' }
            ]
          },
          {
            text: '支持的语言后端',
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
          },
          {
            text: '开发与社区',
            items: [
              { text: '开发指南', link: '/zh-hans/development' },
              { text: '参与贡献', link: '/zh-hans/contributing' },
              { text: '技术支持与社区', link: '/zh-hans/support' },
              { text: '安全政策', link: '/zh-hans/security' }
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
          { text: '快速入門', link: '/zh-hant/getting-started' },
          { text: '系統架構', link: '/zh-hant/architecture' },
          { text: '命令參考', link: '/zh-hant/cli-reference' },
          {
            text: '語言後端',
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
          { text: '配置參考', link: '/zh-hant/configuration' },
          { text: '路線圖', link: '/zh-hant/ROADMAP' },
          { text: '常見問題', link: '/zh-hant/faq-troubleshooting' },
          { text: '開發指南', link: '/zh-hant/development' },
          { text: '翻譯指南', link: '/zh-hant/TRANSLATION' }
        ],
        sidebar: [
          {
            text: '新手指南',
            items: [
              { text: '簡介與快速安裝', link: '/zh-hant/getting-started' },
              { text: '配置選項指南', link: '/zh-hant/configuration' },
              { text: 'CLI 命令大全', link: '/zh-hant/cli-reference' },
              { text: '項目發展路線圖', link: '/zh-hant/ROADMAP' },
              { text: '文檔翻譯指南', link: '/zh-hant/TRANSLATION' },
              { text: '常見問題與排除', link: '/zh-hant/faq-troubleshooting' }
            ]
          },
          {
            text: '架構與內核',
            items: [
              { text: '系統架構概覽', link: '/zh-hant/architecture' },
              { text: 'AI Agent 工作流', link: '/zh-hant/AI_AGENT_WORKFLOW' },
              { text: 'API 介面文件', link: '/zh-hant/api/overview' }
            ]
          },
          {
            text: '支持的語言後端',
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
          },
          {
            text: '開發與社群',
            items: [
              { text: '開發指南', link: '/zh-hant/development' },
              { text: '參與貢獻', link: '/zh-hant/contributing' },
              { text: '技術支持與社群', link: '/zh-hant/support' },
              { text: '安全政策', link: '/zh-hant/security' }
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
          { text: 'ロードマップ', link: '/ja/ROADMAP' },
          { text: 'FAQ', link: '/ja/faq-troubleshooting' },
          { text: '開発ガイド', link: '/ja/development' },
          { text: '翻訳', link: '/ja/TRANSLATION' }
        ],
        sidebar: [
          {
            text: '導入',
            items: [
              { text: '概要とセットアップ', link: '/ja/getting-started' },
              { text: '設定リファレンス', link: '/ja/configuration' },
              { text: 'CLI リファレンス', link: '/ja/cli-reference' },
              { text: 'ロードマップ', link: '/ja/ROADMAP' },
              { text: '翻訳ガイドライン', link: '/ja/TRANSLATION' },
              { text: 'よくある質問とトラブルシューティング', link: '/ja/faq-troubleshooting' }
            ]
          },
          {
            text: 'アーキテクチャ & 内部構造',
            items: [
              { text: 'システムアーキテクチャ', link: '/ja/architecture' },
              { text: 'AI Agent 開発手順', link: '/ja/AI_AGENT_WORKFLOW' },
              { text: 'API リファレンス', link: '/ja/api/overview' }
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
              { text: 'Docker', link: '/ja/backends/docker' },
              { text: 'Java', link: '/ja/backends/java' },
              { text: 'Dotnet (.NET)', link: '/ja/backends/dotnet' }
            ]
          },
          {
            text: '開発とコミュニティ',
            items: [
              { text: '開発ガイド', link: '/ja/development' },
              { text: 'コントリビューション', link: '/ja/contributing' },
              { text: 'サポートとコミュニティ', link: '/ja/support' },
              { text: 'セキュリティポリシー', link: '/ja/security' }
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
    footer: {
      message: 'Released under Apache-2.0 / MIT License.',
      copyright: 'Copyright © 2026-present Fish Build Team'
    }
  }
})
