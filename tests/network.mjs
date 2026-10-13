// Isolated policy integration tests; never uses production files or interfaces.
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';
import { request } from 'node:http';
const image = process.argv[2];
if (!image) throw new Error('Usage: node tests/network.mjs IMAGE');
const name = `tiez-network-${process.pid}`;
const docker = (...args) => execFileSync('docker', args, { encoding: 'utf8' });
const base = 'http://127.0.0.1:12347';
const headers = ip => ({ 'X-Forwarded-For': ip });
// Raw HTTP preserves the proxy's Host header; fetch may replace forbidden Host.
function status(path, h = {}) {
  return new Promise((resolve, reject) => {
    const req = request(base + path, { headers: h }, res => { res.resume(); resolve(res.statusCode); });
    req.on('error', reject); req.end();
  });
}
try {
  docker('run', '-d', '--name', name, '--network', 'host',
    '-e', 'TIEZ_PORT=12347', '-e', 'TIEZ_BIND=::',
    '-e', 'TIEZ_LAN_CIDRS=192.168.8.0/24,2408:8474:5900:6aa5::/64',
    '-e', 'TIEZ_TRUSTED_PROXIES=127.0.0.1/32', image);
  let ready = false;
  for (let i = 0; i < 100; i++) {
    try { if (await status('/healthz') === 200) { ready = true; break; } } catch {}
    await new Promise(r => setTimeout(r, 100));
  }
  assert.ok(ready, 'IPv6 listener also accepts IPv4 health probes on Linux');
  assert.equal(await status('/files'), 403, 'trusted proxy must supply client identity');
  assert.equal(await status('/files', headers('192.168.8.20')), 200);
  assert.equal(await status('/files', headers('2408:8474:5900:6aa5::20')), 200);
  assert.equal(await status('/files', headers('2408:8474:5900:6aa6::20')), 403);
  assert.equal(await status('/files', headers('192.168.9.20')), 403);
  assert.equal(await status('/files', headers('8.8.8.8')), 403);
  assert.equal(await status('/files', headers('192.168.8.20, 2001:4860::1')), 403, 'prepended LAN spoof cannot allow WAN');
  assert.equal(await status('/files', headers('2408:8474:5900:6aa5::20, 2408:8474:5900:6aa5::20')), 200, 'Lucky duplicate XFF');
  assert.equal(await status('/files', headers('192.168.8.20, invalid')), 403);
  assert.equal(await status('/files', headers('127.0.0.1')), 403);
  assert.equal(await status('/files', { 'X-Real-IP': '192.168.8.20' }), 403);
  assert.equal(await status('/files', { ...headers('192.168.8.20'), Origin: 'https://evil.example' }), 403);
  assert.equal(await status('/files', { ...headers('192.168.8.20'), Host: 'tiez.example', Origin: 'https://tiez.example' }), 200);
  assert.equal((await fetch('http://[::1]:12347/files')).status, 200, 'native IPv6 direct connection');
  assert.equal((await fetch('http://[::1]:12347/files', { headers: headers('192.168.8.20') })).status, 403, 'untrusted IPv6 peer cannot forge proxy headers');
  let denied = false;
  try {
    docker('run', '--rm', '--add-host=host.docker.internal:host-gateway',
      '--entrypoint', 'wget', image, '--header=X-Forwarded-For: 192.168.8.20',
      '-S', '-O-', 'http://host.docker.internal:12347/files');
  } catch (error) { denied = /403/.test(String(error.stderr)); }
  assert.ok(denied, 'untrusted remote TCP peer cannot forge a local client');
  console.log('PASS network: IPv4/IPv6, trusted proxy, spoofing, origin, missing identity');
} finally {
  try { docker('rm', '-fv', name); } catch {}
}
