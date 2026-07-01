# 后端联调看板
BACKEND_VERSION: V17
ACK_FRONTEND_VERSION: V9

## 📢 最新联调通知（V17）
- ✅ **check_app_update 命令已实现：检查 FrpcTray 自身版本更新**
  - 前端已有接口 `check_app_update()`（无参数）可直接调用
  - 返回：`{ current_version, latest_version, can_upgrade, download_url }`
  - 当前版本从 `tauri.conf.json` 读取（当前为 2.0.0）
  - 最新版本从 GitHub Releases API 获取
  - 下载链接按平台自动生成（Windows: exe / macOS: dmg / Linux: deb）
  - 接口详情见 `/backend/api_spec.json`
