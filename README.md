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
  --network host \
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
- 左边 `+` 发文件/图片，输入框发文字，**回车直接发送**（`Shift+回车` 换行，全屏编辑器同理；中文输入法组字时的回车是选词，不会误发）。右侧 `SEND` 按钮仍然可用。
- **鼠标移到任意消息上，气泡右下角会出现复制按钮**：文字消息复制正文，图片/视频/文件复制下载链接（自动补成绝对地址）。复制成功按钮会短暂变成 ✓。触屏没有悬停状态，按钮常驻半透明显示。
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

无需本地构建，可以直接 `docker pull 14854/tiez-share:latest`，同一个标签自动选择 ARM64 或 AMD64。也可从 GitHub Actions 下载镜像包离线导入，详见第 11 节。

---

## 5. 配置项

全部通过环境变量控制，改完 `docker compose up -d` 重建即可。

| 变量 | 默认值 | 说明 |
| :--- | :--- | :--- |
| `TIEZ_PORT` | `12345` | 监听端口 |
| `TIEZ_BIND` | `0.0.0.0` | 监听地址；Linux 推荐 `::` 同时接受 IPv6/IPv4 |
| `TIEZ_DATA_DIR` | `/data` | 文件与历史的存放目录（**挂到外置盘**） |
| `TIEZ_HISTORY_FILE` | `<DATA_DIR>/history.jsonl` | 聊天记录文件 |
| `TIEZ_MAX_HISTORY` | `1000` | 保留多少条记录（每条写一行盘） |
| `TIEZ_TITLE` | `局域网聊天室` | 页面标题 |
| `TIEZ_THEME` | `mica` | `sticky-note` / `paper` / `mica` / `acrylic` / `liquid-glass` |
| `TIEZ_COLOR_MODE` | `system` | `system` / `light` / `dark` |
| `TIEZ_LOGO` | 内置 SVG | 头像图标，可填图片 URL 或 data URI |
| `TIEZ_JOIN_NOTICE` | `0` | 加入/离开提醒，默认关闭；设为 `1` 可启用 |
| `TIEZ_LAN_CIDRS` | 自动检测 | 留空时最多每 5 秒重新识别活动 LAN 网段；填写时固定网段，逗号分隔 |
| `TIEZ_LAN_INTERFACE` | 自动选择 | 可固定网卡名称，例如 `br-lan` 或 `phy0-sta0`，该网卡上的网段仍自动刷新；仅在 CIDRS 留空时生效 |
| `TIEZ_TRUSTED_PROXIES` | 空 | 允许传递真实客户端地址的代理主机，例如本机 Lucky `127.0.0.1/32`；只接受 /32 或 /128，默认不信任代理 |
| `TIEZ_MAX_FILE_BYTES` | `2147483648` | 单文件上限 2 GiB；`0` 表示不限 |
| `TIEZ_MAX_STORAGE_BYTES` | `10737418240` | 文件总容量 10 GiB；超限先清理最早上传的已登记文件；`0` 表示不限 |
| `TIEZ_FILE_TTL_SECS` | `604800` | 上传文件保留 7 天；`0` 关闭按时间清理 |
| `TIEZ_MESSAGE_IDLE_TTL_SECS` | `86400` | 单条消息连续 24 小时无人复制后清空；`0` 关闭 |
| `TIEZ_UPLOAD_TTL_SECS` | `3600` | 未完成上传和重试记录闲置 1 小时后回收 |
| `TIEZ_CLEANUP_INTERVAL_SECS` | `60` | 周期清理间隔；上传时也检查容量 |

### 液态玻璃主题

在 Compose 的 `environment` 中将 `TIEZ_THEME` 设为 `"liquid-glass"`，重建容器即可启用。支持 `TIEZ_COLOR_MODE=system/light/dark`；默认主题仍为 `mica`。

新主题提供柔和渐变背景、浮动玻璃顶栏和输入栏、圆角文件面板与清晰的消息气泡。桌面 Chromium 浏览器额外显示轻微边缘折射；Safari、Firefox 与触屏设备使用毛玻璃和高光。只有顶栏和输入栏生成折射贴图，尺寸变化时更新，不在滚动时持续计算。所有 CSS/JS 都内嵌在程序中，无 CDN、外部字体或运行时依赖，可离线使用。

减少动态效果、减少透明度和强制色彩模式会分别降低动画或改用实色表面。浏览器不支持背景模糊时也会使用实色面板。

