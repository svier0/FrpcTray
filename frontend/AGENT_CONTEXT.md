# 前端 Agent 上下文记录

> 此文件仅供前端 Agent 恢复上下文，记录所有未在其他文件中保存的关键信息。

---

## 项目概述
- **项目名称**: frpc-tray
- **前端技术栈** Vue 3, TypeScript, Tailwind CSS v4, Radix Vue, vue-i18n
- **后端技术栈** Rust, Tauri 2
- **包管理器** bun (not npm)
- **目录结构**: 前端在 `/frontend`, 后端在 `/backend`
- **项目目标**: frpc 桌面端配置管理工具，用户将可执行文件放入 frpc 同级目录使用

## 主要约定 (Key Conventions)
- Language: Use Chinese (zh-CN) for UI text and comments
- Styling: Follow CC Switch design patterns (dark theme, glass morphism)
- State: 配置统一存储在后端 conf/config.toml（通过 get_config/save_config 接口）
- UI组件: 使用tailwindcss进行组件设计
- 应用名称: FrpC Tray

---

## 当前开发状态 (截至 2026-06-27)

### 已完成
- ✅ 界面UI设计-主界面-标题栏（FrpC Tray）
- ✅ 界面UI设计-主界面-总开关（默认关闭）
- ✅ 界面UI设计-主界面-设置按钮-跳转设置页
- ✅ 界面UI设计-主界面-服务器tab-切换（显示服务器名称）
- ✅ 界面UI设计-主界面-服务器配置项-列表
- ✅ 界面UI设计-主界面-服务器配置项-拖动排序
- ✅ 界面UI设计-主界面-服务器配置项-日志按钮
- ✅ 界面UI设计-主界面-服务器配置项-编辑按钮
- ✅ 界面UI设计-主界面-服务器配置项-复制按钮
- ✅ 界面UI设计-主界面-服务器配置项-删除按钮
- ✅ 界面UI设计-设置界面-通用-i18n语言切换
- ✅ 界面UI设计-设置界面-通用-外观主题切换
- ✅ 界面UI设计-设置界面-通用-主页面显示服务器选择
- ✅ 界面UI设计-设置界面-通用-开机自启
- ✅ 界面UI设计-设置界面-通用-静默启动
- ✅ 界面UI设计-设置界面-通用-自动运行
- ✅ 界面UI设计-设置界面-服务器-列表（仿CC Switch样式，可展开编辑）
- ✅ 界面UI设计-设置界面-服务器-新增
- ✅ 界面UI设计-设置界面-服务器-编辑
- ✅ 界面UI设计-设置界面-服务器-删除（带确认弹窗）
- ✅ 后端接口对接-启动时加载服务器列表
- ✅ 后端接口对接-删除/添加/编辑调用后端
- ✅ 后端接口对接-删除确认弹窗
- ✅ 后端接口对接-createServer返回id
- ✅ 后端接口对接-代理列表加载（listProxies）
- ✅ 界面UI设计-代理项显示（标题=desc，type蓝色tag）
- ✅ 后端接口对接-代理enabled开关（updateProxy）
- ✅ 后端接口对接-代理CRUD操作（createProxy, updateProxy, deleteProxy）
- ✅ 后端接口对接-代理排序（reorderProxies）
- ✅ 后端接口对接-服务器排序（reorderServers）
- ✅ 界面UI设计-代理项-ProxyItem组件（拖拽手柄、复制、编辑、删除按钮）
- ✅ 界面UI设计-代理项-ProxyDialog弹窗（创建/编辑代理）
- ✅ 界面UI设计-设置界面-内核-卡片（frpc版本信息、Win/amd64标签、升级按钮）
- ✅ 界面UI设计-设置界面-关于-卡片（应用图标、版本号、GitHub/更新日志/检查更新按钮）
- ✅ 后端接口对接-内核版本信息（get_frpc_version）
- ✅ 后端接口对接-内核升级（upgrade_frpc）
- ✅ 界面UI设计-设置界面-内核-未安装状态（黄色感叹号、安装按钮）
- ✅ 界面UI设计-设置界面-内核-升级状态（升级中/升级成功提示，3秒自动清除）
- ✅ 界面UI设计-设置界面-内核-已是最新状态（绿色对勾图标）
- ✅ 界面UI设计-设置界面-内核-最新版本获取失败（请重试按钮）
- ✅ 界面UI设计-设置界面-高级-备份与恢复卡片（可折叠，导出备份/选择ZIP备份文件按钮）
- ✅ 后端接口对接-备份恢复（export_backup, restore_backup）
- ✅ 后端接口对接-应用配置（get_config, save_config）

### 待办事项
- [ ] 界面UI设计-设置界面-高级-其他设置

---

## 关键决策记录

### 2026-06-26
- **决策**: 主题和语言配置使用`localStorage`进行持久化
- **原因**: 先以UI设计为主，然后逐步对接后端
- **实现**: 暂时使用`localStorage`进行持久化存储

### 2026-06-27
- **决策**: 代理复制按钮直接提交，不弹对话框
- **原因**: 复制逻辑应该和新增不同，直接创建enabled=false的副本
- **实现**: handleDuplicate直接调用createProxy，不打开ProxyDialog

