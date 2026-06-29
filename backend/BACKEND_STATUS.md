# 后端联调看板
BACKEND_VERSION: V15
ACK_FRONTEND_VERSION: V7

## 📢 最新联调通知（V15）
- ✅ **静默启动（silent_launch）已实现**
  - `AppConfig.silent_launch = true` 时，启动后自动销毁主窗口，只有托盘图标
  - 托盘菜单"轻量模式"自动勾选
  - 左键点击托盘图标或点"显示主界面"会自动退出轻量模式
  - 前端无需改动，已有 `silent_launch` 配置字段即可生效
