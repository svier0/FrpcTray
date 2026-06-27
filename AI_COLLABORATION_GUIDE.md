# AI Agent 前后端跨堆栈协作协议 (AI_COLLABORATION_GUIDE)

## 1. 核心职责与目录隔离 (Core Isolation)
为了彻底避免 Git 代码冲突，两位 Agent 必须严格在自己的领地工作，严禁修改对方的代码目录和状态文件：
* **后端 Agent ：** 主战场在 `/backend`。对 `/backend/BACKEND_STATUS.md` 拥有唯一写入权；对 `/frontend/FRONTEND_STATUS.md` 只读；对 `/frontend/*` 只读。
* **前端 Agent ：** 主战场在 `/frontend`。对 `/frontend/FRONTEND_STATUS.md` 拥有唯一写入权；对 `/backend/BACKEND_STATUS.md` 只读；对 `/backend/*` 只读。

---

## 2. 接口文档与动态通知分离原则 (Crucial Principle)
* **长久资产（不准删除）：** 具体的 API 路由、请求参数、返回格式等技术细节，后端必须记录在 `/backend/api_spec.json`（或 Swagger/Markdown 文档）中。该文件随开发持续追加，作为前端开发的唯一事实来源。
* **即时通知（覆盖更新）：** `/frontend/FRONTEND_STATUS.md`和`/backend/BACKEND_STATUS.md`状态文件**仅作为“联调看板”**，只写一句话动态。必须控制在 30 行以内，看完即覆盖(要确定对方看过)。

---

## 3. 看板格式规范 (Status File Format)

### == BACKEND_STATUS.md (仅允许后端写入，控制在30行内) ==
```
# 后端联调看板
BACKEND_VERSION: V1
ACK_FRONTEND_VERSION: V1

## 📢 最新联调通知
- [例如：我写好了登录和注册接口，接口详情已更新至 /backend/api_spec.json，请前端开始对接。]
```

### == FRONTEND_STATUS.md (仅允许前端写入，控制在30行内) ==
```
# 前端联调看板
FRONTEND_VERSION: V1
ACK_BACKEND_VERSION: V1

## 📢 最新进度与反馈
- [例如：收到 V1 通知，已根据 api_spec.json 对接完登录页。发现密码错误时后端返回了 500，请修复。]
```

### 格式描述
FRONTEND_VERSION: 前端面板版本
BACKEND_VERSION: 后端面板版本
ACK_FRONTEND_VERSION: V1 表示已经看过前端V1版本的面板内容
ACK_BACKEND_VERSION: V1 表示已经看过后端V1版本的面板内容

---

## 4. 异步交棒工作流 (Asynchronous Workflow)

为了保证可以无限推进开发，同时不让离线的其它端agent漏掉通知，必须严格执行以下逻辑：

### 查看协作留言：
1. 读取`/frontend/FRONTEND_STATUS.md`和`/backend/BACKEND_STATUS.md`的FRONTEND_VERSION、BACKEND_VERSION、ACK_BACKEND_VERSION、ACK_FRONTEND_VERSION。
2. FRONTEND_VERSION大于ACK_FRONTEND_VERSION: 表示前端Agent有新留言给后端Agent
3. BACKEND_VERSION大于ACK_BACKEND_VERSION: 表示后端Agent有新留言给前端Agent
4. 如果对方没有给自己留言，则忽略`/frontend/FRONTEND_STATUS.md`和`/backend/BACKEND_STATUS.md`
5. 如果对方有新的留言，需要处理对方的反馈
6. 每次对话都需要查看协作留言

### 给协作Agent留言：
1. 首先查看对方阅读状态
2. ACK_FRONTEND_VERSION等于FRONTEND_VERSION: 表示后端看过前端的留言了
3. ACK_BACKEND_VERSION等于BACKEND_VERSION: 表示前端看过后端的留言了
4. 如果对方看过自己上次的留言了，需要先清空上次的留言信息,覆盖面板内容时需要迭代版本
    - 前端更新`/frontend/FRONTEND_STATUS.md`前，如果ACK_FRONTEND_VERSION等于FRONTEND_VERSION，发布新的`FRONTEND_VERSION: V[下一数字]`，覆盖更新。
    - 后端更新`/backend/BACKEND_STATUS.md`前，如果ACK_BACKEND_VERSION等于BACKEND_VERSION，发布新的`BACKEND_VERSION: V[下一数字]`，覆盖更新。