视觉设计参考 [hwyuanzi/LiquidGlass-UI](https://github.com/hwyuanzi/LiquidGlass-UI) 的材质层次与主题变量，以及 [rdev/liquid-glass-react](https://github.com/rdev/liquid-glass-react) 的边缘高光与折射表现。本项目的 CSS 与 JS 为独立实现，没有引入这两个项目的组件或源码。

### 上传完整性与重试

网页分片上传前计算完整文件及每个分片的 SHA-256（普通 HTTP 局域网页面也可用），服务端校验分片顺序、内容和最终大小/哈希。网络错误、超时和临时服务错误最多自动重试 3 次，已收到且内容一致的分片只确认一次，最后一片重试也不会重复产生文件或消息。空文件也能上传。

旧客户端可以不传 SHA-256，仍会执行顺序、重复内容和总大小校验。每片最大 1 MiB；网页使用 512 KiB。上传先写隐藏临时文件，全部校验成功后才发布。进程重启后未完成的上传需重新开始；这里的重试不等同于跨重启断点续传。

### 消息闲置清理

每条消息从发送时开始独立计时，连续 **24 小时无人复制就从聊天记录中清除**。网页复制按钮成功复制后（文字正文或文件链接），或选中聊天内容使用键盘/菜单复制后，该条消息重新计时 24 小时。仅查看、刷新、下载文件和其他消息的复制不会续期。外部客户端需在复制后调用 `POST /messages/{id}/copied`；服务端以自己的时间记录，已过期或不存在的消息返回 `404`。

复制时间会写入历史文件，重启后继续计时；旧记录没有复制时间时使用原发送时间，因此升级时已闲置超过 24 小时的旧消息也会清除。后台默认每 60 秒检查，启动和读取历史时也会清理，在线页面同步移除，断线重连后重新同步。可通过 `TIEZ_MESSAGE_IDLE_TTL_SECS` 调整。这里清除的是聊天记录，上传文件仍按下面的文件保留期限及容量规则处理。

### 文件自动清理

**默认会删除超过 7 天的上传文件；容量超出 10 GiB 时，优先删除最早上传的文件。** 删除会同步移除关联聊天记录并刷新在线设备的文件区。可通过上述环境变量调整。

只对新版登记到 `.tiez-managed.json` 的上传文件执行自动淘汰；手动放入的文件、升级前未登记的旧文件、历史文件和符号链接不会自动删除。手动文件仍计入容量，无法腾出空间时返回 `507`，单文件超限返回 `413`。不要删除登记文件，否则这些上传会失去自动清理记录。未完成上传的 `.tmp_tiez_*` 文件会在超时后清理，包括服务重启遗留的临时文件。


### 端口冲突了怎么办

iStoreOS 使用 host 网络，12345 被占用时把 `TIEZ_PORT` 改成 `18080`，然后访问 `http://路由器IP:18080`。无需配置 `ports` 映射。

---

## 6. 安全提醒

服务无需密码，只接受本机回环地址和允许的局域网来源，限制覆盖网页、API、上传、下载和 WebSocket。

- Linux 优先选择已启用且有链路的 OpenWrt `br-lan`，否则选择活动的默认路由网卡，或唯一候选网卡。排除默认 Docker 网桥、veth 和常见 VPN 网卡。允许该网卡的 IPv4 私网/链路本地网段、IPv6 ULA/链路本地网段，以及同一公网 IPv6 `/64` 或更窄网段；公网 IPv6 地址也可属于同一家庭 LAN，并不因此放行其它公网网段。忽略废弃、暂定、DAD 失败的 IPv6 地址。无法唯一选择时只允许本机；非 Linux 自动检测不可用时必须填写 `TIEZ_LAN_CIDRS`。
- iStoreOS/树莓派 Compose 使用 `network_mode: host`。将 `TIEZ_LAN_CIDRS` 留空，IPv4 换网段或运营商 IPv6 前缀变化后，下一次请求会触发最多每 5 秒一次的刷新，无需重启；旧网段会撤销。有复杂多网卡时用 `TIEZ_LAN_INTERFACE` 固定家庭网卡名称，避免选择错误。明确填写 CIDRS 会关闭自动刷新。
- 默认拒绝带来源转发头的请求。需要 HTTPS 域名反向代理时，设置 `TIEZ_TRUSTED_PROXIES` 为实际代理 TCP 来源的单个主机，例如本机 Lucky `127.0.0.1/32`，并保留原始 Host。代理必须用实际连接来源覆盖或追加到 `X-Forwarded-For` 末尾；程序只取最后一个地址做 LAN 检查，不信任客户端自行填写的首个地址。来自其它连接的转发头、缺失客户端来源、跨站 Origin 均拒绝。代理来源为回环时，仅 `/healthz` 允许无转发头的本机健康探测。不要信任会隐藏外网来源的代理。
- 网段判断不是物理位置证明。若网关、VPN 或代理把远程访问改写为本地来源，应用无法区分；不要将服务经这类设备公开转发，网关防火墙仍应阻止 WAN 访问。
- 同一允许网段内的设备可查看聊天记录、上传/下载和删除文件。HTTP 为明文传输。
- Windows/macOS 的 Docker Desktop 使用虚拟网络；推荐在实际 Linux 设备上部署。使用 bridge 模式必须配置真实 LAN CIDR，并确保连接保留原始来源；不要把所有私网都作为放行范围。

---

## 7. 排错

| 现象 | 原因 / 处理 |
| :--- | :--- |
| 启动日志 `data dir ... is not writable` | 挂载盘没挂上或权限不对；`df -h` 确认挂载，`chmod 777 /mnt/sda1/tiez-share` |
| `cannot bind 0.0.0.0:12345` | 端口被占用，改 `TIEZ_PORT` |
| 页面能开但发不出消息 | 看 `docker compose logs`；确认用的 `http://` 而不是 `https://`（WebSocket 协议会自动跟随） |
| 图片显示不出来 | 文件已过期或不在盘上；记录里只保留 1000 条，超出会滚动删除 |
| 手机上传大文件失败 | 检查单文件/总容量限制；`413` 为超大小，`507` 为空间不足；网页会自动重试临时网络错误 |
| 想看实时状态 | `docker exec tiez-share wget -qO- http://127.0.0.1:12345/healthz` |

---

## 8. 自检（可选）

仓库里带了一个零依赖的端到端自检脚本（只用 Node 22+ 内置的 `fetch` / `WebSocket`，不需要 `npm install`），可以验证聊天室、上传下载、分片、分片重试/校验、路径穿越防护、左侧文件区是否正常：

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

自检里也覆盖了网页的复制按钮：按钮存在、气泡渲染时挂上、拦住冒泡（否则点"复制"会变成下载文件）、非安全上下文的 `execCommand` 兜底。

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
    ├── maintenance.rs      # 上传文件登记、容量与过期清理
    ├── network.rs          # 同网段来源限制
    └── web_ui.rs           # 网页聊天室界面（源自上游，GPL-3.0）
```

---

## 10. 附：不走 Docker 的备选方案（OpenWrt 原生服务）

云编译产物里的 `tiez-share-arm64` 是从同一个镜像里抽出来的**静态链接**二进制（`ELF 64-bit ARM aarch64, statically linked, stripped`），不依赖 OpenWrt 的任何库。
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

### 自动发布到 Docker Hub

源码仓库 → GitHub Actions 双架构编译与测试 → Docker Hub `14854/tiez-share`。仅在两个架构全部通过测试后发布；Pull Request 只测试，不使用发布凭据。

```sh
docker pull 14854/tiez-share:latest
```

`latest` 和 `sha-<完整提交 SHA>` 都是包含 ARM64/AMD64 的多架构镜像。推送 `v*` 标签时还会生成去掉 `v` 的版本标签。每个架构另有 `sha-<SHA>-arm64` / `sha-<SHA>-amd64` 标签。

首次设置只需在 GitHub 仓库 Settings → Secrets and variables → Actions 添加 `DOCKERHUB_TOKEN`，值为 Docker Hub `14854` 账号的 Read & Write Access Token。密钥不放进源码或 Compose。更换账号/仓库时需同时修改工作流的登录用户名和 `IMAGE`。

### 通过 Dockge 更新

在原 Stack 使用 `image: 14854/tiez-share:latest`，Linux 使用 `network_mode: host`，保留现有数据挂载路径。推荐 `TIEZ_BIND: "::"`、`TIEZ_LAN_CIDRS: ""` 自动跟随 IPv4/IPv6 网段；需要本机 Lucky 域名反代时设置 `TIEZ_TRUSTED_PROXIES: "127.0.0.1/32"`。首次升级将 `TIEZ_JOIN_NOTICE` 设为 `0`，然后在 Dockge 保存/部署。后续代码发布成功后，在 Dockge 点击该 Stack 的“更新”即可拉取新镜像并重建，数据挂载保持不变。Watchtower 自动更新默认关闭。

### 手动编译与离线下载


打开 [Actions](https://github.com/Funccclub/tiez-share/actions/workflows/build-images.yml)，选择 **Run workflow** 即可手动构建。推送代码到 `main` 或创建 `v*` 标签也会自动构建；只修改文档不会触发。

两套镜像分别编译，执行 Rust 网段判断单元测试，启动容器运行 `tests/smoke.mjs` 自检，再用 `tests/retention.mjs` 验证容量淘汰、过期/临时文件清理、重启持久化和手动文件/符号链接保护。ARM64 编译使用交叉工具链，运行自检时使用 QEMU。无需 Docker Hub 账号、密码或额外 Secrets。

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
  --network host \
  -v /mnt/sda1/tiez-share:/data \
  tiez-share:arm64
```

使用 Compose 时，默认引用 Docker Hub 的 `14854/tiez-share:latest`，自动匹配设备架构。使用离线导入的镜像时，将 `image` 改为本地的 `tiez-share:arm64` 或 `tiez-share:amd64`。

独立二进制也可以配合 `openwrt/tiez-share.init` 原生运行：ARM64 下载包中的 `tiez-share-arm64` 即为第 10 节使用的二进制，复制时将它重命名为 `tiez-share`。

---

## 12. 许可

上游 [jimuzhe/tiez-clipboard](https://github.com/jimuzhe/tiez-clipboard) 使用 **GPL-3.0**。本项目复用了它的代码（尤其是 `web_ui.rs` 界面部分），因此同样是 **GPL-3.0**，见 `LICENSE`。

如果你只是自己在家跑，不用管这些。但**如果把镜像或安装包公开分发（发到论坛、网盘、Docker Hub 等），GPL-3.0 要求你同时提供完整对应源码** —— 把这个目录一起放出去就符合要求了。
