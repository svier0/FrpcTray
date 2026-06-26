# 前端对后端的接口修改建议

## create_server 接口需要修改

**当前接口定义：**
```json
{
  "name": "create_server",
  "parameters": [
    { "name": "server", "type": "ServerInfo", "description": "服务器配置（需包含 id）" }
  ],
  "returns": {
    "type": "Result<(), String>",
    "success": "空",
    "error": "错误信息字符串"
  }
}
```

**建议修改为：**
```json
{
  "name": "create_server",
  "parameters": [
    { "name": "server", "type": "ServerInfo", "description": "服务器配置（无需包含 id，后端自动生成）" }
  ],
  "returns": {
    "type": "Result<String, String>",
    "success": "新增服务器的 id（单个字母 a-z）",
    "error": "错误信息字符串"
  }
}
```

**原因：**
1. 前端不知道磁盘上有哪些 id 已被占用（用户可能手动新建了配置文件）
2. 后端负责生成 id，检查是否重复，保证唯一性
3. 前端调用后根据返回的 id 更新 UI

**后端 id 生成逻辑：**
1. 扫描 `conf/frpc.*.toml` 获取已存在的 id
2. 从 a-z 中找到第一个未被使用的字母作为新 id
3. 如果 a-z 全部被占用，返回错误 `"Maximum 26 servers reached"`

**前端已修改：**
- `utils/ipc.ts` 中 `createServer` 参数改为 `Omit<ServerItem, "id">`，返回值改为 `Promise<string>`
- 前端调用 `createServer` 后用返回的 id 构建完整的 ServerItem 对象
