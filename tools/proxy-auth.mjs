// 本地认证注入代理（发布到 Docker Hub 时用）
//
// 为什么需要它：
//   1. Docker Hub 在国内经常连不上（DNS 被污染、直连超时），而镜像加速器
//      只能加速「拉取」，推送必须直连 registry-1.docker.io 和上传主机。
//   2. Docker Desktop 不允许在代理地址里带用户名密码：
//        loading settings-store.json: invalid http proxy in user settings:
//        must not include credentials
//      而很多代理（例如 OpenClash 的 HTTP 端口）要求 Basic 认证。
//
// 于是：本脚本在 127.0.0.1 上开一个不要求认证的 HTTP 代理，
//      转发时自动补上 Proxy-Authorization 头，Docker Desktop 只需指向它。
//
// 用法：
//   node tools/proxy-auth.mjs 7899
//   环境变量 UPSTREAM_AUTH 设置上游凭据（用户名:密码，必填）
//   环境变量 UPSTREAM 可覆盖上游地址（默认 192.168.8.100:7893）
//
// 配套设置（Docker Desktop 不允许用 GUI 之外的方式带凭据，所以走配置文件）：
//   %APPDATA%\Docker\settings-store.json 里加：
//     "ProxyHttpMode": "manual",
//     "OverrideProxyHTTP": "http://127.0.0.1:7899",
//     "OverrideProxyHTTPS": "http://127.0.0.1:7899",
//     "OverrideProxyExclude": "localhost,127.0.0.1"
//   写完必须重启 Docker Desktop 才会生效。
//   注意：改这个文件前要先退出 Docker Desktop，否则它退出时会覆写你的修改；
//   同时不要删掉原有的 UseContainerdSnapshotter 等键，否则镜像存储会切换、
//   已加载的镜像会「消失」。
import net from "node:net";
import http from "node:http";

const LISTEN_PORT = Number(process.argv[2] || 7899);

const [UPSTREAM_HOST, UPSTREAM_PORT] = (process.env.UPSTREAM || "192.168.8.100:7893").split(":");
if (!process.env.UPSTREAM_AUTH) {
  console.error("请设置 UPSTREAM_AUTH 环境变量（用户名:密码）");
  process.exit(1);
}
const AUTH = "Basic " + Buffer.from(process.env.UPSTREAM_AUTH).toString("base64");

const log = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);

// ---- 普通 HTTP 代理请求（绝对形式 URL）----
const server = http.createServer((req, res) => {
  log(`HTTP  ${req.method} ${req.url}`);
  const up = http.request(
    {
      host: UPSTREAM_HOST,
      port: Number(UPSTREAM_PORT),
      method: req.method,
      path: req.url,
      headers: { ...req.headers, "proxy-authorization": AUTH },
    },
    (upRes) => {
      res.writeHead(upRes.statusCode || 502, upRes.headers);
      upRes.pipe(res);
    }
  );
  up.on("error", (e) => {
    log(`HTTP  上游出错: ${e.message}`);
    res.writeHead(502);
    res.end(String(e.message));
  });
  req.pipe(up);
});

// ---- HTTPS：CONNECT 隧道 ----
server.on("connect", (req, clientSocket, headBuf) => {
  const target = req.url;
  const up = net.connect(Number(UPSTREAM_PORT), UPSTREAM_HOST);
  let phase = "handshake";
  let buf = Buffer.alloc(0);

  up.on("connect", () => {
    // 由上游代理去解析域名，正好绕开被污染的本地 DNS
    up.write(
      `CONNECT ${target} HTTP/1.1\r\nHost: ${target}\r\nProxy-Authorization: ${AUTH}\r\n\r\n`
    );
  });

  up.on("data", (chunk) => {
    if (phase !== "handshake") return;
    buf = Buffer.concat([buf, chunk]);
    const idx = buf.indexOf("\r\n\r\n");
    if (idx === -1) return;

    const statusLine = buf.slice(0, buf.indexOf("\r\n")).toString("latin1");
    if (!/^HTTP\/1\.[01] 200/.test(statusLine)) {
      log(`CONNECT ${target} -> 上游拒绝: ${statusLine}`);
      phase = "failed";
      clientSocket.end("HTTP/1.1 502 Bad Gateway\r\n\r\n");
      up.destroy();
      return;
    }

    log(`CONNECT ${target} -> 隧道建立`);
    phase = "tunnel";
    up.removeAllListeners("data");
    clientSocket.write("HTTP/1.1 200 Connection Established\r\n\r\n");

    const rest = buf.slice(idx + 4);
    if (rest.length) clientSocket.write(rest);
    if (headBuf && headBuf.length) up.write(headBuf);

    up.pipe(clientSocket);
    clientSocket.pipe(up);
  });

  up.on("error", (e) => {
    if (phase === "handshake") {
      log(`CONNECT ${target} -> 连接上游失败: ${e.message}`);
      clientSocket.end("HTTP/1.1 502 Bad Gateway\r\n\r\n");
    } else {
      clientSocket.destroy();
    }
  });
  clientSocket.on("error", () => up.destroy());
  clientSocket.on("close", () => up.destroy());
  up.on("close", () => clientSocket.destroy());
});

server.listen(LISTEN_PORT, "127.0.0.1", () => {
  log(`认证注入代理已就绪: 127.0.0.1:${LISTEN_PORT} -> ${UPSTREAM_HOST}:${UPSTREAM_PORT}`);
});
