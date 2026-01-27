# RoxyBrowser 用户账户快速切换工具 - 可行性调研报告

## 1. 需求澄清

**问题描述**：在 macOS 上使用 RoxyBrowser 时，在不同的登录账户（如 `scaryhell@gmail.com` 和 `yangf1023@gmail.com`）之间切换需要退出重新登录，操作繁琐。

**期望目标**：开发一款工具，通过备份/替换本地存储文件，实现快速切换不同的 RoxyBrowser 登录用户。

---

## 2. 调研结论

### ✅ **可行性：高度可行**

通过分析 RoxyBrowser 的本地存储结构，发现用户认证数据存储在 `Local Storage/leveldb` 目录中，可以通过备份和替换相关文件实现用户切换。

---

## 3. RoxyBrowser 本地存储分析

### 3.1 数据目录结构

```
/Users/evan/Library/Application Support/RoxyBrowser/
├── config.json                    # 加密的配置文件
├── Local Storage/
│   └── leveldb/                   # 🔑 关键：包含用户认证数据
│       ├── 000005.ldb
│       ├── 000029.ldb
│       ├── 000030.log
│       ├── 000031.ldb
│       ├── CURRENT
│       ├── LOCK
│       ├── LOG
│       ├── MANIFEST-000001
│       └── ...
├── Cookies                        # Cookie 数据
├── Session Storage/               # 会话存储
├── IndexedDB/                     # 索引数据库
└── ...
```

### 3.2 关键发现

在 `Local Storage/leveldb` 的 `.ldb` 文件中发现了两个用户的认证数据：

#### 用户 1: scaryhell@gmail.com (当前登录)
```json
{
  "rememberMe": false,
  "password": ":MEekC85aR5964Po6:hpFoEUQzDNEicAIQibik0w==",
  "userInfo": {
    "email": "scaryhell@gmail.com"
  }
}
```

#### 用户 2: yangf1023@gmail.com (之前登录过)
```json
{
  "token": "",
  "rememberMe": false,
  "password": ":PiBpxjrjvSfTyyAd:52jHddN9hNxSgR0MzQJCTg==",
  "userInfo": {
    "email": "yangf1023@gmail.com"
  }
}
```

### 3.3 认证 Token 结构

每个用户还有一个 `lc_auth_token:组织ID` 键：
```json
{
  "accessToken": "us-south1:BdM7KXDITHe9b3BRbHSuWQ",
  "entityId": "1eaf75fa-7426-4ea9-b030-15a786a6f7c7",
  "expiresIn": 14400000,
  "tokenType": "Bearer",
  "creationDate": 1768898380583,
  "organizationId": "0c02efa3-3b53-4510-9591-4038fa91a308"
}
```

---

## 4. 切换方案

### 方案一：完整目录备份切换 (推荐)

**原理**：为每个用户备份完整的 RoxyBrowser 数据目录，切换时替换整个目录。

**需要备份的关键文件/目录**：

| 文件/目录 | 说明 | 重要性 |
|----------|------|--------|
| `config.json` | 加密的用户配置 | ⭐⭐⭐ 关键 |
| `Local Storage/` | 用户认证数据 | ⭐⭐⭐ 关键 |
| `Cookies` | Cookie 数据 | ⭐⭐ 重要 |
| `Session Storage/` | 会话数据 | ⭐ 可选 |
| `IndexedDB/` | 索引数据 | ⭐ 可选 |

**切换流程**：

```
1. 退出 RoxyBrowser
2. 备份当前用户数据到 ~/.roxy_profiles/scaryhell@gmail.com/
3. 恢复目标用户数据从 ~/.roxy_profiles/yangf1023@gmail.com/
4. 启动 RoxyBrowser
```

### 方案二：最小文件切换

**原理**：只备份和切换最关键的认证相关文件。

**最小切换文件集**：
- `config.json`
- `Local Storage/leveldb/*`

> [!WARNING]
> 此方案可能导致部分数据不一致，推荐使用方案一。

---

## 5. 技术实现方案

### 5.1 命令行工具设计

```bash
roxy-switch <action> [user_email]

# 示例
roxy-switch list              # 列出已保存的用户
roxy-switch save              # 保存当前用户配置
roxy-switch load yangf1023@gmail.com  # 切换到指定用户
roxy-switch backup            # 备份所有用户配置
```

### 5.2 核心逻辑伪代码

