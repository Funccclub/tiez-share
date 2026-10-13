# TieZ Share — 局域网文件中转 / 聊天室

从 [jimuzhe/tiez-clipboard](https://github.com/jimuzhe/tiez-clipboard) 里把「**局域网传输**」这一块单独摘出来，去掉 Tauri 桌面端依赖，做成一个能在 **iStoreOS / OpenWrt 旁路由**上用 Docker 跑的常驻服务。

效果：家里所有设备（手机、平板、电脑）用浏览器打开 `http://路由器IP:12345`，就进同一个聊天室 —— 发文字、发图片、发文件，所有在线设备实时收到，文件保存在路由器的外挂盘上。

---

## 1. 摘取范围

上游把局域网传输写成了一个自包含的 axum 服务，并且**自带一个完整的网页界面**（`web_ui.rs`，1097 行内嵌 HTML/CSS/JS，无任何 CDN 外链，断网可用）。这正是它能被干净地抽出来的原因。

| 上游文件 | 本项目的处理 |
| :--- | :--- |
| `services/file_transfer/mod.rs` | 服务编排 → `src/main.rs` + `src/handlers.rs` |
| `services/file_transfer/handlers.rs` | 全部路由原样移植，仅去掉 `AppHandle` |
| `services/file_transfer/models.rs` | `src/models.rs`（`Message` 的 JSON 结构完全一致） |
| `services/file_transfer/utils.rs` | `src/store.rs`（保留取 IP / 文件名逻辑思路） |
| `services/file_transfer/web_ui.rs` | `src/web_ui.rs`，**界面样式原样保留**，只改了消息归属判定 |
| Tauri `AppHandle` 状态、剪贴板写入、SQLite 剪贴板历史 | 全部删除（路由器上没有意义） |

接口路径与上游**逐字兼容**（只有最后两个 `/files` 接口是新增的），所以 TieZ 桌面端的「手机传输」客户端、以及原来的网页链接过来都能直接用：

```
GET  /                     网页聊天室
GET  /ws                   WebSocket 实时通道
GET  /poll?last_id=N       拉取历史消息
POST /send_text  /send-text
POST /upload               单次上传
POST /upload_chunk  /upload-chunk   分片上传
GET  /download/{token}     带 Range 支持的文件下载
GET  /files                已上传文件列表（左侧文件区用）   ← 新增
DELETE /files/{token}      删除一个已上传文件               ← 新增
GET  /healthz              健康检查
```

---

## 2. 相比上游修掉的问题

提取过程中发现 4 个问题，不修的话"当聊天室用"是不成立的：

1. **所有人的消息都显示在自己右侧。**
   上游 `/send_text` 一律把消息记成 `direction: "in"`，而网页端只要看到 `in` 就渲染成"我发的 You"。结果 A 发的消息在 B 的屏幕上也是右侧气泡，分不清谁是谁。
   → 改成按 `sender_id === 本机 deviceId` 判定归属，别人的消息正确显示在左侧并带上名字。

2. **新设备进来看不到历史。**
   网页端只监听 WebSocket，不拉历史，后进来的人只能看到自己之后的消息。
   → 页面加载时先 `GET /poll?last_id=0` 补齐历史，并用 id 去重，避免和实时推送重复。

3. **重启就丢聊天记录。**
   上游聊天记录只存在内存里，路由器一重启全没。
   → 新增 JSON Lines 追加式持久化（`history.jsonl`），启动时回读；下载链接的 token 改成由文件路径哈希得到，**重启后旧链接依然有效**。

4. **上传文件名可穿越目录。**
   上游直接把 multipart 的 `file_name` 拼进保存路径，`../../xxx` 这种文件名可以写到存储目录之外。
   → 统一走 `sanitize_filename()`：剥掉目录部分、去掉控制字符、限制长度；分片上传的 `upload_id` 同样做白名单过滤。

5. **四个主题的毛玻璃和发送按钮样式其实没生效**（上游同样存在，不是移植引入的）。
   主题 CSS 是拼进页面的，里面的花括号本该是单层，上游写成了 `{{ }}`，于是这些规则在浏览器里全是废的：`.theme-mica header/footer` 的毛玻璃、收到消息气泡的模糊、发送按钮的配色。本次默认用的就是 mica 主题，所以修完直接能看出区别。
   → 改成单层花括号；自检里新增一条：渲染出的页面不允许再出现 `{{`。

另外顺手加固/增强的：启动时探测存储目录可写（挂载盘没挂上会立刻报错退出而不是默默失败）、端口被占用直接明确报错、`/healthz` 健康检查、主题/标题/设备名可配置、在线人数显示、加入退出提示（可用 `TIEZ_JOIN_NOTICE=0` 关闭）；另外修了上传**失败时仍然打勾**的问题（失败现在显示「✗ 上传失败」），以及失败分支里 `pendingUploads.delete(name)` 的变量名笔误。

---

## 3. 构建镜像（ARM64）

镜像默认产出 **linux/arm64**。编译阶段在 x86_64 机器上用 aarch64-musl 交叉工具链，**不做 QEMU 全量模拟**，所以速度接近本地编译；运行阶段只拷贝文件、没有 `RUN`，也不需要 QEMU。

```bash
# 在你自己的电脑上（需 Docker Desktop / Docker Engine）
cd tiez-share
docker build --provenance=false --sbom=false -t tiez-share:arm64 .
```

> 那两个参数建议保留：关掉 attestation 清单后，`docker save` 出来的包里只有干净的
> `linux/arm64` 一个平台，直接用 `docker load` 导入到路由器上最不容易出错。

Windows PowerShell 用附带的脚本：

```powershell
.\build-image.ps1            # 构建 arm64 镜像
.\build-image.ps1 -Save      # 顺便导出成 tiez-share-arm64.tar，方便传到路由器
```

<details>
<summary>想先在电脑上跑起来验证（x86_64 测试镜像）</summary>

```bash
docker build \
  --build-arg MUSL_CROSS_IMAGE=messense/rust-musl-cross:x86_64-musl \
  --build-arg RUST_TARGET=x86_64-unknown-linux-musl \
  --build-arg RUNTIME_PLATFORM=linux/amd64 \
  -t tiez-share:amd64 .
docker run --rm -p 12345:12345 -v tiez-data:/data tiez-share:amd64
```
</details>

---

## 4. 部署到 iStoreOS 旁路由

### 4.1 准备外挂盘（重要）

`/data` 会堆积所有中转文件，**千万不要让它落在 overlay 系统分区上**，写满会把路由器搞坏。

```sh
# 路由器 SSH 里执行
df -h | grep mnt          # 确认 U 盘/硬盘已挂载，例如 /mnt/sda1
mkdir -p /mnt/sda1/tiez-share
```

> ⚠️ **别拿公用目录当存储目录。** 页面左侧的文件区会列出存储目录下的所有文件，
> 并且允许删除其中任何一个——要是直接把 `/mnt/sda/disk/others` 这类放着别的东西的目录
> 挂进去，等于把那些文件也一并暴露给局域网里每台设备（包括删除权限）。
> 一定留一层专属子目录，例如 `/mnt/sda/disk/others/tiez`。

### 4.2 把镜像弄到路由器上

推荐在电脑上构建后导出，路由器直接 load（最省事，也不用在路由器上装 Rust）：

```powershell
# 电脑上
docker save -o tiez-share-arm64.tar tiez-share:arm64
scp tiez-share-arm64.tar root@192.168.1.2:/root/
```

```sh
# 路由器上
docker load -i /root/tiez-share-arm64.tar
docker images | grep tiez-share     # 应该看到 tiez-share  arm64
```

> iStoreOS 的 **Docker → 镜像 → 导入** 图形界面也可以直接选这个 `.tar` 文件导入。

### 4.3 启动

把 `docker-compose.yml` 传到路由器（例如 `/root/tiez-share/`），记得确认里面的挂载路径是 `- /mnt/sda1/tiez-share:/data`：

```sh
cd /root/tiez-share
docker compose up -d
docker compose logs -f          # 看到 TieZ Share 的启动横幅就成功了
```

不想用 compose，一条命令也行：

```sh
docker run -d --name tiez-share --restart unless-stopped \
  -p 12345:12345 \
  -e TIEZ_TITLE="家庭局域网中转" \
  -v /mnt/sda1/tiez-share:/data \
  tiez-share:arm64
```

### 4.4 开始用

浏览器打开（手机、电脑都行）：

```
http://<旁路由的LAN地址>:12345
```

- 右上角显示的是**本机名字**，点一下可以改（会自动按手机/系统猜一个初值），也可以直接用 `http://路由器IP:12345/?name=客厅平板` 指定。
- 顶部圆点旁边显示**在线设备数**，鼠标悬停可以看到都有谁。
- 左边 `+` 发文件/图片，输入框发文字，回车右侧 `SEND` 发送。
- **把文件拖到页面任意位置就能上传**，拖拽时整页会出现虚线提示框。
- 左侧是**已上传文件区**，可以下载、删除，详见下一节。
- 所有设备共享同一份历史，重启路由也不丢。

### 4.5 左侧文件区

页面左侧列出存储目录里当前的所有文件，每个文件都能下载和删除：

- **点文件即下载**：走 `/download/{token}`，支持 Range 请求，所以浏览器自带的下载器、IDM 之类都能续传。
- **点 🗑 删除**：删掉服务器上的文件，**同时把聊天记录里引用该文件的那条消息一并移除**。

  这一点是刻意的：如果只删文件不删消息，聊天里会留下一个点不开的死链接，而且左侧文件区和聊天记录会对不上。删除不可恢复，会弹二次确认。
- **拖拽上传**：把文件拖到页面任意位置（不限于文件区）松手即上传；多选拖入会逐个上传，进度显示在聊天气泡里。

  有设备上传或删除时，其它在线设备的文件区会通过 WebSocket 自动同步，不用手动刷新。
- 面板顶部显示「文件数 · 总占用」，`⟳` 重新扫描列表，`📁` 按钮折叠/展开面板。**窄屏（≤720px）下文件区变成左侧抽屉**，默认收起，点 `📁` 滑出来。

文件列表是**直接扫描存储目录**得到的，所以：

- 你手动往挂载的存储目录里丢的文件，刷新后也会出现在列表里，同样能下载和删除。
- `history.jsonl` 这类属于服务自身的数据文件会被排除，既不会列出、也删不掉。

### 4.6 或者直接从 Docker Hub 拉（可选）

镜像已经发布到 Docker Hub，是个多架构清单，路由器会自动选 arm64：

```sh
docker pull 14854/tiez-share:0.1.0     # 也可用 :latest
```

这样 `compose.yaml` 里的 `image:` 可以从 `tiez-share:arm64` 换成 `14854/tiez-share:0.1.0`，
以后升级就是 `docker compose pull && docker compose up -d`，不用再传 tar 包。
怎么自己重新推见第 11 节。

---

## 5. 配置项

全部通过环境变量控制，改完 `docker compose up -d` 重建即可。

| 变量 | 默认值 | 说明 |
| :--- | :--- | :--- |
| `TIEZ_PORT` | `12345` | 监听端口 |
| `TIEZ_BIND` | `0.0.0.0` | 监听地址 |
| `TIEZ_DATA_DIR` | `/data` | 文件与历史的存放目录（**挂到外置盘**） |
| `TIEZ_HISTORY_FILE` | `<DATA_DIR>/history.jsonl` | 聊天记录文件 |
| `TIEZ_MAX_HISTORY` | `1000` | 保留多少条记录（每条写一行盘） |
| `TIEZ_TITLE` | `局域网聊天室` | 页面标题 |
| `TIEZ_THEME` | `mica` | `sticky-note` / `paper` / `mica` / `acrylic` |
| `TIEZ_COLOR_MODE` | `system` | `system` / `light` / `dark` |
| `TIEZ_LOGO` | 内置 SVG | 头像图标，可填图片 URL 或 data URI |
| `TIEZ_JOIN_NOTICE` | `1` | 是否提示"某某已加入/已离开" |

### 端口冲突了怎么办

iStoreOS 上 12345 被占用的话，把 compose 里的 `"12345:12345"` 改成 `"18080:12345"`（只改左边，容器内还是 12345），然后访问 `http://路由器IP:18080`。

---

## 6. 安全提醒

这个服务**没有任何认证**，设计前提是"只在家里局域网里跑"：

- 局域网内任何人打开这个地址，就能看到全部聊天记录、下载全部中转文件。
- 明文 HTTP，同网段抓包可见内容。
- **不要做端口映射到公网**。需要外网访问就用 WireGuard/Tailscale 回家再连，或者在前面套一层带认证的反向代理。
- 想限制访问范围，可以在路由器防火墙里只放行指定网段/设备访问 `12345`。
- 容器默认以 root 运行（这样挂载盘权限最不容易出问题）。想收紧的话，在 compose 里加 `user: "1000:1000"`，并确保 `/mnt/sda1/tiez-share` 属于该 uid。

---

## 7. 排错

| 现象 | 原因 / 处理 |
| :--- | :--- |
| 启动日志 `data dir ... is not writable` | 挂载盘没挂上或权限不对；`df -h` 确认挂载，`chmod 777 /mnt/sda1/tiez-share` |
| `cannot bind 0.0.0.0:12345` | 端口被占用，改映射端口 |
| 页面能开但发不出消息 | 看 `docker compose logs`；确认用的 `http://` 而不是 `https://`（WebSocket 协议会自动跟随） |
| 图片显示不出来 | 文件已过期或不在盘上；记录里只保留 1000 条，超出会滚动删除 |
| 手机上传大文件失败 | 服务端不限制体积，一般是浏览器/网络问题；大文件建议走分片上传（网页端已内置） |
| 想看实时状态 | `docker exec tiez-share wget -qO- http://127.0.0.1:12345/healthz` |

---

## 8. 自检（可选）

仓库里带了一个零依赖的端到端自检脚本（只用 Node 22+ 内置的 `fetch` / `WebSocket`，不需要 `npm install`），可以验证聊天室、上传下载、分片、断点续传、路径穿越防护、左侧文件区是否正常：

```bash
# 本地测试镜像
docker run -d --name tiez-probe -p 18080:12345 -v tiez-probe-data:/data tiez-share:amd64
BASE=http://127.0.0.1:18080 node tests/smoke.mjs

# 或者直接打路由器（注意：会在聊天记录里留下测试消息和测试文件）
BASE=http://192.168.1.2:12345 node tests/smoke.mjs
```

输出形如（本仓库实测结果）：

```
[3] 两个设备互相可见（聊天室核心）
  ✓ B 看到两台在线设备
  ✓ B 实时收到 A 发的文字
  ✓ 消息带正确的 sender_id
  ...
[8] 左侧文件区：列表 / 下载 / 删除
  ✓ 页面含左侧文件区骨架
  ✓ 聊天记录文件不会被当成共享文件列出
  ✓ 删除文件后聊天里对应的文件消息一并移除
  ...
全部通过：53 项检查
```

> 脚本会往服务里写测试数据和测试文件，**别拿它打你正在用的实例**——删起来麻烦。
> 只想过一遍健康检查的话，`curl http://路由器IP:12345/healthz` 就够了。

---

## 9. 目录结构

```
tiez-share/
├── Cargo.toml / Cargo.lock
├── Dockerfile              # 默认产出 linux/arm64
├── docker-compose.yml      # iStoreOS 部署用
├── build-image.ps1         # Windows 构建/导出脚本
├── build-image.sh          # Linux/macOS 构建/导出脚本
├── push-dockerhub.ps1      # 推送到 Docker Hub（产出多架构清单）
├── tiez-share-arm64.tar    # docker save 产物，scp 到路由器 docker load 即可
├── tiez-share-aarch64      # 同一镜像抽出的静态二进制（不走 Docker 的备选方案）
├── openwrt/
│   └── tiez-share.init     # OpenWrt procd 启动脚本
├── tools/
│   └── proxy-auth.mjs      # 推送时用的本地认证注入代理（见第 11 节）
├── LICENSE                 # GPL-3.0（继承自上游）
├── tests/
│   └── smoke.mjs           # 零依赖端到端自检
└── src/
    ├── main.rs             # 配置、启动、优雅退出
    ├── handlers.rs         # 路由（移植自上游 handlers.rs）
    ├── models.rs           # 数据结构与运行状态
    ├── store.rs            # 历史持久化、文件名/token 处理
    └── web_ui.rs           # 网页聊天室界面（源自上游，GPL-3.0）
```

---

## 10. 附：不走 Docker 的备选方案（OpenWrt 原生服务）

`tiez-share-aarch64` 是从同一个镜像里抽出来的**静态链接**二进制（1.4 MB，
`ELF 64-bit ARM aarch64, statically linked, stripped`），不依赖 OpenWrt 的任何库。
万一你的路由器上 Docker 版本太老、`docker load` 不认这个包，可以直接当原生服务跑：

```sh
# 电脑上
scp tiez-share-aarch64      root@192.168.1.2:/usr/bin/tiez-share
scp openwrt/tiez-share.init root@192.168.1.2:/etc/init.d/tiez-share

# 路由器上
chmod +x /usr/bin/tiez-share /etc/init.d/tiez-share
mkdir -p /mnt/sda1/tiez-share
/etc/init.d/tiez-share enable     # 开机自启
/etc/init.d/tiez-share start
logread -e tiez-share -f          # 看启动日志
```

端口、标题、存储目录在 `/etc/init.d/tiez-share` 顶部改，改完 `restart` 生效。
这种方式没有容器开销，比 Docker 更省内存，适合小内存设备。

---

## 11. 发布到 Docker Hub（可选）

### 11.1 已经发好的镜像

```sh
docker pull 14854/tiez-share:0.1.0          # 或 :latest
docker pull 14854/tiez-share:0.1.0-arm64    # 只要某一个架构时
```

`:0.1.0` 和 `:latest` 是 manifest list，同时包含 `linux/amd64` 与 `linux/arm64`，
`docker pull` 会按机器架构自动选，所以同一个标签在 x86 电脑和 ARM 路由器上都能用。

### 11.2 自己重新推

先按第 3 节构建出两个架构的本地镜像，然后：

```powershell
.\push-dockerhub.ps1 -Username 你的用户名 -Version 0.1.0
```

脚本流程：检查两个架构的镜像 → `docker login` → 分别推 `:版本-amd64` / `:版本-arm64`
→ 用 `docker buildx imagetools create` 合成 `:版本` 和 `:latest` 多架构清单 → 打印清单做验证。

登录建议用 Access Token（Docker Hub → 右上角头像 → Account settings → Personal access tokens
→ Generate new token，权限选 **Read & Write**），推完可以在后台直接删掉。

> 小提示：`docker push` 的进度输出走 stderr，PowerShell 会把它显示成红色的 NativeCommandError，
> 脚本结束时也可能返回非零退出码——只要输出里能看到 `Pushed` 和 digest，就是成功了。

### 11.3 国内网络：推送必须走代理，而加速器帮不上

镜像加速器（`daemon.json` 的 registry-mirrors）只代理**拉取**，推送一律直连
`registry-1.docker.io` 和上传主机 `*.cloudflare.docker.com`。国内典型现象是：

- `registry-1.docker.io` 被 DNS 污染，解析到不相干的 IP（实测见过 `199.59.148.229`、`162.125.32.2`）；
- 直连全部超时，但**拉取因为走加速器一切正常**，很容易误以为网络没问题。

解决办法是给 Docker Desktop 配一个能出网的代理，但有两个坑：

**坑 1：Docker Desktop 不允许代理地址里带用户名密码。** 直接填
`http://用户名:密码@主机:端口`，后端会启动即崩溃：

```
loading/formatting settings-store.json: invalid http proxy in user settings: must not include credentials
```

症状是"Docker Desktop 起不来"，且只在启动时出现一次。两种绕法：在 GUI 里让它弹框输入凭据
（会存进 Windows 凭据管理器），或者用本项目的 `tools/proxy-auth.mjs`——本地开一个不要求
认证的代理，转发时自动补上上游凭据。

**坑 2：改 `settings-store.json` 之前必须先退出 Docker Desktop。** 它退出时会覆写这个文件，
你的改动会丢。另外**不要删掉文件里原有的键**，尤其 `UseContainerdSnapshotter`：少了它
Docker Desktop 会切换镜像存储，之前加载的镜像会集体"消失"（实际没丢，把键加回去重启就复原了）。

完整步骤（Windows + Docker Desktop，实测可用）：

```powershell
# 1) 起本地认证注入代理（上游地址和凭据换成你自己的）
$env:UPSTREAM = "192.168.8.100:7893"; $env:UPSTREAM_AUTH = "用户名:密码"
node tools\proxy-auth.mjs 7899

# 2) 退出 Docker Desktop（这步不能省）
docker desktop stop

# 3) 编辑 %APPDATA%\Docker\settings-store.json，加入四项（不要删原有键）
#    "ProxyHttpMode": "manual",
#    "OverrideProxyHTTP": "http://127.0.0.1:7899",
#    "OverrideProxyHTTPS": "http://127.0.0.1:7899",
#    "OverrideProxyExclude": "localhost,127.0.0.1"

# 4) 启动 Docker Desktop 并推送
docker desktop start
.\push-dockerhub.ps1 -Username 你的用户名 -Proxy http://127.0.0.1:7899
```

**推完记得把第 3 步那四项删掉、重启 Docker Desktop**：本地注入代理一停，
所有出网（连同走加速器的拉取）都会失败。

### 11.4 推完怎么确认

```sh
docker buildx imagetools inspect 14854/tiez-share:latest
```

输出里 `MediaType` 是 `application/vnd.docker.distribution.manifest.list.v2+json`，
且能看到 `linux/amd64` 和 `linux/arm64` 两条，就说明多架构清单没问题。
也可以直接在路由器上拉一次，拉下来的镜像 ID 应该和本机 `tiez-share:arm64` 那个一致。

公开分发镜像别忘了 GPL-3.0 的源码义务，见下一节。

### 11.5 管理 Hub 上的镜像（列标签 / 清理旧版本）

推送久了 Hub 上会堆一批历史版本。`hub-images.ps1` 用来查看和清理：

```powershell
# 列出所有标签（公开仓库匿名可读，不需要凭据）
.\hub-images.ps1

# 只保留最新 3 个版本，先干跑看要删什么（干跑不需要凭据）
.\hub-images.ps1 -Prune -Keep 3 -DryRun

# 确认无误后真删（会要求输入 yes）
.\hub-images.ps1 -Prune -Keep 3

# 删指定标签
.\hub-images.ps1 -Delete 0.0.9,0.0.9-arm64
```

用户名默认已填 `14854`，仓库名默认 `tiez-share`，日常不用带参数。

**保留规则**：只有能解析成 `主.次.补丁` 的标签参与版本排序，`latest` 这类非版本标签
永不删除。删掉某个版本时，它的 `-amd64` / `-arm64` 架构标签会一起删掉。

**删除需要 Access Token**：在 <https://hub.docker.com/settings/security> 新建一个，
权限勾 **Read & Delete**。注意是 token 不是登录密码——账号开了 2FA 的话密码登录一定失败。
两种给法：

```powershell
$env:DOCKERHUB_TOKEN = "dckr_pat_xxx"   # 推荐，只在本会话有效
.\hub-images.ps1 -Prune -Keep 3
```

不加参数时脚本会安全提示输入（不回显）。在 CI 之类非交互环境下会直接报错而不是卡住。

**网络**：这点和推送不一样——管理镜像只走 HTTP API，**不需要**改 Docker Desktop 的
`settings-store.json`，也不需要重启 Docker Desktop。只要第 11.3 节那个
`tools\proxy-auth.mjs` 代理在跑就行；脚本会自己检测，没开会告诉你怎么启动。
若你在墙外，加 `-NoProxy`。

---

## 12. 许可

上游 [jimuzhe/tiez-clipboard](https://github.com/jimuzhe/tiez-clipboard) 使用 **GPL-3.0**。本项目复用了它的代码（尤其是 `web_ui.rs` 界面部分），因此同样是 **GPL-3.0**，见 `LICENSE`。

如果你只是自己在家跑，不用管这些。但**如果把镜像或安装包公开分发（发到论坛、网盘、Docker Hub 等），GPL-3.0 要求你同时提供完整对应源码** —— 把这个目录一起放出去就符合要求了。
