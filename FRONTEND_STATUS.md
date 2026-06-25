# 前端联调看板
VERSION: V1
ACK_BACKEND_VERSION: V1

## 📢 最新进度与反馈
- 收到 V1 通知,相关接口与前端所需不符，请修改

## 📢 最新联调通知

### 前端需要一系列接口
- 服务器增删改查操作、排序操作(用于拖动排序)
- 服务器属性:"id"(字符串),标题"title",启用"enable"(true/false),排序序号"sort"(数字),地址"serverAddr"(ip),端口"serverPort",鉴权方式"auth.method",鉴权密钥"auth.token"
- 代理增删改查操作、排序操作(用于拖动排序)
- 代理属性:名称"name",描述"desc",类型"type,本地ip"localIP",本地端口"localPort",绑定域名"customDomains",路径"locations"

### 提示
> 服务器(server)对应frpc配置文件 conf/frpc.{id}.toml
> 代理(proxies)对应frpc配置文件中的[[proxies]]
> 示例：
  服务器id="w" 对应配置文件 conf/frpc.w.toml
  ```
title = "阿里云服务器"
enable = true
sort = 1
serverAddr = "20.205.243.166"
serverPort = 7000
auth.method = "token"
auth.token = "12345678"
log.to = "frpc.log"
log.level = "info"
log.maxDays = 3

# 本地宝塔面板 https
[[proxies]]
desc = "本地宝塔面板 https"
name = "com_example_bt_https"
type = "https"
localIP = "127.0.0.1"
localPort = 8888
customDomains = ["bt.example.com"]
locations = ["/"]

# 本地宝塔面板 http
[[proxies]]
desc = "本地宝塔面板 http"
name = "com_example_bt_http"
type = "http"
localIP = "127.0.0.1"
localPort = 8888
customDomains = ["bt.example.com"]
locations = ["/"]

# 宝塔 https
[[proxies]]
desc = "宝塔 https"
name = "com_example_*_https"
type = "https"
localIP = "127.0.0.1"
localPort = 443
customDomains = ["*.example.com"]
locations = ["/"]

# 宝塔 http
[[proxies]]
desc = "宝塔 http"
name = "com_example_*_http"
type = "http"
localIP = "127.0.0.1"
localPort = 80
customDomains = ["*.example.com"]
locations = ["/"]

  ```