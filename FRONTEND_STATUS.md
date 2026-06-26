# 前端联调看板
VERSION: V2
ACK_BACKEND_VERSION: V3

## 📢 最新进度与反馈
- ✅ `create_server` 已修改：参数不含 `id`，返回值改为自动生成的单字母 id，符合前端期望
- ✅ 前端已确认：代码已就绪，接口对接完成

## 📢 最新联调通知

### 前端需要一系列接口
- 服务器增删改查操作、排序操作(用于拖动排序)
- 服务器属性:"id"(字符串),标题"title",启用"enable"(true/false),排序序号"sort"(数字),地址"serverAddr"(ip),端口"serverPort",鉴权方式"auth.method",鉴权密钥"auth.token"
- 代理增删改查操作、排序操作(用于拖动排序)
- 代理属性:名称"name",描述"desc",类型"type",本地ip"localIP",本地端口"localPort",绑定域名"customDomains",路径"locations"

### 需要后端支持
- **ProxyItem 需要 enabled 字段**：代理项需要一个 enabled 字段来控制启用/禁用状态，用于在 UI 上显示开关。请在 ProxyItem 数据结构中添加 `enabled: bool` 字段。

### 提示
> 服务器(server)对应frpc配置文件 conf/frpc.{id}.toml
> 代理(proxies)对应frpc配置文件中的[[proxies]]
> 示例：
  服务器id="w" 对应配置文件 conf/frpc.w.toml
  文件内容请阅读/backend/target/debug/conf/frpc.w.toml作为示例