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

另外顺手加固/增强的：启动时探测存储目录可写（挂载盘没挂上会立刻报错退出而不是默默失败）、端口被占用直接明确报错、`/healthz` 健康检查、主题/标题/设备名可配置、在线人数显示、加入退出提示（默认关闭，可用 `TIEZ_JOIN_NOTICE=1` 启用）；另外修了上传**失败时仍然打勾**的问题（失败现在显示「✗ 上传失败」），以及失败分支里 `pendingUploads.delete(name)` 的变量名笔误。

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

### 4.6 云编译下载

无需本地构建，可以从 GitHub Actions 下载对应架构的镜像包，导入后按上面的步骤启动，详见第 11 节。

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
| `TIEZ_JOIN_NOTICE` | `0` | 加入/离开提醒，默认关闭；设为 `1` 可启用 |

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
├── .github/workflows/build-images.yml # ARM64 / AMD64 云编译与自检
├── Cargo.toml / Cargo.lock
├── Dockerfile              # 默认产出 linux/arm64
├── docker-compose.yml      # iStoreOS 部署用
├── build-image.ps1         # Windows 构建/导出脚本
├── build-image.sh          # Linux/macOS 构建/导出脚本
├── openwrt/
│   └── tiez-share.init     # OpenWrt procd 启动脚本
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

云编译产物里的 `tiez-share-arm64` 是从同一个镜像里抽出来的**静态链接**二进制（1.4 MB，
`ELF 64-bit ARM aarch64, statically linked, stripped`），不依赖 OpenWrt 的任何库。
万一你的路由器上 Docker 版本太老、`docker load` 不认这个包，可以直接当原生服务跑：

```sh
# 电脑上
scp tiez-share-arm64        root@192.168.1.2:/usr/bin/tiez-share
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

## 11. GitHub Actions 云编译（树莓派 / x86）

打开 [Actions](https://github.com/Funccclub/tiez-share/actions/workflows/build-images.yml)，选择 **Run workflow** 即可手动构建。推送代码到 `main` 或创建 `v*` 标签也会自动构建；只修改文档不会触发。

两套镜像分别编译，并启动容器运行 `tests/smoke.mjs` 自检。ARM64 编译使用交叉工具链，运行自检时使用 QEMU。无需 Docker Hub 账号、密码或额外 Secrets。

| 下载产物 | 对应设备 | 镜像标签 |
|---|---|---|
| `tiez-share-arm64` | 树莓派 3/4/5 等运行 64 位系统的设备、ARM64 软路由 | `tiez-share:arm64` |
| `tiez-share-amd64` | x86_64 电脑、NAS、软路由 | `tiez-share:amd64` |

不包含 32 位 ARM 系统或 32 位 x86 的镜像。

成功运行页面底部的 **Artifacts** 可下载 ZIP（需登录 GitHub），保留 30 天。解压后包含镜像 `tar.gz`、静态二进制和 `SHA256SUMS`。它们是应用容器镜像，不是树莓派系统刷机镜像。

```sh
# 树莓派 / ARM64
sha256sum -c SHA256SUMS
docker load -i tiez-share-arm64.tar.gz

# x86_64 则导入 tiez-share-amd64.tar.gz
# 然后启动（按设备修改数据目录，x86_64 改用 tiez-share:amd64）
docker run -d --name tiez-share --restart unless-stopped \
  -p 12345:12345 \
  -v /mnt/sda1/tiez-share:/data \
  tiez-share:arm64
```

使用 Compose 时，默认引用本地已导入的 `tiez-share:arm64`；x86_64 修改 `image` 为 `tiez-share:amd64`。镜像不会自动发布到 Docker Hub 或 GHCR。

独立二进制也可以配合 `openwrt/tiez-share.init` 原生运行：ARM64 下载包中的 `tiez-share-arm64` 即为第 10 节使用的二进制，复制时将它重命名为 `tiez-share`。

---

## 12. 许可

上游 [jimuzhe/tiez-clipboard](https://github.com/jimuzhe/tiez-clipboard) 使用 **GPL-3.0**。本项目复用了它的代码（尤其是 `web_ui.rs` 界面部分），因此同样是 **GPL-3.0**，见 `LICENSE`。

如果你只是自己在家跑，不用管这些。但**如果把镜像或安装包公开分发（发到论坛、网盘、Docker Hub 等），GPL-3.0 要求你同时提供完整对应源码** —— 把这个目录一起放出去就符合要求了。
