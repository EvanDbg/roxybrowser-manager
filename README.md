# 🦊 RoxyBrowser Manager

> RoxyBrowser 多账户管理与快速切换工具

[English](#english) | [中文](#中文)

---

## 中文

### 简介

RoxyBrowser Manager 是一款基于 [Tauri v2](https://v2.tauri.app/) 构建的桌面应用，用于管理和快速切换 RoxyBrowser 的多个用户配置。适合需要在不同账户之间频繁切换的用户，一键切换、启停浏览器，简洁高效。

### ✨ 功能特性

- **多账户管理** — 添加、删除、编辑多个用户配置
- **一键切换** — 快速切换当前使用的浏览器配置
- **启停控制** — 直接从管理器启动或停止 RoxyBrowser
- **配置导入/导出** — 轻松备份和迁移用户配置
- **系统托盘** — 最小化到系统托盘，随时快速操作
- **深色/浅色主题** — 支持主题切换，保护你的眼睛
- **快捷键** — `Ctrl+N` 添加用户 / `Ctrl+R` 刷新状态
- **自动检测** — 自动检测 RoxyBrowser 安装路径

### 🛠 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | [Tauri v2](https://v2.tauri.app/) |
| 前端 | React 19 + TypeScript + Vite 6 |
| 样式 | TailwindCSS 3 + DaisyUI 4 |
| 状态管理 | Zustand 5 |
| 图标 | Lucide React |
| 后端 | Rust (Tokio, Serde, Sysinfo) |

### 📦 安装

前往 [Releases](https://github.com/EvanDbg/roxybrowser-manager/releases) 页面下载最新版本：

| 平台 | 文件 | 说明 |
|------|------|------|
| **macOS (Apple Silicon)** | `.dmg` | 适用于 M1/M2/M3/M4 芯片的 Mac |
| **Windows** | `.exe` | Windows 安装程序 (NSIS) |

> **macOS 用户**：如果遇到"无法打开"的提示，请在 **系统偏好设置 > 安全性与隐私** 中允许运行。
>
> **Windows 用户**：如果遇到 SmartScreen 警告，请选择"仍要运行"。

### 🚀 开发指南

#### 前置要求

- [Node.js](https://nodejs.org/) >= 20
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/)

#### 本地开发

```bash
# 克隆仓库
git clone https://github.com/EvanDbg/roxybrowser-manager.git
cd roxybrowser-manager

# 安装前端依赖
npm install

# 启动开发模式
npm run tauri dev
```

#### 构建生产版本

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

### 📁 项目结构

```
roxybrowser-manager/
├── src/                    # 前端源码 (React)
│   ├── components/         # UI 组件
│   ├── stores/             # Zustand 状态管理
│   └── types/              # TypeScript 类型定义
├── src-tauri/              # 后端源码 (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri 命令
│   │   ├── models/         # 数据模型
│   │   └── lib.rs          # 应用入口
│   └── tauri.conf.json     # Tauri 配置
├── .github/workflows/      # CI/CD (自动构建 & 发布)
└── package.json
```

### 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 📄 许可证

[MIT License](LICENSE)

---

## English

### Introduction

RoxyBrowser Manager is a desktop application built with [Tauri v2](https://v2.tauri.app/) for managing and quickly switching between multiple RoxyBrowser user profiles. Perfect for users who need to frequently switch between different accounts.

### ✨ Features

- **Multi-Account Management** — Add, delete, and edit multiple user profiles
- **One-Click Switch** — Quickly switch the active browser profile
- **Start/Stop Control** — Launch or stop RoxyBrowser directly from the manager
- **Import/Export** — Easily backup and migrate user profiles
- **System Tray** — Minimize to system tray for quick access
- **Dark/Light Theme** — Toggle between themes
- **Keyboard Shortcuts** — `Ctrl+N` to add user / `Ctrl+R` to refresh
- **Auto-Detection** — Automatically detect RoxyBrowser installation path

### 🛠 Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://v2.tauri.app/) |
| Frontend | React 19 + TypeScript + Vite 6 |
| Styling | TailwindCSS 3 + DaisyUI 4 |
| State | Zustand 5 |
| Icons | Lucide React |
| Backend | Rust (Tokio, Serde, Sysinfo) |

### 📦 Installation

Download the latest release from the [Releases](https://github.com/EvanDbg/roxybrowser-manager/releases) page:

| Platform | File | Description |
|----------|------|-------------|
| **macOS (Apple Silicon)** | `.dmg` | For M1/M2/M3/M4 Macs |
| **Windows** | `.exe` | Windows installer (NSIS) |

### 🚀 Development

#### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/)

#### Local Development

```bash
git clone https://github.com/EvanDbg/roxybrowser-manager.git
cd roxybrowser-manager
npm install
npm run tauri dev
```

#### Production Build

```bash
npm run tauri build
```

Build artifacts are located in `src-tauri/target/release/bundle/`.

### 📁 Project Structure

```
roxybrowser-manager/
├── src/                    # Frontend (React)
│   ├── components/         # UI components
│   ├── stores/             # Zustand state management
│   └── types/              # TypeScript types
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── models/         # Data models
│   │   └── lib.rs          # App entry point
│   └── tauri.conf.json     # Tauri config
├── .github/workflows/      # CI/CD (auto build & release)
└── package.json
```

### 🤝 Contributing

Issues and Pull Requests are welcome!

### 📄 License

[MIT License](LICENSE)

https://github.com/EvanDbg
