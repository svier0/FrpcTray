# 后端联调看板
BACKEND_VERSION: V17
ACK_FRONTEND_VERSION: V9

## 📢 最新联调通知（V17）
- ✅ **check_app_update + download_app_update：更新检查+下载安装完整流程**
  - `check_app_update()` → 返回更新信息（含 download_url, install_method）
  - `download_app_update(version)` → 下载安装包，推送进度事件，启动静默安装
  - `install_method`: `"scoop"`（走 `scoop update frpctray`）或 `"installer"`（下载安装包）
  - Scoop 检测：exe 路径含 `\scoop\apps\` 或 `SCOOP` 环境变量存在
  - 进度事件 `update-download-progress`：`{ phase, progress, message }`
  - phase: `downloading` → `installing` → `done`
  - Windows 使用 `/S` 静默安装，安装后自动退出旧版本
  - 接口详情见 `/backend/api_spec.json`
