<p align="center">
  <img src="docs/logo.png" alt="[你的品牌名] Logo" width="144">
</p>

<h1 align="center">[你的品牌名]</h1>

<p align="center">
  <a href="README_EN.md">English</a> | 简体中文
</p>

<p align="center">
  一个现代化、轻量级的 OpenClaw 配置管理工具
</p>

---

## Attribution Notice / 版权声明

**本项目基于开源项目 [OpenClawSwitch](https://github.com/RongleCat/OpenClawSwitch) 改造开发**

- **原项目**: OpenClawSwitch by RongleCat
- **原许可**: MIT License
- **修改者**: [你的名字/公司]
- **修改起始**: 2025年

本项目遵循 MIT License 开源协议。完整许可文本请参阅 [LICENSE](LICENSE) 文件。
第三方组件许可信息请参阅 [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md)。

---

## 简介

[你的品牌名] 是一款专为 **OpenClaw** 设计的可视化配置管理工具，基于 Tauri + Vue 3 构建。通过简洁直观的图形界面，让您轻松管理 AI 模型配置，无需手动编辑 JSON 文件。

### 相比原项目的改进

- ✨ 全新的 UI/UX 设计，更现代化的视觉风格
- ✨ 增强的渠道绑定功能
- ✨ 优化的快速设置向导
- ✨ 改进的用户交互体验
- ✨ 额外的功能增强

---

## 功能特性

- **可视化配置管理** - 告别手动编辑 JSON，通过图形界面轻松管理配置
- **多服务商支持** - 支持 OpenAI、Anthropic、Ollama 等多种 AI 服务商
- **模型快速切换** - 一键切换主要模型和备用模型
- **本地/远程模式** - 支持本地配置自动保存和远程文件手动管理
- **跨平台** - 支持 Windows、macOS (Apple/Intel)
- **极致轻量** - 打包体积仅 3-5MB，内存占用低

---

## 安装

### 下载 Release

前往 [Releases](https://github.com/YourOrg/YourRepo/releases) 页面下载对应平台的安装包：

| 平台 | 文件 |
|------|------|
| Windows | `[你的品牌名]_x.x.x_x64-setup.exe` 或 `.msi` |
| macOS (Apple Silicon) | `[你的品牌名]_x.x.x_aarch64.dmg` |
| macOS (Intel) | `[你的品牌名]_x.x.x_x64.dmg` |

> **macOS 用户注意**：由于应用未签名，首次运行需要在终端执行：
> ```bash
> xattr -c /Applications/[你的品牌名].app
> ```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/YourOrg/YourRepo.git
cd YourRepo

# 安装依赖
npm install

# 开发模式
npm run tauri:dev

# 构建发布版
npm run tauri:build
```

**构建依赖**：
- Node.js 18+
- Rust 1.70+
- Windows: [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- macOS: `xcode-select --install`

---

## 使用方法

### 1. 启动应用

应用启动后会自动加载默认配置文件 `~/.openclaw/openclaw.json`。

### 2. 添加服务商

点击「添加」按钮，填写服务商信息：

| 字段 | 说明 | 示例 |
|------|------|------|
| 服务商名称 | 自定义标识符 | `openai` |
| Base URL | API 基础地址 | `https://api.openai.com/v1` |
| API Key | 可选，也可通过环境变量设置 | `sk-xxx` |

**快速填充**：点击 `OpenAI` / `Anthropic` / `Ollama` 按钮自动填入常用配置。

### 3. 添加模型

在服务商卡片中点击「+」添加模型 ID，如 `gpt-4o`、`claude-sonnet-4-20250514`。

### 4. 切换模型

- **主要模型**：在左侧「模型配置」区域的下拉菜单中选择
- **备用模型**：点击「添加备用」选择备用模型列表

### 5. 工具功能

- **重启网关** - 重启 OpenClaw 网关服务
- **打开 TUI** - 打开 OpenClaw 终端界面

---

## 配置文件

配置文件位置：
- **Windows**: `%USERPROFILE%\.openclaw\openclaw.json`
- **macOS**: `~/.openclaw/openclaw.json`

配置格式示例：
```json
{
  "models": {
    "providers": {
      "openai": {
        "baseUrl": "https://api.openai.com/v1",
        "apiKey": "sk-..."
      }
    }
  },
  "agent": {
    "model": "openai/gpt-4o"
  }
}
```

---

## 技术栈

| 类别 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + Vite |
| UI | Tailwind CSS + Lucide Icons |
| 桌面框架 | Tauri 1.5 |
| 后端 | Rust |

---

## 许可协议

本项目采用 **MIT License** 开源协议。

- 原项目 OpenClawSwitch 由 RongleCat 开发，采用 MIT License
- 本项目的修改部分同样采用 MIT License
- 完整许可文本: [LICENSE](LICENSE)
- 第三方组件许可: [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md)

---

## 相关链接

- [原项目: OpenClawSwitch](https://github.com/RongleCat/OpenClawSwitch)
- [OpenClaw](https://github.com/miaoxworld/openclaw-manager)
- [Tauri](https://tauri.app/)
- [Vue 3](https://vuejs.org/)

---

## 支持与反馈

- 📧 Email: [your-email@example.com]
- 💬 微信群: [二维码图片]
- 📖 文档: [https://your-docs-site.com]
- 🐛 问题反馈: [https://github.com/YourOrg/YourRepo/issues]

---

## 开发致谢

感谢 OpenClawSwitch 原作者 [RongleCat](https://github.com/RongleCat) 的出色工作。

本项目基于 OpenClawSwitch (MIT License) 进行了深度定制和功能增强。

---

**Copyright (c) 2026 OpenClawSwitch Contributors**
**Modifications Copyright (c) 2025 [Your Name / Your Company]**
