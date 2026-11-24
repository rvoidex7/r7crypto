#!/usr/bin/env node
/**
 * Browser runtime smoke test for the Trunk/WASM build.
 *
 * The Rust gates prove that the code compiles and that host-only APIs are not
 * used, but they cannot prove that the page actually boots: wasm-bindgen glue,
 * WebGL context creation and the eframe frame loop can still fail at runtime.
 * This script serves `dist/` locally, opens it in headless Chromium over the
 * DevTools protocol, and fails on any console error or uncaught exception.
 *
 * Zero dependencies: Node's built-in WebSocket speaks CDP directly.
 * Requires `trunk build --release` to have been run first. `--self-test` feeds
 * the detector a synthetic console error to prove it is not silently green.
 */
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '..');
const DIST = path.join(ROOT, 'dist');
// `--self-test` proves the detector is actually attached: a synthetic page
// logs a known error that must be reported. Without this, a broken listener
// would make every smoke run look green.
const selfTest = process.argv.includes('--self-test');
const SELF_TEST_HTML = `<!doctype html><title>r7 Crypto Lab</title>
<canvas id="r7crypto_canvas" width="64" height="64"></canvas>
<script>console.error('SMOKE_SELF_TEST');</script>`;
const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.css': 'text/css; charset=utf-8',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.json': 'application/json',
};

function findChrome() {
  const candidates = [
    process.env.CHROME_BIN,
    '/usr/bin/google-chrome',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
    path.join(os.homedir(), '.local/bin/chromium'),
    path.join(os.homedir(), '.local/bin/google-chrome'),
  ];
  return candidates.find((candidate) => candidate && existsSync(candidate));
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function waitFor(predicate, timeoutMs, label) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const value = await predicate();
    if (value) return value;
    await sleep(200);
  }
  throw new Error(`timed out waiting for ${label}`);
}

function startServer(inlineHtml) {
  const server = createServer(async (req, res) => {
    if (inlineHtml) {
      res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      res.end(inlineHtml);
      return;
    }
    let pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    if (pathname === '/favicon.ico') {
      res.writeHead(204);
      res.end();
      return;
    }
    // The Pages build uses /r7crypto/ as its base; local builds may use "/".
    if (pathname.startsWith('/r7crypto/')) {
      pathname = pathname.slice('/r7crypto'.length);
    }
    if (pathname === '' || pathname === '/') pathname = '/index.html';
    const file = path.resolve(DIST, `.${pathname}`);
    if (!file.startsWith(DIST)) {
      res.writeHead(403);
      res.end('forbidden');
      return;
    }
    try {
      const body = await readFile(file);
      res.writeHead(200, {
        'content-type': MIME[path.extname(file)] ?? 'application/octet-stream',
      });
      res.end(body);
    } catch {
      res.writeHead(404);
      res.end('not found');
    }
  });
  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () =>
      resolve({ server, url: `http://127.0.0.1:${server.address().port}/r7crypto/` }),
    );
  });
}

