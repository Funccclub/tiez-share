/**
 * TieZ Share 端到端自检
 *
 * 只用 Node 内置能力（fetch / FormData / Blob / WebSocket），不需要 npm install。
 * 需要 Node 22+（全局 WebSocket）。
 *
 *   node tests/smoke.mjs
 *   BASE=http://192.168.1.2:12345 node tests/smoke.mjs
 */

import { createHash, randomBytes } from 'node:crypto';
import vm from 'node:vm';

const BASE = process.env.BASE || "http://127.0.0.1:12345";
const WS_URL = BASE.replace(/^http/, "ws") + "/ws";

let pass = 0;
const failures = [];

function check(name, cond, detail) {
  if (cond) {
    pass++;
    console.log(`  \u2713 ${name}`);
  } else {
    failures.push(name);
    console.log(`  \u2717 ${name}${detail ? `  -> ${detail}` : ""}`);
  }
}

function section(title) {
  console.log(`\n${title}`);
}

function connect(deviceId, deviceName) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(WS_URL);
    const inbox = [];
    const timer = setTimeout(() => reject(new Error("WebSocket 连接超时")), 5000);
    ws.addEventListener("message", (e) => {
      try {
        inbox.push(JSON.parse(e.data));
      } catch {
        /* ignore non-JSON */
      }
    });
    ws.addEventListener("open", () => {
      ws.send(JSON.stringify({ type: "identity", device_id: deviceId, device_name: deviceName }));
      setTimeout(() => {
        clearTimeout(timer);
        resolve({ ws, inbox });
      }, 250);
    });
    ws.addEventListener("error", () => {
      clearTimeout(timer);
      reject(new Error("WebSocket 连接失败"));
    });
  });
}

async function waitFor(inbox, pred, ms = 3000) {
  const start = Date.now();
  while (Date.now() - start < ms) {
    const hit = inbox.find(pred);
    if (hit) return hit;
    await new Promise((r) => setTimeout(r, 50));
  }
  return null;
}

async function pollAll() {
  const res = await fetch(`${BASE}/poll?last_id=0`);
  if (!res.ok) throw new Error(`/poll 返回 ${res.status}`);
  return res.json();
}

async function upload(fileName, bytes, { senderId = "device-A", senderName = "设备A" } = {}) {
  const fd = new FormData();
  fd.append("sender_id", senderId);
  fd.append("sender_name", senderName);
  fd.append("file", new Blob([bytes], { type: "application/octet-stream" }), fileName);
  const res = await fetch(`${BASE}/upload`, { method: "POST", body: fd });
  return res.status;
}

async function uploadChunk(meta, bytes) {
  const fd = new FormData();
  fd.append("metadata", JSON.stringify(meta));
  fd.append("data", new Blob([bytes], { type: "application/octet-stream" }));
  const res = await fetch(`${BASE}/upload-chunk`, { method: "POST", body: fd });
  return res.status;
}

async function download(pathWithQuery) {
  const res = await fetch(`${BASE}${pathWithQuery}`);
  const buf = new Uint8Array(await res.arrayBuffer());
  return { status: res.status, buf, headers: res.headers };
}

const PAYLOAD = new TextEncoder().encode(
  "TieZ Share 中转测试内容\n" + "x".repeat(4096) + "\n— end —\n"
);

console.log(`TieZ Share 端到端自检 @ ${BASE}`);

