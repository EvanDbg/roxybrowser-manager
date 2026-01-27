# RoxyBrowser Manager - Windows 适配开发计划

## 概述

将 macOS 版本的 RoxyBrowser Manager 移植到 Windows 平台。经代码分析发现，项目已包含 Windows 平台的条件编译代码（`#[cfg(target_os = "windows")]`），主要工作是**验证和调整**现有代码以适配 Windows 环境。

---

## 项目现状分析

### 已有的 Windows 支持代码

| 文件 | 内容 | 状态 |
|------|------|------|
| `process.rs` | RoxyBrowser 启动路径 | ⚠️ 需验证路径 |
| `profile.rs` | 数据目录路径 | ⚠️ 需验证路径 |
| `tauri.conf.json` | 打包配置 | ✅ 已配置 MSI/NSIS |

### 关键路径对照

| 项目 | macOS | Windows (预期) |
|------|-------|----------------|
| RoxyBrowser 数据目录 | `~/Library/Application Support/RoxyBrowser/` | `%LOCALAPPDATA%\RoxyBrowser\` |
| Manager 配置目录 | `~/.roxy_manager/` | `%USERPROFILE%\.roxy_manager\` |
| RoxyBrowser 可执行文件 | `/Applications/RoxyBrowser.app` | `C:\Program Files\RoxyBrowser\RoxyBrowser.exe` |

---

## 开发阶段

### 第一阶段：环境验证

**开发环境要求**：
- Node.js v20+
- Rust (通过 rustup 安装)
- Visual Studio Build Tools (C++ 工具链)
- WebView2 Runtime

**验证命令**：
```bash
node --version
rustc --version
cargo --version
```

---

### 第二阶段：代码验证与适配

#### `process.rs` - 启动路径

验证并更新 RoxyBrowser 启动路径：

```rust
#[cfg(target_os = "windows")]
{
    let paths = [
        r"C:\Program Files\RoxyBrowser\RoxyBrowser.exe",
        r"C:\Program Files (x86)\RoxyBrowser\RoxyBrowser.exe",
        // 可能需要添加其他路径
    ];
}
```

#### `profile.rs` - 数据目录

验证数据目录路径：

```rust
#[cfg(target_os = "windows")]
{
    dirs::data_local_dir()  // %LOCALAPPDATA%
        .unwrap()
        .join("RoxyBrowser")
}
```

---

### 第三阶段：构建与测试

```bash
# 安装依赖
pnpm install

# 开发模式测试
pnpm tauri dev

# 构建安装包
pnpm tauri build
```

---

## 测试清单

| 测试项 | 操作 | 预期结果 |
|--------|------|----------|
| 状态检测 | 启动应用 | 正确显示 RoxyBrowser 运行状态 |
| 启动功能 | 点击"启动" | RoxyBrowser 成功启动 |
| 停止功能 | 点击"停止" | RoxyBrowser 成功关闭 |
| 添加用户 | 完成添加向导 | 新用户出现在列表 |
| 切换用户 | 选择其他用户并切换 | 切换后 RoxyBrowser 加载正确账户 |
| 导出配置 | 点击导出 | 配置文件成功导出到指定目录 |
| 导入配置 | 点击导入 | 配置文件成功导入 |

---

## 预计工时

| 阶段 | 时间 |
|------|------|
| 环境验证与准备 | 0.5 小时 |
| 代码验证与适配 | 1-2 小时 |
| 构建与测试 | 1-2 小时 |
| 问题修复缓冲 | 1 小时 |
| **总计** | **3-5 小时** |
