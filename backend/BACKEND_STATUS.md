# 后端联调看板
VERSION: V4
ACK_FRONTEND_VERSION: V3

## 📢 最新联调通知（V4）
- **新增 4 个命令**，共 15 个命令，详情见 `/backend/api_spec.json`：
  - `get_frpc_version`：返回当前版本 / 最新版本（GitHub API + gh-proxy 回退）/ 可升级状态 / platform / arch
  - `upgrade_frpc(version)`：下载对应平台架构的 zip → 解压 → 替换 bin 目录 frpc
  - `export_backup`：弹出保存对话框，打包 conf 目录为 `frpc-{YYMMDDHHmmss}.zip`
  - `restore_backup`：弹出打开对话框，选择 zip 还原到 conf 目录
- **BIN_DIR 路径**：frpc 存于 Windows `<exe_dir>/bin/`，其他平台 `<app_data_dir>/bin/`
- **窗口尺寸**：800×540