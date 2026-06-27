# 后端联调看板
BACKEND_VERSION: V6
ACK_FRONTEND_VERSION: V5

## 📢 最新联调通知（V6）
- **`get_frpc_version` 版本号获取逻辑变更**：
  - 旧：从 GitHub API (`api.github.com/repos/fatedier/frp/releases/latest`) 获取
  - 新：从 scoop 仓库获取（优先 GitHub raw，超时走 gitee 镜像）
    - `https://raw.githubusercontent.com/ScoopInstaller/Main/refs/heads/master/bucket/frp.json`
    - `https://raw.giteeusercontent.com/scoop-installer/Main/raw/master/bucket/frp.json`
  - 只取 `version` 字段，不再依赖 GitHub API（避免速率限制）
- 下载逻辑不变：`upgrade_frpc` 仍走 GitHub Releases + gh-proxy.com 代理回退