// ---------------------------------------------------------------- 网页界面
section("[1] 网页聊天室界面");
const indexRes = await fetch(`${BASE}/`);
const html = await indexRes.text();
const hashSource = html.slice(html.indexOf('class UploadSHA256'), html.indexOf('async function uploadFile'));
const BrowserHash = vm.runInNewContext(hashSource + '; UploadSHA256');
for (const size of [0, 1, 55, 56, 63, 64, 65, 1025, 524288]) {
  const data = randomBytes(size), hash = new BrowserHash();
  for (let i = 0; i < data.length; i += 37) hash.update(data.subarray(i, i + 37));
  check(`网页 SHA-256 正确（${size} bytes）`, hash.digest() === createHash('sha256').update(data).digest('hex'));
}
check("GET / 返回页面", indexRes.ok && html.includes("<!DOCTYPE html>"));
check("标题已注入配置项", html.includes("<h1>") && !html.includes("{title}"), "模板占位符没有被替换");
check("无 Tauri 残留", !html.includes("__TAURI__") && !html.includes("invoke("), "浏览器里 invoke 会直接报错");
check(
  "无外部 CDN 依赖",
  !/<(script|link|img)[^>]+(src|href)=["']https?:\/\//i.test(html),
  "路由器断网时页面也要能用"
);
check("含消息归属判定（按 sender_id）", html.includes("msg.sender_id === deviceId"));
check("含历史加载", html.includes("/poll?last_id=0"));

// ------------------------------------------------------------------ 健康检查
section("[2] 健康检查");
const health = await (await fetch(`${BASE}/healthz`)).json();
check("GET /healthz", health.status === "ok", JSON.stringify(health));
check("报告存储目录", typeof health.data_dir === "string" && health.data_dir.length > 0);
for (const headers of [{ 'X-Forwarded-For': '192.168.8.2' }, { Forwarded: 'for=127.0.0.1' }, { Origin: 'https://external.example' }]) {
  check("拒绝代理来源和跨站请求 " + JSON.stringify(headers), (await fetch(`${BASE}/healthz`, { headers })).status === 403);
}

// ---------------------------------------------------------- 双设备 / 聊天室
section("[3] 两个设备互相可见（聊天室核心）");
const A = await connect("device-A", "设备A");
const B = await connect("device-B", "设备B");
const rosterA = await waitFor(A.inbox, (m) => m.type === "devices_update");
check("A 收到 devices_update", !!rosterA);
const rosterB = await waitFor(
  B.inbox,
  (m) => m.type === "devices_update" && Array.isArray(m.devices) && m.devices.length >= 2
);
check(
  "B 看到两台在线设备",
  !!rosterB,
  rosterB ? rosterB.devices.map((d) => d.name).join(",") : "设备列表里没有 A/B"
);

const textA = `hello-from-A-${Date.now()}`;
await fetch(`${BASE}/send-text`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ content: textA, sender_id: "device-A", sender_name: "设备A" }),
});
const gotB = await waitFor(B.inbox, (m) => m.content === textA);
check("B 实时收到 A 发的文字", !!gotB);
check(
  "消息带正确的 sender_id",
  gotB && gotB.sender_id === "device-A",
  gotB ? `sender_id=${gotB.sender_id}` : "未收到"
);
check("消息带可读的 sender_name", gotB && gotB.sender_name === "设备A");

const gotA = await waitFor(A.inbox, (m) => m.content === textA);
check("A 自己也收到回显（自己发的显示在自己这边）", !!gotA && gotA.sender_id === "device-A");

const textB = `hello-from-B-${Date.now()}`;
await fetch(`${BASE}/send-text`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ content: textB, sender_id: "device-B", sender_name: "设备B" }),
});
const bFromA = await waitFor(A.inbox, (m) => m.content === textB);
check(
  "A 收到的 B 的消息 sender_id 是 B",
  !!bFromA && bFromA.sender_id === "device-B",
  bFromA ? `sender_id=${bFromA.sender_id}` : "未收到"
);

// ------------------------------------------------------------------ 历史记录
section("[4] 历史记录");
const history = await pollAll();
const histA = history.find((m) => m.content === textA);
const histB = history.find((m) => m.content === textB);
check("历史里有 A 的消息", !!histA);
check("历史里有 B 的消息", !!histB);
check(
  "历史里发送方信息未被抹掉",
  histA && histA.sender_id === "device-A" && histA.sender_name === "设备A"
);
const ids = history.map((m) => m.id);
check(
  "消息 id 单调递增",
  ids.every((v, i) => i === 0 || v > ids[i - 1]),
  ids.join(",")
);