```python
ROXY_DATA_DIR = "~/Library/Application Support/RoxyBrowser"
PROFILES_DIR = "~/.roxy_profiles"

def save_current_profile(email):
    """保存当前用户配置"""
    if is_roxy_running():
        raise Error("请先退出 RoxyBrowser")
    
    profile_dir = f"{PROFILES_DIR}/{email}"
    files_to_backup = [
        "config.json",
        "Local Storage/",
        "Cookies",
        "Session Storage/",
    ]
    for f in files_to_backup:
        copy(f"{ROXY_DATA_DIR}/{f}", f"{profile_dir}/{f}")

def load_profile(email):
    """加载指定用户配置"""
    if is_roxy_running():
        raise Error("请先退出 RoxyBrowser")
    
    profile_dir = f"{PROFILES_DIR}/{email}"
    if not exists(profile_dir):
        raise Error(f"未找到用户配置: {email}")
    
    # 先保存当前配置
    current_email = get_current_email()
    save_current_profile(current_email)
    
    # 恢复目标用户配置
    for f in files_to_backup:
        copy(f"{profile_dir}/{f}", f"{ROXY_DATA_DIR}/{f}")
```

### 5.3 用户界面方案

**方案 A: 菜单栏应用** (macOS 原生体验)
- SwiftUI 开发
- 常驻菜单栏
- 下拉菜单快速切换

**方案 B: 命令行工具** (快速开发)
- Python/Shell 脚本
- 简单直接
- 可配合快捷键使用

---

## 6. 实施计划

### 阶段一：验证可行性 (手动测试)

1. 退出 RoxyBrowser
2. 备份当前数据目录：
   ```bash
   cp -r ~/Library/Application\ Support/RoxyBrowser ~/Desktop/roxy_backup_scaryhell
   ```
3. 删除 `Local Storage/leveldb/` 和 `config.json`
4. 重新登录 yangf1023@gmail.com
5. 验证数据是否正常

### 阶段二：开发命令行工具

1. 创建 Python 脚本
2. 实现 save/load/list 功能
3. 添加错误处理和日志

### 阶段三：开发 GUI 工具 (可选)

1. 使用 SwiftUI 或 Electron
2. 实现菜单栏应用
3. 添加快捷键支持

---

## 7. 风险与注意事项

| 风险 | 说明 | 缓解措施 |
|------|------|----------|
| 数据损坏 | 切换过程中可能损坏数据 | 始终保留完整备份 |
| 版本兼容 | RoxyBrowser 更新可能改变存储格式 | 版本检测和兼容处理 |
| 进程冲突 | RoxyBrowser 运行时切换会导致问题 | 检测进程状态，强制要求退出后切换 |
| Token 过期 | accessToken 有过期时间 (14400000ms ≈ 4小时) | 过期后需重新登录 |

> [!CAUTION]
> **重要提示**：切换用户前必须完全退出 RoxyBrowser，包括所有相关进程。

---

## 8. 结论与建议

### 可行性评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 技术可行性 | ⭐⭐⭐⭐⭐ | 数据结构清晰，替换方案可行 |
| 开发难度 | ⭐⭐ (低) | 核心是文件复制操作 |
| 使用体验 | ⭐⭐⭐⭐ | 需要退出应用，但切换快速 |

### 推荐下一步

> [!TIP]
> **建议先进行手动测试验证**：按照阶段一的步骤手动测试一次，确认方案可行后再开发自动化工具。

如果确认可行，我可以继续帮你：
1. 编写 Python 命令行工具
2. 或开发 macOS 菜单栏应用

---

## 9. 附录：快速手动测试命令

```bash
# 1. 确保 RoxyBrowser 已退出
pkill -f RoxyBrowser

# 2. 备份当前用户 (scaryhell@gmail.com)
mkdir -p ~/.roxy_profiles/scaryhell@gmail.com
cp ~/Library/Application\ Support/RoxyBrowser/config.json ~/.roxy_profiles/scaryhell@gmail.com/
cp -r ~/Library/Application\ Support/RoxyBrowser/Local\ Storage ~/.roxy_profiles/scaryhell@gmail.com/
cp ~/Library/Application\ Support/RoxyBrowser/Cookies ~/.roxy_profiles/scaryhell@gmail.com/

# 3. 切换到另一个用户时，恢复备份
# cp -r ~/.roxy_profiles/yangf1023@gmail.com/* ~/Library/Application\ Support/RoxyBrowser/

# 4. 启动 RoxyBrowser
open -a RoxyBrowser
```
