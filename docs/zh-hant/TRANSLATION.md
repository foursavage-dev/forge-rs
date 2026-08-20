# 文档翻译与本地化贡献指南

我们非常欢迎社区贡献者将 Fish 的文档翻译为**任何语言**。本指南概述了翻译工作流、披露政策、宽松的贡献者守则以及最佳实践。

---

## 适用范围

翻译仅适用于**文档内容**（`docs/` 目录下的 Markdown 文件及项目指南）。

- **源代码、单元测试、变量标识符以及 Git 提交信息**必须保持 **100% 英文**。
- 文档中的 **CLI 命令、参数标志、代码片段及配置键名**不得进行翻译。

---

## 文档贡献者的宽松守则

为了让文档贡献变得尽可能简单友好，我们制定了以下宽松政策：

1. **欢迎增量/部分翻译**：
   - 您无需一次性完成整篇文档的翻译。
   - 仅翻译单个小节（例如*安装*或*快速入门*）同样非常欢迎。
   - 暂未翻译的章节可以直接保留英文原文，或添加形如 `<!-- TODO: translate this section -->` 的注释。

2. **无需在本地搭建 Rust 环境**：
   - 您无需在本地克隆代码仓库或安装 Rust 编译环境。
   - 您可以直接在 **GitHub 网页端**点击任何 Markdown 文件上的铅笔图标进行在线编辑。

3. **快速通道审核**：
   - 针对错别字修正、格式调整、链接更新与翻译润色的 Pull Request 将走快速通道，迅速完成合并。

---

## 支持的语言列表

Fish 接受**所有语言**的文档翻译。除开放所有社区语言外，Fish 优先维护 5 种核心语言：

| 语言 | 语言代码 | 状态 | 定位 |
| :--- | :--- | :--- | :--- |
| **英語** (English) | `en` | 活跃 | 官方权威数据源 |
| **簡體中文** | `zh-CN` | 开放 | 核心社区语言 |
| **繁體中文** | `zh-TW` | 开放 | 核心社区语言 |
| **日語** (日本語) | `ja` | 开放 | 核心社区语言 |
| **越南语** (Tiếng Việt) | `vi` | 开放 | 核心社区语言 |
| **所有其他语言**（西、法、德、韩等） | `*` | 开放 | 社区扩展语言 |

---

## 工具政策与机器翻译强制披露

### 1. 允许使用自动化与 AI 工具
贡献者可以使用 AI 助手（ChatGPT、Claude、Gemini）以及机器翻译引擎（DeepL、Google 翻译）来起草或加速翻译进程。

### 2. 强制性机器翻译披露
所有提交的翻译均需保持透明。在提交翻译 Pull Request 时，您**必须**在 PR 说明中披露是否使用了自动化翻译工具。

请在 PR 中注明对应等级：
- **Tier 1** - 纯人工母语翻译：100% 由母语者或流利掌握该语言的人员人工翻译。
- **Tier 2** - AI / 机器辅助翻译并经母语者校对：由机器或 AI 生成初稿，并经过母语者彻底校对与修正。
- **Tier 3** - 纯机器 / AI 翻译草稿（待校对）：直接由机器/AI 生成，等待社区母语者进行深度审核。

### 3. 母语者校对优先合并
经过母语者校对或撰写的翻译将优先合并，以确保技术表述的专业度与自然流畅度。

---

## 目录结构规划

翻译文件按照标准 ISO 语言代码归类于 `docs/` 下的子目录中：

```text
docs/
├── getting-started.md       # 英文（权威源文件）
├── architecture.md          # 英文（权威源文件）
├── vi/                      # 越南语
│   ├── getting-started.md
│   └── architecture.md
├── zh-CN/                   # 簡體中文
│   ├── getting-started.md
│   └── architecture.md
├── zh-TW/                   # 繁體中文
│   ├── getting-started.md
│   └── architecture.md
├── ja/                      # 日語
│   ├── getting-started.md
│   └── architecture.md
└── <lang-code>/             # 任何其他语言 (例如 es, fr, de, ko)
    └── getting-started.md
```

---

## 如何提交翻译

### 方式 A：通过 GitHub 网页界面（最简单）
1. 在 GitHub 上浏览至 `docs/` 目录下的目标文件。
2. 点击右上角的 **Edit this file**（铅笔）图标。
3. 保存修改至新分支并提交 Pull Request。

### 方式 B：通过 Git 命令行
1. Fork 仓库并克隆到本地：
   ```bash
   git clone https://github.com/<your-username>/fish.git
   cd fish
   git checkout -b docs/translate-<lang>-<topic>
   ```
2. 在 `docs/<lang-code>/` 中新增或修改 Markdown 文件。
3. 使用英文提交信息进行 Commit（例如 `docs: translate getting-started to Chinese`）。
4. 推送至您的 Fork 分支并向 `dev` 分支发起 Pull Request。