// ------------------------------------------------------------------ 文件上传
section("[5] 文件上传 / 下载");
const fname = `smoke-${Date.now()}.bin`;
const upStatus = await upload(fname, PAYLOAD);
check("POST /upload 成功", upStatus === 200, `status=${upStatus}`);

const afterUpload = await pollAll();
const fileMsg = [...afterUpload].reverse().find((m) => m.file_path && m.file_path.includes(fname));
check("上传后产生文件消息", !!fileMsg);
check(
  "文件消息指向 /download/ 链接",
  !!fileMsg && fileMsg.content.startsWith("/download/"),
  fileMsg ? fileMsg.content : ""
);
check("文件消息类型为 file", !!fileMsg && fileMsg.msg_type === "file");

if (fileMsg) {
  const dl = await download(fileMsg.content);
  check("下载返回 200", dl.status === 200, `status=${dl.status}`);
  check(
    "下载内容与原文件逐字节一致",
    dl.buf.length === PAYLOAD.length && dl.buf.every((b, i) => b === PAYLOAD[i]),
    `本地 ${PAYLOAD.length}B / 远端 ${dl.buf.length}B`
  );
  check(
    "支持 Range 断点续传（视频拖动依赖它）",
    dl.headers.get("accept-ranges") === "bytes"
  );

  const ranged = await fetch(`${BASE}${fileMsg.content}`, { headers: { Range: "bytes=0-9" } });
  const rangedBuf = new Uint8Array(await ranged.arrayBuffer());
  check("Range 请求返回 206", ranged.status === 206, `status=${ranged.status}`);
  check(
    "Range 返回的 10 字节正确",
    rangedBuf.length === 10 && rangedBuf.every((b, i) => b === PAYLOAD[i])
  );
}

// ------------------------------------------------------------------ 分片上传
section("[6] 分片上传（大文件走的路径）");
const chunkName = `chunked-${Date.now()}.bin`;
const uploadId = `smoke-${Date.now()}`;
const half = Math.floor(PAYLOAD.length / 2);
const meta = (i, total) => ({
  upload_id: uploadId,
  chunk_index: i,
  total_chunks: total,
  file_name: chunkName,
  sender_id: "device-A",
  sender_name: "设备A",
  total_size: PAYLOAD.length,
  content_type: "application/octet-stream",
  file_sha256: createHash('sha256').update(PAYLOAD).digest('hex'),
});
check("拒绝从第二片开始上传", await uploadChunk(meta(1, 2), PAYLOAD.slice(half)) === 409);
check("拒绝错误分片校验和", await uploadChunk({ ...meta(0, 2), chunk_sha256: '0'.repeat(64) }, PAYLOAD.slice(0, half)) === 422);
const s1 = await uploadChunk(meta(0, 2), PAYLOAD.slice(0, half));
check("相同分片重试成功且不重复追加", await uploadChunk(meta(0, 2), PAYLOAD.slice(0, half)) === 200);
check("拒绝内容不同的重复分片", await uploadChunk(meta(0, 2), new Uint8Array(half)) === 409);
check("拒绝途中修改元数据", await uploadChunk({ ...meta(1, 2), file_name: 'different.bin' }, PAYLOAD.slice(half)) === 409);
check("拒绝文件总大小不符", await uploadChunk(meta(1, 2), PAYLOAD.slice(half + 1)) === 422);
const s2 = await uploadChunk(meta(1, 2), PAYLOAD.slice(half));
check("最后一片重试幂等成功", await uploadChunk(meta(1, 2), PAYLOAD.slice(half)) === 200);
check("拒绝片号越界", await uploadChunk(meta(2, 2), PAYLOAD) === 400);
check("拒绝整文件校验失败", await uploadChunk({ ...meta(0, 1), upload_id: uploadId+'-bad', file_sha256: '0'.repeat(64) }, PAYLOAD) === 422);
check("零字节文件可上传", await uploadChunk({ ...meta(0, 1), upload_id: uploadId+'-empty', file_name: 'empty.bin', total_size: 0, file_sha256: createHash('sha256').digest('hex') }, new Uint8Array()) === 200);
check("分片 1 返回 200", s1 === 200, `status=${s1}`);
check("分片 2 返回 200", s2 === 200, `status=${s2}`);

