import { expect, test } from '@playwright/test';
import { execFileSync, spawn, type ChildProcess } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

/** The release binary the CI builds once (cargo build --release -p gy). */
const GY = resolve(__dirname, '..', '..', 'target', 'release', 'gy');

/** The port line the server prints, as the base URL. */
function address(server: ChildProcess): Promise<string> {
  return new Promise((done, fail) => {
    const timer = setTimeout(() => fail(new Error('gy serve printed no URL')), 15000);
    let text = '';
    server.stdout?.on('data', (chunk: Buffer) => {
      text += chunk.toString();
      const found = text.match(/http:\/\/127\.0\.0\.1:\d+\//);
      if (found) {
        clearTimeout(timer);
        done(found[0]);
      }
    });
    server.on('error', fail);
  });
}

/// A shared copy whose remote is gone: the write stays in the copy and the
/// page says so (n-94bb, ac-e63f).
test('a write with an unreachable remote is marked not pushed', async ({ page }) => {
  const dir = mkdtempSync(join(tmpdir(), 'gy-e2e-offline-'));
  const data = join(dir, 'data');
  mkdirSync(data);
  const remote = join(dir, 'remote.git');
  const env = {
    ...process.env,
    GY_ACTOR: 'e2e',
    XDG_DATA_HOME: data,
    GIT_AUTHOR_NAME: 'e2e',
    GIT_AUTHOR_EMAIL: 'e2e@example.com',
    GIT_COMMITTER_NAME: 'e2e',
    GIT_COMMITTER_EMAIL: 'e2e@example.com',
  };
  execFileSync('git', ['init', '-q', '--bare', '-b', 'main', remote]);
  writeFileSync(join(dir, 'gy.toml'), `remote = "file://${remote}"\n\n[scopes.a]\n`);
  const gy = (args: string[]) =>
    execFileSync(GY, ['-C', dir, ...args, '--json'], { env, encoding: 'utf8' });

  gy(['--scope', 'a', 'criterion', 'add', 'the first']);
  gy(['sync']);
  rmSync(remote, { recursive: true, force: true });
  gy(['--scope', 'a', 'criterion', 'add', 'the offline one']);

  const server = spawn(GY, ['-C', dir, 'serve'], { env });
  try {
    await page.goto(await address(server));
    await expect(page.getByTestId('main')).toContainText('not pushed', { timeout: 20000 });
  } finally {
    server.kill();
  }
});