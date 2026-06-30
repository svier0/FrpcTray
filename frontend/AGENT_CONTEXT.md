# 前端 Agent 上下文记录

> 此文件仅供前端 Agent 恢复上下文，记录所有未在其他文件中保存的关键信息。

---

## 项目概述
- **项目名称**: FrpcTray
- **前端技术栈** Vue 3, TypeScript, Tailwind CSS v4, Radix Vue, vue-i18n
- **后端技术栈** Rust, Tauri 2
- **包管理器** bun (not npm)
- **目录结构**: 前端在 `/frontend`, 后端在 `/backend`
- **项目目标**: frpc 桌面端配置管理工具，用户将可执行文件放入 frpc 同级目录使用

## 主要约定 (Key Conventions)
- Language: Use Chinese (zh-CN) for UI text and comments
- Styling: dark theme, glass morphism
- State: 配置统一存储在后端 conf/config.toml（通过 get_config/save_config 接口）
- UI组件: 使用radix-vue+tailwindcss进行组件设计
- 应用名称: FrpcTray

---

## 当前开发状态 (截至 2026-06-29)

### 已完成
- ✅ 主界面：标题栏、总开关（默认关闭）、设置按钮、服务器tab
- ✅ 服务器列表：CRUD、拖拽排序、重命名（无id图标，显示服务器名称）
- ✅ 代理列表：CRUD、拖拽排序、enabled开关、type蓝色tag（无图标、无副标题）
- ✅ ProxyDialog：创建/编辑代理表单，locations仅http类型显示
- ✅ 设置-通用：语言、主题、主页面显示、开机自启、静默启动、自动运行
- ✅ 设置-内核：frpc版本卡片、安装/升级、状态指示器、GitHub代理开关
- ✅ 设置-关于：应用信息卡片
- ✅ 设置-高级：备份与恢复（export_backup/restore_backup）
- ✅ IPC接口：get_config, save_config, get_frpc_version, upgrade_frpc, export_backup, restore_backup
- ✅ 服务器运行管理：start_frpc, stop_frpc, start_all_frpc, stop_all_frpc, get_all_frpc_status
- ✅ frpc-status-changed 事件监听，实时状态更新
- ✅ 服务器错误信息显示：启动失败后在状态后显示错误原因，重启时清空
- ✅ 错误码 i18n 翻译（zh-CN/zh-TW/en/ja），支持命名占位符变量提取
- ✅ translateError 函数：正则模式匹配 + vue-i18n 翻译
- ✅ 控制台调试工具：`cmd('命令名', {参数})` 透传 invoke
- ✅ GitHub代理开关配置（useGithubProxy，默认false）
- ✅ 后端V13对接：connecting状态、openLogFile
- ✅ frp 官方 SVG logo（public/logo.svg）
- ✅ 关于页面图标更新为 frp 官方 logo
- ✅ TypeScript 类型修复（AppHeader ServerStatus 添加 connecting）

### 待办事项
- [ ] 设置界面-高级-其他设置

---

## 关键决策记录

### 2026-06-29
- **决策**: 关于页面使用 frp 官方 SVG logo
- **原因**: 统一品牌标识，从 gofrp.org 提取官方图标
- **实现**: public/logo.svg 存放图标，SettingsPage.vue 使用 img 标签引用

- **决策**: AppHeader ServerStatus 添加 connecting 类型
- **原因**: TypeScript 类型不匹配导致构建失败
- **实现**: AppHeader.vue ServerStatus 类型从 "idle" | "running" | "error" 改为 "idle" | "connecting" | "running" | "error"

### 2026-06-28
- **决策**: 服务器错误信息在"已停止"状态后显示，不改变状态为"异常"
- **原因**: 后端发 new_status: "stopped" + error_message，状态应显示"已停止"
- **实现**: hasError 不影响状态判断，serverError 单独存储并在状态后显示

- **决策**: 错误翻译绕过 vue-i18n 的引号key问题
- **原因**: vue-i18n 对含引号的 JSON key（如 `Unknown config field "{name}"`）解析失败
- **实现**: i18n 改用简单 key（如 error.unknownConfigField），正则提取变量后传给 t()

- **决策**: 控制台调试用 `cmd()` 透传 invoke
- **原因**: 用户需要快速调用任意后端接口调试
- **实现**: `(window as any).cmd = (name, args) => invoke(name, args)`

- **决策**: GitHub代理开关放内核卡片下方，和通用配置一起调 save_config
- **原因**: 下载frpc时可选通过代理加速
- **实现**: SettingsPage.vue 添加 SwitchRoot 开关，config.ts 添加 useGithubProxy 字段

- **决策**: locations 字段仅 http 类型显示，https 不显示
- **原因**: https 类型没有 locations 字段
- **实现**: ProxyDialog.vue v-if 改为 `formData.type === 'http'`

- **决策**: connecting 状态显示脉冲动画，按钮置灰
- **原因**: 后端新增 connecting 状态（启动后检测登录前），需明确视觉反馈
- **实现**: App.vue 状态圆点 bg-blue-500 + animate-pulse，按钮 disabled

- **决策**: showFrpcConsole 不对接，是后端调试用的功能
- **原因**: 调试开关属于运行时配置，和 autoRun 同级
- **实现**: SettingsPage.vue 添加 SwitchRoot 开关

---

## 技术细节备忘

### 配置管理
- 统一配置结构（存储在 conf/config.toml）：
  ```json
  {
    "language": "zh-CN",
    "theme": "system",
    "autostart": false,
    "silent_launch": false,
    "auto_run": false,
    "use_github_proxy": false
  }
  ```
- 前端 config.ts 负责前后端字段映射（camelCase ↔ snake_case）

### 错误翻译
- i18n key 使用简单名称（如 error.unknownConfigField）
- 正则模式匹配提取变量，替换占位符 {name}/{port}/{detail}/{reason}
- 错误码列表见 backend/api_spec.json → events[0].errorCodes.values

### 服务器状态
- 事件监听 frpc-status-changed，payload 包含 server_id, old_status, new_status, error_message
- 状态：running / connecting / idle / error
- connecting：启动后检测到登录前，圆点脉冲动画，按钮置灰
- 错误信息：有 error_message 时显示在"已停止"后面，启动时清空

### 静态资源
- frp 官方 SVG logo：public/logo.svg，用于关于页面

---

## 协作状态
- **前端版本**: V9
- **后端确认**: ACK_BACKEND_VERSION: V16
- **阻塞点**: 无

---

## 下次启动检查清单
1. 检查 GitHub 代理开关是否正常工作
2. 检查错误信息显示和翻译
3. 检查 connecting 状态显示
4. 检查日志按钮打开日志文件
5. 设置界面-高级-其他设置