const afterChunk = await pollAll();
const chunkMsg = [...afterChunk]
  .reverse()
  .find((m) => m.file_path && m.file_path.includes(chunkName));
check("分片收完后生成文件消息", !!chunkMsg);
check("最后一片重试不会生成重复消息", afterChunk.filter(m => m.file_path?.includes(chunkName)).length === 1);
if (chunkMsg) {
  const dl = await download(chunkMsg.content);
  check(
    "分片拼出的文件内容正确",
    dl.buf.length === PAYLOAD.length && dl.buf.every((b, i) => b === PAYLOAD[i]),
    `期望 ${PAYLOAD.length}B / 实际 ${dl.buf.length}B`
  );
}

// -------------------------------------------------------------- 目录穿越加固
section("[7] 路径穿越防护");
const evilName = "../../../../tmp/tiez-escape-probe.txt";
const evilStatus = await upload(evilName, new TextEncoder().encode("escape probe"));
check("恶意文件名不会导致上传失败", evilStatus === 200, `status=${evilStatus}`);

const afterEvil = await pollAll();
const evilMsg = [...afterEvil].reverse().find((m) => m.file_path && m.file_path.includes("tiez-escape-probe"));
check("恶意文件名被记录", !!evilMsg);
if (evilMsg) {
  check(
    "文件名里的目录部分被剥掉",
    !evilMsg.file_path.includes("..") && !evilMsg.file_path.includes("/tmp/"),
    evilMsg.file_path
  );
  const nameParam = decodeURIComponent(
    (evilMsg.content.split("?name=")[1] || "").replace(/\+/g, " ")
  );
  check("下载名不含 ..", !nameParam.includes(".."), nameParam);
}

// ------------------------------------------------------- 左侧文件区（新增）
section("[8] 左侧文件区：列表 / 下载 / 删除");

check(
  "页面含左侧文件区骨架",
  html.includes('id="file-panel"') && html.includes('id="file-list"'),
);
check("页面含拖拽上传覆盖层", html.includes('id="drop-overlay"'));
check("页面含文件区开关按钮", html.includes('id="files-toggle"'));

const filesRes = await fetch(`${BASE}/files`);
check("GET /files 返回 200", filesRes.status === 200, `status=${filesRes.status}`);
const filesData = await filesRes.json();
const fileList = filesData.files || [];
check("返回文件数组", Array.isArray(fileList) && fileList.length > 0, `count=${fileList.length}`);

const listed = fileList.find((f) => f.name.includes(fname));
check("刚上传的文件出现在列表里", !!listed);
check(
  "列表项字段完整（token/size/url）",
  !!(listed && listed.token && listed.size === PAYLOAD.length && listed.url),
  listed ? JSON.stringify({ token: listed.token, size: listed.size }) : "缺少该项",
);
check(
  "聊天记录文件不会被当成共享文件列出",
  !fileList.some((f) => f.name.startsWith("history")),
  fileList.map((f) => f.name).join(","),
);
check(
  "统计字段齐全且总量大于 0",
  typeof filesData.count === "number" &&
    typeof filesData.total_size === "number" &&
    filesData.total_size > 0,
  `count=${filesData.count} total=${filesData.total_size}`,
);

