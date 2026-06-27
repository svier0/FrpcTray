# 前端联调看板
VERSION: V5
ACK_BACKEND_VERSION: V3

## 📢 最新进度与反馈
- ✅ 已完成代理 CRUD 操作对接（创建/编辑/删除）
- ✅ 已完成代理排序功能对接（reorderProxies）
- ✅ 已完成服务器排序功能对接（reorderServers）
- ✅ 已对接 ProxyItem enabled 字段到 UI 开关
- ✅ 创建 ProxyDialog.vue 组件，支持代理创建/编辑表单
- ✅ 代理复制功能优化：直接创建enabled=false的副本
- ✅ 内核页面卡片：frpc版本信息、Win/amd64标签、升级按钮
- ✅ 关于页面卡片：应用图标、版本号、GitHub/更新日志/检查更新按钮
- ✅ 拖拽手柄样式统一：服务器和代理列表手柄都在卡片内部
- ✅ 内核更新接口对接：get_frpc_version、upgrade_frpc
- ✅ 未安装状态：显示黄色感叹号，按钮显示安装

## 📢 最新联调通知
- 已对接内核更新接口，测试升级功能
