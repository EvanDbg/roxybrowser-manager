# RoxyBrowser 快速切换用户工具 - 可行性调研报告

## 1. 背景与需求

**问题描述**：在 macOS 上使用 RoxyBrowser 时，每次切换账户都需要退出并重新登录，操作繁琐，影响工作效率。

**期望目标**：开发一款工具，可以快速切换 RoxyBrowser 的用户/浏览器配置文件，无需退出重新登录。

---

## 2. 调研结论

### ✅ **可行性：高度可行**

RoxyBrowser 提供了完善的本地 API 接口，完全支持开发第三方工具来实现快速切换用户配置文件的功能。

---

## 3. RoxyBrowser 技术能力分析

### 3.1 本地 API 支持

RoxyBrowser 提供了基于 HTTP 的本地 API 服务：

| 特性 | 说明 |
|------|------|
| **API 端点** | `http://127.0.0.1:50000` (默认端口) |
| **认证方式** | API Key (在软件内 API 配置中获取) |
| **API 版本** | 当前支持 v3.6.9+ |

### 3.2 关键 API 接口

以下是实现快速切换功能所需的核心 API：

| API 接口 | 方法 | 功能 |
|----------|------|------|
| `/browser/workspace` | GET | 获取工作空间和项目列表 |
| `/browser/list_v3` | GET | 获取浏览器配置文件列表 |
| `/browser/open` | POST | 打开指定的浏览器配置文件 |
| `/browser/close` | POST | 关闭指定的浏览器配置文件 |
| `/browser/detail` | GET | 获取浏览器配置文件详情 |

### 3.3 API 响应示例

**打开浏览器配置文件 (`POST /browser/open`)**：

```json
{
  "workspaceId": 1,
  "dirId": "dc1e73d4dd954a3a8ca087d53d2e18ce",
  "args": ["--remote-allow-origins=*"]
}
```

**响应结果**：

```json
{
  "code": 0,
  "data": {
    "ws": "ws://127.0.0.1:52314/devtools/browser/...",
    "http": "127.0.0.1:52314",
    "pid": 1111,
    "windowName": "Profile Name"
  },
  "msg": "Success"
}
```

---

## 4. 可实现的功能

基于 RoxyBrowser API，可以开发以下功能：

### 4.1 核心功能

- ✅ **配置文件列表展示**：显示所有浏览器配置文件
- ✅ **一键启动配置文件**：快速打开选定的浏览器配置
- ✅ **一键关闭配置文件**：快速关闭当前运行的浏览器
- ✅ **快速切换**：关闭当前配置文件并立即打开另一个

### 4.2 高级功能

- ✅ **工作空间管理**：按项目/工作空间分组显示配置文件
- ✅ **收藏夹**：标记常用配置文件便于快速访问
- ✅ **快捷键支持**：为常用配置文件分配全局快捷键
- ✅ **状态监控**：实时显示哪些配置文件正在运行
- ✅ **批量操作**：同时打开/关闭多个配置文件

---

## 5. 技术方案建议

### 5.1 推荐技术栈

| 组件 | 推荐方案 | 理由 |
|------|----------|------|
| **开发语言** | Python / Swift | Python 快速开发，Swift 原生 macOS 体验 |
| **UI 框架** | SwiftUI / PyQt / Electron | 根据开发经验选择 |
| **HTTP 客户端** | requests (Python) / URLSession (Swift) | 调用 RoxyBrowser API |

### 5.2 架构设计

```
┌─────────────────────────────────────────────────┐
│              macOS 管理工具 UI                   │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────┐ │
│  │ 配置文件列表 │  │ 状态监控面板 │  │ 快捷操作 │ │
│  └─────────────┘  └─────────────┘  └──────────┘ │
├─────────────────────────────────────────────────┤
│              API 管理层 (HTTP Client)            │
├─────────────────────────────────────────────────┤
│         RoxyBrowser Local API (Port 50000)      │
└─────────────────────────────────────────────────┘
```

### 5.3 实现步骤

1. **环境准备**
   - 在 RoxyBrowser 中启用 API 功能
   - 获取 API Key 并配置端口

2. **核心功能开发**
   - 实现 API 调用封装类
   - 开发配置文件列表获取功能
   - 实现打开/关闭配置文件功能

3. **UI 开发**
   - 设计简洁的管理界面
   - 实现菜单栏图标（Menu Bar App）
   - 添加快捷键支持

4. **优化与完善**
   - 添加配置持久化
   - 实现自动刷新状态
   - 错误处理与提示

---

## 6. 替代方案

如果不想自行开发，也可以考虑以下方案：

### 6.1 使用 RoxyBrowser MCP Server

RoxyBrowser 官方提供了 MCP (Model Context Protocol) 服务器，可以：

- 与 AI 助手集成实现语音/文字控制
- 通过 Playwright/Selenium 脚本自动化
- GitHub 仓库：[roxybrowserlabs/roxybrowser-mcp-server](https://github.com/roxybrowserlabs/roxybrowser-mcp-server)

### 6.2 使用自动化脚本

可以编写简单的 Shell/Python 脚本通过 API 实现快速切换：

```python
import requests

API_HOST = "http://127.0.0.1:50000"
API_KEY = "your-api-key"

def switch_profile(workspace_id, profile_id):
    headers = {"token": API_KEY}
    # 关闭当前配置文件（如有）
    requests.post(f"{API_HOST}/browser/close", 
                  json={"dirId": profile_id}, 
                  headers=headers)
    # 打开新配置文件
    resp = requests.post(f"{API_HOST}/browser/open",
                         json={"workspaceId": workspace_id, "dirId": profile_id},
                         headers=headers)
    return resp.json()
```

---

## 7. 风险与注意事项

| 风险 | 说明 | 缓解措施 |
|------|------|----------|
| API 变更 | 未来版本可能修改 API | 使用版本兼容检查 |
| 端口冲突 | 50000 端口可能被占用 | 支持自定义端口配置 |
| 认证失效 | API Key 可能被重置 | 提供 Key 配置入口 |

---

## 8. 结论与建议

### 可行性评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 技术可行性 | ⭐⭐⭐⭐⭐ | API 完善，文档齐全 |
| 开发难度 | ⭐⭐ (低) | 核心功能简单直接 |
| 维护成本 | ⭐⭐ (低) | API 稳定，变更风险小 |

### 建议

> [!TIP]
> **推荐开发一个 macOS 菜单栏应用**，可以：
> - 常驻菜单栏，快速访问
> - 支持快捷键切换常用配置
> - 实时显示当前运行的配置文件

如果你决定开发此工具，我可以继续帮你：
1. 制定详细的实现计划
2. 编写核心代码
3. 设计 UI 界面

---

## 9. 参考资料

- [RoxyBrowser 官网](https://roxybrowser.com)
- [RoxyBrowser API 文档](https://faq.roxybrowser.com/api-documentation/api-reference.html)
- [RoxyBrowser GitHub Labs](https://github.com/RoxyBrowserLabs)
- [RoxyBrowser MCP Server](https://github.com/roxybrowserlabs/roxybrowser-mcp-server)
