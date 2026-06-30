# Bun 项目使用说明

## 项目结构
```
FrpcTray/
├── frontend/         # 前端代码 (Vue + TypeScript)
├── backend/          # 后端代码 (Tauri Rust)
└── ...
```

## 使用 Bun 的命令

### 在项目根目录安装依赖
```bash
bun install
```

### 在 frontend 目录运行开发服务器
```bash
cd frontend
bun run dev              # 启动前端开发服务器
bun run tauri dev       # 启动完整的 Tauri 开发服务器
```

### 构建项目
```bash
cd frontend
bun run build           # 构建前端代码
bun run tauri build     # 构建完整的 Tauri 应用
```

### 其他命令
```bash
cd frontend
bun run preview         # 预览构建结果
bun run tauri           # 显示 Tauri 帮助信息
```

## 注意事项
- 项目使用 **bun** 作为包管理器，**不要使用 npm**
- Tauri 配置文件位于 `backend/tauri.conf.json`
- 前端构建输出到 `frontend/dist` 目录
- 开发时前端运行在 `http://localhost:1420`