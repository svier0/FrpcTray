# AGENTS.md

## Project Overview
frpc tray - A system tray application for managing frpc proxy configurations, built with Tauri 2 + Vue 3.

## Tech Stack
- **Frontend:** Vue 3, TypeScript, Tailwind CSS v4, Radix Vue, vue-i18n, vue-draggable-plus
- **Backend:** Rust, Tauri 2
- **Package Manager:** bun (not npm)

## Directory Structure
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

## Commands
```bash
# Frontend
cd frontend && bun run dev      # Start Vite dev server
cd frontend && bun run build    # Build frontend
cd frontend && bun run tauri dev    # Start Tauri dev
cd frontend && bun run tauri build  # Build Tauri app
```

## Key Conventions
- Language: Use Chinese (zh-CN) for UI text and comments
- Styling: Follow CC Switch design patterns (dark theme, glass morphism)
- State: Theme and language persisted in localStorage

## Collaboration
- See `/AI_COLLABORATION_GUIDE.md` for frontend-backend collaboration protocol
- Frontend writes to `FRONTEND_STATUS.md` only
- Backend writes to `BACKEND_STATUS.md` only
- API specs in `/backend/api_spec.json`