- **决策**: 拖拽手柄使用SortableJS的forceFallback模式
- **原因**: Tauri webview拦截HTML5 Drag API，导致拖拽无法正常工作
- **实现**: SortableJS配置forceFallback:true，使用JS模拟拖拽

- **决策**: 服务器列表和代理列表的拖拽手柄样式统一
- **原因**: 之前服务器手柄在卡片外（absolute定位），代理手柄在卡片内
- **实现**: 将服务器手柄移入ServerItem组件内部，与ProxyItem布局一致

### 2026-06-27 (下午)
- **决策**: 内核版本卡片始终渲染，值用 `versionInfo?.field || '-'` 占位
- **原因**: 之前 `v-if="versionInfo"` 导致接口请求时整个卡片不渲染
- **实现**: 移除v-if，用可选链和fallback值占位

- **决策**: 未安装/已安装/升级 三种状态用不同颜色区分
- **原因**: 仿CC Switch设计风格，状态一目了然
- **实现**: 未安装→黄色感叹号，已是最新→绿色对勾，可升级→黄色感叹号+文字

- **决策**: 安装按钮用灰色边框，升级按钮用蓝色实心
- **原因**: 安装是新操作，升级是更新操作，视觉区分
- **实现**: 通过条件class动态切换按钮样式

- **决策**: 安装/升级成功后3秒自动清除提示
- **原因**: 成功提示不需要一直显示
- **实现**: setTimeout 3秒后清空 upgradeProgress

- **决策**: 安装/升级状态文案区分（安装中/升级中，安装成功/升级成功）
- **原因**: 用户体验更清晰
- **实现**: 根据 current_version === '0' 判断当前是安装还是升级

- **决策**: 服务器列表和通用设置移除id图标，改用服务器名称
- **原因**: 简化UI，服务器名称更直观
- **实现**: 移除id图标，AppHeader显示server.title

- **决策**: 代理项type改为蓝色tag显示，移除图标和副标题
- **原因**: 统一视觉风格，tag更清晰
- **实现**: 移除type图标，用蓝色背景tag显示type

- **决策**: 应用配置统一存储在后端 conf/config.toml
- **原因**: 前后端配置同步，支持跨设备
- **实现**: 使用 get_config/save_config 接口，config.ts 统一管理

- **决策**: 应用名称改为 FrpC Tray
- **原因**: 品牌统一
- **实现**: 修改 AppHeader 和 i18n 翻译

---

## 技术细节备忘

### UI风格
- 使用tailwindcss进行组件设计
- Styling: Follow CC Switch design patterns (dark theme, glass morphism)

### 服务器列表UI
- 使用ServerItem.vue组件：可展开编辑的卡片
- 使用ServerList.vue组件：支持拖拽排序的列表
- 表单布局：名称+保存删除按钮（第一行）、地址+端口（第二行）、鉴权方式+密钥（第三行）
- 新增服务器时鉴权方式默认填入"token"
- 最多支持26个服务器（id为a-z单个字母）

### 代理列表UI
- 使用ProxyItem.vue组件：显示代理信息
- 使用ProxyList.vue组件：支持拖拽排序的列表
- 使用ProxyDialog.vue组件：创建/编辑代理弹窗
- 标题显示desc字段，type用蓝色tag显示
- 代理复制：直接创建enabled=false的副本，不弹对话框

### 设置页面
- 通用页面：语言切换、主题切换、主页面显示服务器选择、开机自启、静默启动、自动运行
- 内核页面：frpc版本卡片（Win/amd64标签、当前版本、最新版本、升级按钮）
  - 未安装（current_version === '0'）：黄色感叹号、灰色安装按钮
  - 可升级：黄色感叹号+文字、蓝色升级按钮
  - 已是最新：绿色对勾图标
  - 最新版本获取失败：显示"请重试"按钮
  - 成功提示3秒自动清除
- 关于页面：应用信息卡片（图标、版本号、GitHub/更新日志/检查更新按钮）
- 高级页面：备份与恢复折叠卡片（橙色图标、导出备份/选择ZIP备份文件按钮）

### 配置管理
- 统一配置结构（存储在 conf/config.toml）：
  ```json
  {
    "language": "zh-CN",
    "theme": "system",
    "autostart": false,
    "silent_launch": false,
    "auto_run": false
  }
  ```
- 前端 config.ts 负责前后端字段映射（camelCase ↔ snake_case）
- 启动时通过 get_config 加载，修改时通过 save_config 保存

### 拖拽排序实现
- 使用SortableJS（非vue-draggable-plus）
- 配置forceFallback:true绕过Tauri webview限制
- onEnd事件中手动splice数组并emit新顺序
- 拖拽手柄使用.drag-handle类选择器

---

## 协作状态
- **当前版本**: V5
- **后端 ACK**: V7
- **阻塞点**: 无

---

## 下次启动检查清单
1. 如无新需求，继续完善高级tab其他设置
2. 检查代理拖拽排序是否正常工作
3. 检查服务器拖拽排序是否正常工作
4. 备份与恢复功能已对接完成
