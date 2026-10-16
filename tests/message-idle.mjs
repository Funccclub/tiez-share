// Isolated integration test with accelerated idle deadlines; never uses live data.
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';

const image = process.argv[2];
if (!image) throw new Error('Usage: node tests/message-idle.mjs IMAGE');
const data = mkdtempSync(join(tmpdir(), 'tiez-message-idle-'));
const name = 'tiez-message-idle-' + process.pid;
const base = 'http://127.0.0.1:12347';
const docker = (...args) => execFileSync('docker', args, { encoding: 'utf8' });
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const historyPath = join(data, 'history.jsonl');
const old = Date.now() - 2 * 86400000;
writeFileSync(historyPath, JSON.stringify({ id: 1, timestamp: old, content: 'legacy expired' }) + '\n');
let socket;
async function ready() {
  for (let i = 0; i < 100; i++) {
    try { if ((await fetch(base + '/healthz')).ok) return; } catch {}
    await pause(100);
  }
  throw new Error('startup timed out');
}
const history = async () => (await fetch(base + '/poll')).json();
const copied = id => fetch(`${base}/messages/${id}/copied`, { method: 'POST' });
async function send(content) {
  assert.equal((await fetch(base + '/send-text', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content })
  })).status, 200);
  return (await history()).find(m => m.content === content);
}
try {
  docker('run', '-d', '--name', name, '--network', 'host',
    '-e', 'TIEZ_PORT=12347', '-e', 'TIEZ_MESSAGE_IDLE_TTL_SECS=8',
    '-e', 'TIEZ_CLEANUP_INTERVAL_SECS=1', '-e', 'TIEZ_FILE_TTL_SECS=0',
    '-e', 'TIEZ_LAN_CIDRS=192.168.8.0/24', '-v', `${data}:/data`, image);
  await ready();
  assert.deepEqual(await history(), [], 'startup expires legacy records using send time');
  assert.equal((await copied(1)).status, 404, 'expired record cannot be revived');
  assert.equal((await copied(999)).status, 404);
  const events = [];
  socket = new WebSocket(base.replace('http', 'ws') + '/ws');
  await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
  socket.onmessage = e => events.push(JSON.parse(e.data));
  const idle = await send('no one copies this');
  const active = await send('copy extends this record');
  const form = new FormData();
  form.append('file', new Blob(['keep file after message expiry']), 'survives.txt');
  assert.equal((await fetch(base + '/upload', { method: 'POST', body: form })).status, 200);
  const file = (await (await fetch(base + '/files')).json()).files.find(f => f.name.endsWith('survives.txt'));
  const fileMsg = (await history()).find(m => m.file_path);
  await pause(5000);
  assert.equal((await copied(active.id)).status, 204);
  const touched = (await history()).find(m => m.id === active.id);
  assert.ok(touched.last_copied_at > touched.timestamp);
  const persisted = readFileSync(historyPath, 'utf8').trim().split('\n').map(JSON.parse);
  assert.equal(persisted.find(m => m.id === active.id).last_copied_at, touched.last_copied_at);
  await pause(3500);
  let remaining = await history();
  assert.ok(!remaining.some(m => m.id === idle.id), 'poll/viewing never renews idle messages');
  assert.ok(!remaining.some(m => m.id === fileMsg.id), 'file messages expire too');
  assert.ok(remaining.some(m => m.id === active.id), 'copy renews only its own message');
  assert.equal((await copied(idle.id)).status, 404);
  assert.equal((await fetch(base + file.url)).status, 200, 'chat expiry preserves uploaded file');
  await pause(100);
  assert.ok(events.some(e => e.type === 'messages_removed' && e.removed_message_ids.includes(idle.id)),
    'connected clients receive removal events');
  socket.close(); socket = null;
  docker('restart', name); await ready();
  remaining = await history();
  assert.equal(remaining.find(m => m.id === active.id)?.last_copied_at, touched.last_copied_at,
    'restart preserves copy time');
  await pause(5000);
  assert.deepEqual(await history(), [], 'renewed message eventually expires after restart');
  assert.equal(readFileSync(historyPath, 'utf8'), '', 'expired content removed from disk');
  assert.equal((await fetch(base + file.url)).status, 200);
  console.log('Message idle expiry, copy renewal, persistence, restart, broadcasts and file preservation passed');
} finally {
  socket?.close();
  try { console.log(docker('logs', name)); } catch {}
  try { docker('rm', '-f', name); } catch {}
  rmSync(data, { recursive: true, force: true });
}