async function main() {
  if (!selfTest && !existsSync(path.join(DIST, 'index.html'))) {
    throw new Error('dist/index.html is missing; run `trunk build --release` first');
  }
  const chrome = findChrome();
  if (!chrome) {
    throw new Error('no Chromium binary found; set CHROME_BIN or install google-chrome');
  }

  const { server, url } = await startServer(selfTest ? SELF_TEST_HTML : null);
  const profile = await mkdtemp(path.join(os.tmpdir(), 'r7crypto-smoke-'));
  let browser;
  let socket;
  const errors = [];

  try {
    browser = spawn(
      chrome,
      [
        '--headless=new',
        '--no-first-run',
        '--no-default-browser-check',
        '--disable-dev-shm-usage',
        '--enable-unsafe-swiftshader',
        '--use-angle=swiftshader',
        `--user-data-dir=${profile}`,
        '--remote-debugging-port=0',
        'about:blank',
      ],
      { stdio: ['ignore', 'ignore', 'pipe'] },
    );
    let chromeStderr = '';
    browser.stderr.on('data', (chunk) => {
      chromeStderr += chunk.toString();
    });

    const devtoolsPort = await waitFor(async () => {
      try {
        const text = await readFile(path.join(profile, 'DevToolsActivePort'), 'utf8');
        return Number.parseInt(text.split('\n')[0], 10);
      } catch {
        return null;
      }
    }, 15_000, 'Chromium DevTools port');

    const targets = await waitFor(async () => {
      try {
        const response = await fetch(`http://127.0.0.1:${devtoolsPort}/json/list`);
        const list = await response.json();
        return list.find((target) => target.type === 'page');
      } catch {
        return null;
      }
    }, 15_000, 'a Chromium page target');

    socket = new WebSocket(targets.webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {
      socket.addEventListener('open', resolve, { once: true });
      socket.addEventListener('error', () => reject(new Error('CDP WebSocket failed')), {
        once: true,
      });
    });

    let nextId = 0;
    const pending = new Map();
    socket.addEventListener('message', (event) => {
      const message = JSON.parse(event.data);
      if (message.id && pending.has(message.id)) {
        const { resolve, reject } = pending.get(message.id);
        pending.delete(message.id);
        if (message.error) reject(new Error(message.error.message));
        else resolve(message.result);
        return;
      }
      if (message.method === 'Runtime.exceptionThrown') {
        const details = message.params.exceptionDetails;
        errors.push(
          `uncaught exception: ${details.exception?.description ?? details.text}`,
        );
      }
      if (message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error') {
        const text = message.params.args
          .map((arg) => arg.value ?? arg.description ?? '')
          .join(' ');
        errors.push(`console.error: ${text}`);
      }
      if (message.method === 'Log.entryAdded' && message.params.entry.level === 'error') {
        errors.push(`log error: ${message.params.entry.text}`);
      }
    });

    const send = (method, params = {}) =>
      new Promise((resolve, reject) => {
        const id = ++nextId;
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      });

    await send('Runtime.enable');
    await send('Log.enable');
    await send('Page.enable');
    await send('Page.navigate', { url });

    // Give the module, wasm-bindgen init, WebGL context and several frames time
    // to run. A load-only smoke test covers boot failures; the compile-time
    // guard covers host-only APIs used later by a button press.
    await sleep(4_000);

    if (selfTest) {
      const detected = errors.some((error) => error.includes('SMOKE_SELF_TEST'));
      if (detected) {
        console.log('Smoke detector self-test passed (synthetic console error was caught)');
      } else {
        console.error('Smoke detector self-test FAILED: synthetic console error was missed');
        process.exitCode = 1;
      }
    } else {
      const probe = await send('Runtime.evaluate', {
        expression: `(() => {
          const canvas = document.getElementById('r7crypto_canvas');
          return {
            title: document.title,
            canvas: Boolean(canvas),
            width: canvas ? canvas.width : 0,
            height: canvas ? canvas.height : 0,
            // wasm-bindgen init must have completed and eframe must own a live
            // WebGL context; a canvas that is merely present proves very little.
            bindings: Boolean(window.wasmBindings),
            gl: Boolean(canvas && (canvas.getContext('webgl2') || canvas.getContext('webgl'))),
          };
        })()`,
        returnByValue: true,
      });
      const state = probe.result.value;
      if (state.title !== 'r7 Crypto Lab') errors.push(`unexpected title: ${state.title}`);
      if (!state.canvas || state.width === 0 || state.height === 0) {
        errors.push(`canvas missing or not sized: ${JSON.stringify(state)}`);
      }
      if (!state.bindings) errors.push('wasm-bindgen module did not initialize');
      if (!state.gl) errors.push('no WebGL context on the canvas');

      if (errors.length > 0) {
        console.error('Browser smoke test failed:');
        for (const error of errors) console.error(`  - ${error}`);
        if (chromeStderr.trim()) console.error(`Chromium stderr:\n${chromeStderr.trim()}`);
        process.exitCode = 1;
      } else {
        console.log(`Browser smoke test passed: ${url} (canvas ${state.width}x${state.height})`);
      }
    }
  } finally {
    if (socket && socket.readyState === WebSocket.OPEN) socket.close();
    if (browser && !browser.killed) browser.kill('SIGTERM');
    if (browser) {
      await Promise.race([
        new Promise((resolve) => browser.once('exit', resolve)),
        sleep(3_000),
      ]);
    }
    server.close();
    // Chromium may still be flushing its profile after exit; retry the cleanup
    // instead of turning a passing smoke test into a failing one.
    await rm(profile, {
      recursive: true,
      force: true,
      maxRetries: 8,
      retryDelay: 250,
    });
  }
}

main().catch((error) => {
  console.error(`Browser smoke test error: ${error.message}`);
  process.exitCode = 1;
});
