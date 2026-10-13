// Isolated Docker integration test; never touches a real deployment directory.
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, readdirSync, symlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';

const image = process.argv[2];
if (!image) throw new Error('Usage: node tests/retention.mjs IMAGE');
const root = mkdtempSync(join(tmpdir(), 'tiez-retention-'));
const data = join(root, 'data'); mkdirSync(data);
const outside = join(root, 'outside.txt'); writeFileSync(outside, 'outside must survive');
writeFileSync(join(data, 'manual.txt'), 'manual must survive');
symlinkSync('/outside.txt', join(data, 'linked.txt'));
const name = 'tiez-retention-' + process.pid;
const base = 'http://127.0.0.1:12346';
const pause = ms => new Promise(r => setTimeout(r, ms));
const docker = (...args) => execFileSync('docker', args, { encoding: 'utf8' });
async function waitFor(predicate, message) {
  for(let i=0;i<100;i++) { if(await predicate()) return; await pause(100); }
  throw new Error(message);
}
function start(ttl) {
  docker('run', '-d', '--name', name, '--network', 'host',
    '-e', 'TIEZ_PORT=12346', '-e', `TIEZ_FILE_TTL_SECS=${ttl}`,
    '-e', 'TIEZ_MAX_FILE_BYTES=8192', '-e', 'TIEZ_MAX_STORAGE_BYTES=10000',
    '-e', 'TIEZ_UPLOAD_TTL_SECS=1', '-e', 'TIEZ_CLEANUP_INTERVAL_SECS=1',
    '-v', `${data}:/data`, '-v', `${outside}:/outside.txt:ro`, image);
}
async function ready() {
  await waitFor(async()=>{try{return (await fetch(base+'/healthz')).ok;}catch{return false;}},'service startup');
}
async function upload(name, size) {
  const form = new FormData(); form.append('file', new Blob([new Uint8Array(size)]), name);
  return fetch(base+'/upload',{method:'POST',body:form});
}
async function files() { return (await (await fetch(base+'/files')).json()).files; }

try {
  start(0); await ready();
  assert.equal((await upload('too-large.bin', 9000)).status,413);
  assert.equal((await upload('first.bin', 6000)).status,200);
  let entries=await files();
  const first=entries.find(f=>f.name.endsWith('first.bin')); assert.ok(first);
  assert.equal((await upload('second.bin',6000)).status,200);
  entries=await files();
  assert.ok(!entries.some(f=>f.name.endsWith('first.bin')), 'quota evicts oldest managed file');
  assert.ok(entries.some(f=>f.name.endsWith('second.bin')));
  assert.ok(entries.some(f=>f.name==='manual.txt'), 'quota preserves unregistered file');
  const history=await (await fetch(base+'/poll')).json();
  assert.ok(!history.some(m=>m.file_path?.endsWith('first.bin')), 'cleanup removes stale history');
  assert.equal((await fetch(base+first.url)).status,404);
  const form=new FormData();
  form.append('metadata',JSON.stringify({upload_id:'abandoned',chunk_index:0,total_chunks:2,file_name:'partial.bin',total_size:20}));
  form.append('data',new Blob([new Uint8Array(10)]));
  assert.equal((await fetch(base+'/upload-chunk',{method:'POST',body:form})).status,200);
  await waitFor(()=>!readdirSync(data).some(n=>n.startsWith('.tmp_tiez_')),'partial upload cleanup');
  assert.ok(!(await files()).some(f=>f.name.endsWith('partial.bin')));
  docker('rm','-f',name);
  start(1); await ready();
  await waitFor(async()=>!(await files()).some(f=>f.name.endsWith('second.bin')),'retention survives restart');
  assert.ok(existsSync(join(data,'manual.txt')));
  assert.ok(existsSync(outside));
  console.log('Retention, quota, abandoned upload, restart, unregistered file and symlink protection passed');
} finally {
  try { console.log(docker('logs',name)); } catch {}
  try { docker('rm','-f',name); } catch {}
  rmSync(root,{recursive:true,force:true});
}
