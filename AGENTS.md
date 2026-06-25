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

## 协作 (Collaboration)
- See `/AI_COLLABORATION_GUIDE.md` for frontend-backend collaboration protocol
- Frontend writes to `FRONTEND_STATUS.md` only
- Backend writes to `BACKEND_STATUS.md` only
- API specs in `/backend/api_spec.json`

## 会话开始 (on session start)
- 不要一上来就扫描整个项目
- 阅读`BUN_USAGE.md`
- 后端阅读`backend/AGENT_CONTEXT.md`恢复上下文
- 前端阅读`frontend/AGENT_CONTEXT.md`恢复上下文
- 上下文恢复完成后再根据用户指示按需阅读代码