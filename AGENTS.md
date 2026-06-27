# AGENTS.md

## 项目概述 (Project Overview)
frpc tray - A system tray application for managing frpc proxy configurations, built with Tauri 2 + Vue 3.

## 技术栈 (Tech Stack)
- **Frontend:** Vue 3, TypeScript, Tailwind CSS v4, Radix Vue, vue-i18n, vue-draggable-plus
- **Backend:** Rust, Tauri 2
- **Package Manager:** bun (not npm)

## 目录结构 (Directory Structure)
```
/
├── frontend/          # Vue 3 frontend
│   ├── src/
│   │   ├── components/
│   │   ├── locales/   # i18n: zh-CN, zh-TW, en, ja
│   │   └── ...
│   └── package.json
└── backend/           # Rust/Tauri backend
    ├── src/
    ├── tauri.conf.json
    └── Cargo.toml
```

## 命令 (Commands)
```bash
# Frontend
cd frontend && bun run dev      # Start Vite dev server
cd frontend && bun run build    # Build frontend
cd frontend && bun run tauri dev    # Start Tauri dev
cd frontend && bun run tauri build  # Build Tauri app
```

## 开发者角色 (Developer Role)
如果会话开始用户没有为你指定开发者角色(前端/后端)，则codex=前端，claude=后端，如果都不是则询问用户。

## 目录隔离 (Directory isolation) - 必须记住
- 前端角色，只能写`/frontend/`目录，禁止修改`/backend/`目录
- 后端角色，只能写`/backend/`目录，禁止修改`/frontend/`目录
- 前后端角色都禁止直接修改根目录的任何文件，根目录由用户维护

## 协作 (Collaboration) - 必须记住
- 前后端协作协议具体内容查看 `/AI_COLLABORATION_GUIDE.md`
- `/AI_COLLABORATION_GUIDE.md`协作协议的内容必须遵守
- Frontend writes to `frontend/*/*` only
- Backend writes to `backend/*/*` only
- 根目录文件`/*`由用户维护，禁止agent编辑
- API specs in `/backend/api_spec.json`
- 前端角色不要读写后端代码，后端角色不要读写前端代码
- 联调看板是前后端沟通的桥梁，请不要把与对方无关的内容写入

## 项目 Git 习惯
- 提交在项目目录(`backend`或`frontend`)使用 `git add . && git commit -m "..."` 而非 `git add -A`
- 每次修改完文件都执行git提交

## 会话开始 (on session start) - 必须执行

**第一步：在回复用户任何消息之前，必须先执行以下操作：**

1. 阅读 `BUN_USAGE.md`
2. 前端阅读 `frontend/AGENT_CONTEXT.md`，后端阅读 `backend/AGENT_CONTEXT.md`
3. 完成上下文恢复后，才可以回复用户

**注意：用户发送任何内容（包括空消息或占位符）都必须先完成上述步骤。**

## **重要记忆** - 必须永远记住
- `AGENT_CONTEXT.md`是你自己的上下文记忆，你需要自行维护
- 每次对话结束前，如果有文件修改，在提交git之前，先更新自己的`AGENT_CONTEXT.md`(注意增量更新，不随意删除)
- 当用户要求 更新上下文、更新记忆、保存上下文 时，更新自己的`AGENT_CONTEXT.md`
- 自己的待办工作写到自己的`AGENT_CONTEXT.md`，不要写到前后端联调看板里
- 所有md文件的新增，都应该询问用户
- 所有md文件的编辑，大部分都应该使用增量更新