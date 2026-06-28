# 后端联调看板
BACKEND_VERSION: V13
ACK_FRONTEND_VERSION: V6

## 📢 最新联调通知（V13 增量）
- **进程管理架构重构**：用 oneshot channel 替代 `Arc<Mutex<Child>>`，stop 不再阻塞
  - monitor 独占 child，`select!` 等进程退出或 kill 信号
  - stop 发信号即返回，不再抢锁
- **新增 AppConfig.show_frpc_console**（调试用）
  - `true`：启动 frpc 时显示独立命令行窗口，stdout/stderr 继承
  - `false`（默认）：后台运行，stdout/stderr 管道捕获
  - Windows 使用 `CREATE_NEW_CONSOLE` 创建新窗口
- **新增 AppConfig.use_github_proxy**（前端配置）
  - `true`：版本检查走 gitee 镜像，升级走 gh-proxy/ghfast.top
  - `false`（默认）：版本检查/升级都只直连 GitHub，不退回镜像
- **新增 connecting 状态**（V13 新增）
  - 启动 frpc 后先发 `connecting`（按钮置灰），检测到 `login to server success` 后发 `running`（按钮变停止）
  - `get_all_frpc_status` 也返回 `connecting`
  - `show_frpc_console` 模式跳过检测直接 `running`
  - `api_spec.json` 已更新状态字段
