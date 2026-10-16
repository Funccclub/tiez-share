import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

// Execute the actual embedded copy handlers, including the Rust format escaping.
const source = readFileSync(new URL('../src/web_ui.rs', import.meta.url), 'utf8');
const start = source.indexOf('        async function reportMessageCopied');
const end = source.indexOf('        function createMessageElement', start);
const code = source.slice(start, end).replaceAll('{{', '{').replaceAll('}}', '}');
const requests = [];
const listeners = {};
let copyOK = true;
let click;
let selected = false;
const message = { dataset: { id: '42' } };
const bubble = {
  closest: () => message, appendChild() {}, classList: { add() {} }
};
const context = vm.createContext({
  COPY_ICON: '', COPIED_ICON: '', renderedIds: new Set(),
  document: { createElement: () => ({
    setAttribute() {}, addEventListener(type, fn) { if (type === 'click') click = fn; },
    classList: { add() {}, remove() {} }
  }) },
  chatBox: {
    addEventListener(type, fn) { listeners[type] = fn; },
    querySelectorAll: () => [message], querySelector: () => null
  },
  window: { getSelection: () => ({ isCollapsed: !selected, containsNode: () => selected }) },
  fetch: async (url, options) => { requests.push({ url, options }); return { ok: true, status: 204 }; },
  copyToClipboard: async () => copyOK, renderToast() {}, setTimeout() {}
});
vm.runInContext(code, context);
context.attachCopyButton(bubble, 'hello');
const event = { preventDefault() {}, stopPropagation() {} };
await click(event);
assert.equal(requests.length, 1);
assert.equal(requests[0].url, '/messages/42/copied');
assert.equal(requests[0].options.method, 'POST');
copyOK = false;
await click(event);
assert.equal(requests.length, 1, 'failed clipboard operation does not renew retention');
listeners.copy({ defaultPrevented: false });
assert.equal(requests.length, 1, 'empty selection does not renew retention');
selected = true;
listeners.copy({ defaultPrevented: false });
assert.equal(requests.length, 2, 'selected chat content renews its message');
listeners.copy({ defaultPrevented: true });
assert.equal(requests.length, 2, 'cancelled native copy does not renew retention');
message.dataset.id = '';
copyOK = true;
await click(event);
assert.equal(requests.length, 2, 'pending upload without message id does not report activity');
console.log('Copy button success/failure, native selection and pending upload retention passed');
