# GitHub CI/CD 流程设计文档

## 1. 概述

### 1.1 背景
RoxyBrowser Manager 项目需要实现自动化构建和发布流程，以便在每次发布新版本时，能够自动构建多平台安装包并发布到 GitHub Releases。

### 1.2 目标
- 自动构建 **macOS (Apple Silicon/M芯片)** 版本的安装文件 (`.dmg`)
- 自动构建 **Windows x64** 版本的安装文件 (`.msi`, `.exe`)
- 在每次推送 tag 时，自动将构建产物上传到 GitHub Release

### 1.3 技术栈
- **构建框架**: Tauri 2.x
- **前端**: React + TypeScript + Vite
- **后端**: Rust
- **CI/CD**: GitHub Actions

---

## 2. 触发条件

### 2.1 Tag 触发规则
Workflow 仅在推送符合以下格式的 tag 时触发：

```yaml
on:
  push:
    tags:
      - 'v*'  # 匹配所有以 'v' 开头的 tag，如 v1.0.0, v2.1.3
```

### 2.2 版本命名规范
建议遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范：
- `v1.0.0` - 初始发布
- `v1.0.1` - 补丁版本（Bug 修复）
- `v1.1.0` - 次版本（新功能，向后兼容）
- `v2.0.0` - 主版本（重大变更，可能不兼容）

---

## 3. 构建矩阵

| 平台 | Runner | 目标架构 | 输出格式 |
|------|--------|----------|----------|
| macOS | `macos-14` | `aarch64-apple-darwin` | `.dmg`, `.app` |
| Windows | `windows-latest` | `x86_64-pc-windows-msvc` | `.msi`, `.exe` |

### 3.1 macOS 构建说明
- 使用 `macos-14` runner，该 runner 默认运行在 Apple Silicon (M1) 硬件上
- 编译目标为 `aarch64-apple-darwin` (ARM64)
- 输出 DMG 磁盘映像，用户可直接拖拽安装

### 3.2 Windows 构建说明
- 使用 `windows-latest` runner (Windows Server 2022)
- 默认编译为 x64 架构
- 输出两种安装程序：
  - **MSI**: Microsoft Installer 格式，适用于企业部署
  - **EXE**: NSIS 安装程序，适用于普通用户

---

## 4. Workflow 详细流程

### 4.1 流程图

```
[Push Tag v*] 
      │
      ├──────────────────┬────────────────────┐
      ▼                  ▼                    │
┌─────────────┐   ┌─────────────┐             │
│ build-macos │   │build-windows│             │
│  (macos-14) │   │  (windows)  │             │
└──────┬──────┘   └──────┬──────┘             │
       │                 │                    │
       │    ┌────────────┘                    │
       ▼    ▼                                 │
  ┌──────────────┐                            │
  │ Upload       │                            │
  │ Artifacts    │                            │
  └──────┬───────┘                            │
         │                                    │
         ▼                                    │
  ┌───────────────┐                           │
  │ create-release│ ◄─────────────────────────┘
  │  (ubuntu)     │   needs: [build-macos, build-windows]
  └───────┬───────┘
          │
          ▼
  ┌───────────────┐
  │ GitHub Release│
  │  with Assets  │
  └───────────────┘
```

### 4.2 Job 详情

#### Job 1: build-macos
1. 检出代码
2. 配置 Node.js 20.x
3. 配置 Rust stable + aarch64-apple-darwin target
4. 缓存 Rust 依赖
5. 安装前端依赖 (`npm ci`)
6. 使用 `tauri-apps/tauri-action` 构建
7. 上传构建产物为 Artifact

#### Job 2: build-windows
1. 检出代码
2. 配置 Node.js 20.x
3. 配置 Rust stable
4. 缓存 Rust 依赖
5. 安装前端依赖 (`npm ci`)
6. 使用 `tauri-apps/tauri-action` 构建
7. 上传构建产物为 Artifact

#### Job 3: create-release
1. 等待 build-macos 和 build-windows 完成
2. 下载所有 Artifacts
3. 创建 GitHub Release
4. 上传所有安装包文件

---

## 5. 配置要求

### 5.1 GitHub Secrets（可选）

如果需要对应用进行签名，需要配置以下 Secrets：

| Secret 名称 | 描述 | 必需 |
|-------------|------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri 更新签名私钥 | 可选 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码 | 可选 |

> **注意**: `GITHUB_TOKEN` 由 GitHub Actions 自动提供，无需手动配置。

### 5.2 macOS 代码签名（可选）

如需对 macOS 应用进行公证 (Notarization)，需要额外配置：

| Secret 名称 | 描述 |
|-------------|------|
| `APPLE_CERTIFICATE` | Base64 编码的 .p12 证书 |
| `APPLE_CERTIFICATE_PASSWORD` | 证书密码 |
| `APPLE_SIGNING_IDENTITY` | 签名身份 |
| `APPLE_ID` | Apple ID 邮箱 |
| `APPLE_PASSWORD` | App 专用密码 |
| `APPLE_TEAM_ID` | Apple 开发者团队 ID |

---

## 6. 使用方法

### 6.1 发布新版本

```bash
# 1. 确保所有更改已提交
git add .
git commit -m "Release v1.0.0"

# 2. 创建并推送 tag
git tag v1.0.0
git push origin v1.0.0

# 或者一步完成
git push origin v1.0.0 --tags
```

### 6.2 查看构建状态
1. 进入 GitHub 仓库的 **Actions** 标签页
2. 查看 "Build and Release" workflow 的运行状态
3. 构建完成后，在 **Releases** 页面可以看到发布的版本

### 6.3 删除并重新发布

```bash
# 删除远程 tag
git push origin :refs/tags/v1.0.0

# 删除本地 tag
git tag -d v1.0.0

# 重新创建并推送
git tag v1.0.0
git push origin v1.0.0
```

---

## 7. 输出产物

### 7.1 产物清单

| 平台 | 文件名格式 | 说明 |
|------|-----------|------|
| macOS | `RoxyBrowser Manager_x.x.x_aarch64.dmg` | DMG 磁盘映像 |
| Windows | `RoxyBrowser Manager_x.x.x_x64-setup.exe` | NSIS 安装程序 |
| Windows | `RoxyBrowser Manager_x.x.x_x64_en-US.msi` | MSI 安装程序 |

### 7.2 Release 页面示例

发布后的 Release 页面将包含：
- 版本标题：`RoxyBrowser Manager v1.0.0`
- 更新日志/下载说明
- 所有平台的安装包下载链接

---

## 8. 常见问题

### Q1: 构建失败怎么办？
1. 检查 GitHub Actions 日志查看详细错误
2. 确保本地可以成功执行 `npm run tauri build`
3. 检查 `tauri.conf.json` 配置是否正确

### Q2: macOS 安装包无法打开？
- 用户需要在"系统偏好设置 > 安全性与隐私"中允许运行
- 如需避免此问题，需要进行 Apple 公证

### Q3: Windows 安装包报毒？
- 未签名的 Windows 应用可能被杀毒软件误报
- 如需避免，需要购买代码签名证书

### Q4: 如何添加 macOS Intel 版本？
在 workflow 中添加另一个 job，使用 `macos-13` runner 和 `x86_64-apple-darwin` target。

---

## 9. 版本历史

| 版本 | 日期 | 更改说明 |
|------|------|----------|
| 1.0.0 | 2026-01-27 | 初始版本，支持 macOS ARM64 和 Windows x64 |

---

## 10. 参考资料

- [Tauri GitHub Action](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 构建配置](https://tauri.app/v1/guides/building/)
- [语义化版本规范](https://semver.org/lang/zh-CN/)
