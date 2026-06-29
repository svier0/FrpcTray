# FrpcTray

frpc 图形化管理工具，运行在系统托盘。支持多服务器、多代理规则管理，一键启动/停止 frpc 进程。

## 功能

- **多服务器管理** — 添加多个 frp 服务端连接，每个服务器独立配置代理规则
- **代理管理** — 增删改查、拖拽排序、一键开关，支持 TCP/UDP/HTTP/HTTPS/STCP/SUDP/XTCP
- **进程管理** — 单台/全部启动/停止 frpc，可设置开机自动运行
- **状态监控** — 实时显示连接状态、运行状态、错误信息，支持查看 frpc 日志
- **自动升级** — 检测 frpc 新版本，一键下载更新（支持 GitHub 代理加速）
- **备份恢复** — 导出/导入全部配置为 ZIP 文件
- **国际化** — 简体中文、繁体中文、英文、日文
- **主题切换** — 浅色/深色/跟随系统

## 安装

下载 [Releases](https://github.com/svier0/FrpcTray/releases) 中的安装包运行即可。首次使用时在"内核"页面下载 frpc 程序。

## 使用

1. 启动 FrpcTray（驻留在系统托盘）
2. 添加服务器（地址、端口、认证信息）
3. 添加代理规则
4. 点击启动按钮运行 frpc
5. 关闭窗口自动最小化到托盘

## 开发

```bash
cd frontend
bun install
bun run tauri dev    # 开发模式
bun run tauri build  # 构建安装包
```

## License

MIT