if (listed) {
  const dl = await download(listed.url);
  check(
    "用文件区给出的链接能下回原文件",
    dl.status === 200 && dl.buf.length === PAYLOAD.length,
    `status=${dl.status} ${dl.buf.length}B`,
  );

  const delRes = await fetch(`${BASE}/files/${encodeURIComponent(listed.token)}`, {
    method: "DELETE",
  });
  check("DELETE /files/{token} 返回 200", delRes.status === 200, `status=${delRes.status}`);

  const afterDel = await (await fetch(`${BASE}/files`)).json();
  check(
    "删除后列表里不再有该文件",
    !(afterDel.files || []).some((f) => f.name.includes(fname)),
  );

  const histAfterDel = await pollAll();
  check(
    "删除文件后聊天里对应的文件消息一并移除",
    !histAfterDel.some((m) => m.file_path && m.file_path.includes(fname)),
  );
  check(
    "其它聊天消息没被误删",
    histAfterDel.some((m) => m.content === textA) && histAfterDel.some((m) => m.content === textB),
  );

  const dlGone = await download(listed.url);
  check("删掉的文件再下载会 404", dlGone.status === 404, `status=${dlGone.status}`);
}

const badDel = await fetch(`${BASE}/files/deadbeefdeadbeef`, {
  method: "DELETE",
});
check("删除未知 token 返回 404", badDel.status === 404, `status=${badDel.status}`);

// ------------------------------------------------------- 消息复制按钮（新增）
section("[9] 消息复制按钮");

check(
  "页面含复制按钮样式骨架",
  html.includes(".copy-btn") && html.includes(".bubble.has-copy"),
);
check(
  "气泡渲染时挂上复制按钮",
  html.includes("attachCopyButton(bubble, copyTargetOf("),
);
check(
  "复制按钮拦住冒泡，不会误触发文件下载",
  html.includes("e.stopPropagation()"),
  "气泡本身 onclick 是下载；不拦住的话点复制会变成下载文件",
);
check(
  "非安全上下文有 execCommand 兜底",
  html.includes("isSecureContext") && html.includes("execCommand('copy')"),
  "路由器上是 http://192.168.x.x，不是安全上下文，navigator.clipboard 不存在",
);
check(
  "data:/blob: 不给复制按钮",
  html.includes("/^(data|blob):/"),
  "上传中的占位气泡走 blob:，复制出来没法用",
);

// ------------------------------------------------------- 回车发送（新增）
section("[10] 回车发送");

check("回车直接发送", html.includes("sendBtn.click()") && html.includes("isEnterToSend"));
check(
  "输入法组字时的回车不发送",
  html.includes("!e.isComposing") && html.includes("keyCode !== 229"),
  "中文候选词用回车确认；不挡的话每选一次词就发一条空消息",
);
check(
  "Shift+回车保留换行",
  /e\.shiftKey/.test(html) && /e\.preventDefault\(\)/.test(html),
);
check(
  "全屏编辑器行为一致",
  html.includes("fs-textarea')") || html.includes("fs-textarea\")"),
);

// ------------------------------------------------------------------ 清理连接
B.ws.close();
await new Promise((r) => setTimeout(r, 300));
if (process.env.EXPECT_JOIN_NOTICE !== "1") {
  check(
    "默认不发送访客加入/离开提醒",
    !A.inbox.some((m) => m.msg_type === "notice") &&
      !B.inbox.some((m) => m.msg_type === "notice")
  );
}
A.ws.close();
const afterClose = await pollAll();
check("断开连接后聊天记录仍在", afterClose.length >= history.length);

// ---------------------------------------------------------------------- 汇总
console.log("\n" + "=".repeat(56));
if (failures.length === 0) {
  console.log(`全部通过：${pass} 项检查`);
} else {
  console.log(`通过 ${pass} 项，失败 ${failures.length} 项：`);
  for (const f of failures) console.log(`  - ${f}`);
}
console.log("=".repeat(56));
process.exit(failures.length === 0 ? 0 : 1);
