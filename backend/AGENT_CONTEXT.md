# 后端 Agent 上下文记录

> 此文件仅供后端 Agent 恢复上下文，记录所有未在其他文件中保存的关键信息。

---

## 项目概述
- **项目名称**: frpc-tray
- **技术栈**: Tauri 2.0 + Rust
- **目录结构**: 前端在 `/frontend`, 后端在 `/backend`
- **项目目标**: frpc 桌面端配置管理工具，用户将可执行文件放入 frpc 同级目录使用

---

## 当前开发状态 (截至 2026-06-25)

### 已完成
- ✅ TOML 配置文件管理功能 (9 个 Tauri 命令)
- ✅ 接口规范已写入 `/backend/api_spec.json`
- ✅ BACKEND_STATUS.md V1 已发布，等待前端 ACK

### 待办事项
- [ ] 等待前端对接 TOML 管理接口
- [ ] 根据前端反馈调整接口（如有）

### 下一步计划
- 等待前端 ACK V1 后，根据用户需求开发新功能

---

## 关键决策记录

### 2026-06-25
- **决策**: TOML 管理功能使用 `#[tauri::command]` 暴露给前端
- **原因**: 用户要求不改前端，只实现 Rust 方法
- **实现**: 9 个命令已注册到 `invoke_handler`

---

## 技术细节备忘

### 文件路径规则
- TOML 文件操作基于**可执行文件同级目录**
- 使用 `get_executable_dir()` 获取路径

### 数据结构
- `ProxyConfig`: id, name, url, enabled, icon?, color?
- 所有命令返回 `Result<T, String>` 格式

---

## 协作状态
- **当前版本**: V1
- **前端 ACK**: 未确认 (需检查 FRONTEND_STATUS.md)
- **阻塞点**: 无

---

## 下次启动检查清单
1. 读取 `FRONTEND_STATUS.md` 检查 ACK 状态
2. 如果前端已 ACK V1，可以发布新版本通知
3. 如果前端有反馈/bug 报告，优先处理
4. 如无新需求，继续等待或开发新功能
