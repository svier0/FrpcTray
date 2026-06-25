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

### 待办事项
- [ ] 界面UI设计-设置界面-服务器-列表
- [ ] 界面UI设计-设置界面-服务器-新增
- [ ] 界面UI设计-设置界面-服务器-编辑
- [ ] 界面UI设计-设置界面-服务器-删除

### 下一步计划
- 继续完善界面UI设计
- 界面UI设计完成之前先不要看`BACKEND_STATUS.md`

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

---

## 协作状态
- **当前版本**: 无
- **后端 ACK**: 无
- **阻塞点**: 无

---

## 下次启动检查清单
1. 如无新需求，继续晚上UI设计

## 项目 Git 习惯
- 提交使用 `git add . && git commit -m "..."` 而非 `git add -A`
