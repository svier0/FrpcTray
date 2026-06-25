# AI Agent 前后端跨堆栈协作协议 (AI_COLLABORATION_GUIDE)

## 1. 核心职责与目录隔离 (Core Isolation)
为了彻底避免 Git 代码冲突，两位 Agent 必须严格在自己的领地工作，严禁修改对方的代码目录和状态文件：
* **后端 Agent (OpenCode)：** 主战场在 `/backend`。对 `BACKEND_STATUS.md` 拥有唯一写入权；对 `FRONTEND_STATUS.md` 只读。
* **前端 Agent (Claude)：** 主战场在 `/frontend`。对 `FRONTEND_STATUS.md` 拥有唯一写入权；对 `BACKEND_STATUS.md` 只读。

---

## 2. 接口文档与动态通知分离原则 (Crucial Principle)
* **长久资产（不准删除）：** 具体的 API 路由、请求参数、返回格式等技术细节，后端必须记录在 `/backend/api_spec.json`（或 Swagger/Markdown 文档）中。该文件随开发持续追加，作为前端开发的唯一事实来源。
* **即时通知（覆盖更新）：** 根目录下的两个 `.md` 状态文件**仅作为“联调看板”**，只写一句话动态。必须控制在 15 行以内，看完即覆盖。

---

## 3. 看板格式规范 (Status File Format)

### == BACKEND_STATUS.md (仅允许后端写入，控制在30行内) ==
# 后端联调看板
VERSION: V1

## 📢 最新联调通知
- [例如：我写好了登录和注册接口，接口详情已更新至 /backend/api_spec.json，请前端开始对接。]


### == FRONTEND_STATUS.md (仅允许前端写入，控制在30行内) ==
# 前端联调看板
ACK_BACKEND_VERSION: V1

## 📢 最新进度与反馈
- [例如：收到 V1 通知，已根据 api_spec.json 对接完登录页。发现密码错误时后端返回了 500，请修复。]

---

## 4. 异步交棒工作流 (Asynchronous Workflow)

为了保证后端可以无限疯狂推进开发，同时不让离线的前端漏掉通知，必须严格执行以下逻辑：

### 🤖 后端 Agent 执行逻辑：
1.  **业务开发（绝对自由）：** 你可以连续、无限制地开发后端业务代码，并将所有新接口的详细定义同步写进 `/backend/api_spec.json`。你的代码开发不受前端进度影响。
2.  **更新看板（严格卡关）：** 当你准备更新 `BACKEND_STATUS.md` 喊话前端时，检查前端的 `ACK_BACKEND_VERSION`。
    * **如果相等：** 说明前端已经处理完你上一版的通知。请**清空并覆盖** `BACKEND_STATUS.md`，发布新的 `VERSION: V[下一数字]`，一口气写明你最近积压的所有新功能动态。
    * **如果不相等：** 说明前端还没上线，或者还在处理你上一版的通知。**严禁修改 BACKEND_STATUS.md**。请继续去写你的业务代码，直到前端上线签收。

### 🤖 前端 Agent 执行逻辑：
1.  **看板对齐：** 每次启动或写完一个阶段，首先读取 `BACKEND_STATUS.md`。
2.  **接口查阅：** 如果发现后端的 `VERSION` 大于你的 `ACK_BACKEND_VERSION`，说明后端有新动作。直接去阅读 `/backend/api_spec.json` 查阅最新的接口技术细节。
3.  **业务联调：** 按照顺序，逐步对接后端更新的功能。
4.  **签字放行：** 当你把后端某一版通知里的功能对接完毕后，**清空并覆盖** `FRONTEND_STATUS.md`，将 `ACK_BACKEND_VERSION` 修改为已对齐的后端版本号。这等于通知后端：“我已经追上你的进度了，你可以刷新看板了”。

---

## 5. 死锁仲裁 (Human Escalation)
如果前端在某一版本卡住（遇到 Bug），在 `FRONTEND_STATUS.md` 提出了报错反馈。后端在未修复该 Bug 并获得前端新一轮 ACK 之前，不得强推看板版本。若双方逻辑产生冲突无法推进，立即停止自动化，向人类 Owner 汇报。