5. 如果对方没有看过自己上次的留言，保持版本不变，增量更新
    - 前端更新`/frontend/FRONTEND_STATUS.md`前，如果ACK_FRONTEND_VERSION小于FRONTEND_VERSION，增量更新。
    - 后端更新`/backend/BACKEND_STATUS.md`前，如果ACK_BACKEND_VERSION小于BACKEND_VERSION，增量更新。

### 🤖 后端 Agent 执行逻辑：
1.  **业务开发（绝对自由）：** 你可以连续、无限制地开发后端业务代码，并将所有新接口的详细定义同步写进 `/backend/api_spec.json`。你的代码开发不受前端进度影响。
2.  **看板对齐：** **每次**启动或写完一个阶段，FRONTEND_VERSION大于ACK_FRONTEND_VERSION时，处理`/frontend/FRONTEND_STATUS.md`中前端留言内容，并设置ACK_FRONTEND_VERSION=FRONTEND_VERSION。
3.  **更新看板（严格卡关）：** 当你准备更新 `/frontend/BACKEND_STATUS.md` 喊话前端时，检查前端的 `ACK_BACKEND_VERSION`和自己的BACKEND_VERSION。
    * **如果相等：** 说明前端已经处理完你上一版的通知。请**清空并覆盖** `BACKEND_STATUS.md`，发布新的 `BACKEND_VERSION: V[下一数字]`，一口气写明你最近积压的所有新功能动态。
    * **如果不相等：** 说明前端还没上线，或者还在处理你上一版的通知。**增量更新 BACKEND_STATUS.md**。待前端上线签收一并处理。
4.  **业务联调：** 按照顺序，逐步完成前端需要的接口。

### 🤖 前端 Agent 执行逻辑：
1.  **业务开发（绝对自由）：** 你可以连续、无限制地进行前端UI代码的设计开发。你的UI开发不受后端进度影响。
2.  **看板对齐：** **每次**每次启动或写完一个阶段，BACKEND_VERSION大于ACK_BACKEND_VERSION时，处理`/backend/BACKEND_STATUS.md`中后端留言内容。直接去阅读 `/backend/api_spec.json` 查阅最新的接口技术细节。并设置ACK_BACKEND_VERSION=BACKEND_VERSION。
3.  **业务联调：** 按照顺序，逐步对接后端更新的功能。
4.  **签字放行：** 当你把后端某一版通知里的功能对接完毕后，**清空并覆盖** `FRONTEND_STATUS.md`，将 `ACK_BACKEND_VERSION` 修改为已对齐的后端版本号。这等于通知后端：“我已经追上你的进度了，你可以刷新看板了”。

### **每次启动或写完一个阶段** 都必须进行看板对齐

---

## 5. 死锁仲裁 (Human Escalation)
如果前端在某一版本卡住（遇到 Bug），在 `FRONTEND_STATUS.md` 提出了报错反馈。后端在未修复该 Bug 并获得前端新一轮 ACK 之前，不得强推看板版本。若双方逻辑产生冲突无法推进，立即停止自动化，向人类 Owner 汇报。

## 6. 沟通真实性 (Authenticity)
绝对不要虚构沟通结果
例如：
```
# 前端联调看板
VERSION: V1
ACK_BACKEND_VERSION: V1
```
```
# 后端联调看板
VERSION: V2
ACK_FRONTEND_VERSION: V2
```
后端更新看板为V2时，直接顺带更新ACK_FRONTEND_VERSION为V2(表示已经看过前端的V2看板)，实际前端看板最新版本为V1，这将造成版本混乱。
