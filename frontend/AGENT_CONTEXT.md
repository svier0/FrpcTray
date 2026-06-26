# 前端 Agent 上下文记录

> 此文件仅供前端 Agent 恢复上下文，记录所有未在其他文件中保存的关键信息。

---

## 项目概述
- **项目名称**: frpc-tray
- **前端技术栈** Vue 3, TypeScript, Tailwind CSS v4, Radix Vue, vue-i18n, vue-draggable-plus
- **后端技术栈** Rust, Tauri 2
- **包管理器** bun (not npm)
- **目录结构**: 前端在 `/frontend`, 后端在 `/backend`
- **项目目标**: frpc 桌面端配置管理工具，用户将可执行文件放入 frpc 同级目录使用

## 主要约定 (Key Conventions)
- Language: Use Chinese (zh-CN) for UI text and comments
- Styling: Follow CC Switch design patterns (dark theme, glass morphism)
- State: Theme and language persisted in localStorage
- UI组件: 使用radix-vue和tailwindcss进行组件设计

---

## 当前开发状态 (截至 2026-06-26)

### 已完成
- ✅ 界面UI设计-主界面-标题栏
- ✅ 界面UI设计-主界面-总开关
- ✅ 界面UI设计-主界面-设置按钮-跳转设置页
- ✅ 界面UI设计-主界面-服务器tab-切换
- ✅ 界面UI设计-主界面-服务器配置项-列表
- ✅ 界面UI设计-主界面-服务器配置项-拖动排序
- ✅ 界面UI设计-主界面-服务器配置项-日志按钮
- ✅ 界面UI设计-主界面-服务器配置项-编辑按钮
- ✅ 界面UI设计-主界面-服务器配置项-复制按钮
- ✅ 界面UI设计-主界面-服务器配置项-删除按钮
- ✅ 界面UI设计-设置界面-通用-i18n语言切换
- ✅ 界面UI设计-设置界面-通用-外观主题切换
- ✅ 界面UI设计-设置界面-通用-主页面显示服务器选择
- ✅ 界面UI设计-设置界面-服务器-列表（仿CC Switch样式，可展开编辑）
- ✅ 界面UI设计-设置界面-服务器-新增
- ✅ 界面UI设计-设置界面-服务器-编辑
- ✅ 界面UI设计-设置界面-服务器-删除（带确认弹窗）
- ✅ 后端接口对接-启动时加载服务器列表
- ✅ 后端接口对接-删除/添加/编辑调用后端
- ✅ 后端接口对接-删除确认弹窗
- ✅ 后端接口对接-createServer返回id

### 待办事项
- [ ] 界面UI设计-设置界面-内核-列表
- [ ] 界面UI设计-设置界面-高级-列表
- [ ] 界面UI设计-设置界面-关于-列表
- [ ] 后端接口对接-服务器排序（拖拽排序）

### 下一步计划
- 继续完善界面UI设计（内核、高级、关于tab）

---

## 关键决策记录

### 2026-06-26
- **决策**: 主题和语言配置使用`localStorage`进行持久化
- **原因**: 先以UI设计为主，然后逐步对接后端
- **实现**: 暂时使用`localStorage`进行持久化存储

---

## 技术细节备忘

### UI风格
- 使用radix-vue和tailwindcss进行组件设计
- Styling: Follow CC Switch design patterns (dark theme, glass morphism)

### 服务器列表UI
- 使用ServerItem.vue组件：可展开编辑的卡片
- 使用ServerList.vue组件：支持拖拽排序的列表
- 表单布局：名称+保存删除按钮（第一行）、地址+端口（第二行）、鉴权方式+密钥（第三行）
- 新增服务器时鉴权方式默认填入"token"
- 最多支持26个服务器（id为a-z单个字母）

### 主页面显示设置
- 在设置-通用tab中，外观主题下方新增"主页面显示"区域
- 使用Switch组件（radix-vue）控制服务器在主页面的显示/隐藏
- 状态存储在localStorage的`serverVisibility`键中
- 默认所有服务器都显示（true）

---

## 协作状态
- **当前版本**: V2
- **后端 ACK**: V3
- **阻塞点**: 无

---

## 下次启动检查清单
1. 如无新需求，继续晚上UI设计
