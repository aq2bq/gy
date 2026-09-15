import { execFileSync, spawn, type ChildProcess } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

/** The release binary the CI builds once (cargo build --release -p gy). */
const GY = resolve(__dirname, '..', '..', 'target', 'release', 'gy');

export type Ledger = {
  /** The temporary repository root (gy.toml, and the ledger under data). */
  dir: string;
  /** The server's base URL, with a trailing slash. */
  url: string;
  /** One CLI write against this ledger; answers stdout. */
  gy: (args: string[]) => string;
  stop: () => void;
};

const run = (dir: string, data: string, args: string[], json = true) =>
  execFileSync(GY, ['-C', dir, ...args, ...(json ? ['--json'] : [])], {
    env: { ...process.env, GY_ACTOR: 'e2e', XDG_DATA_HOME: data },
    encoding: 'utf8',
  });

/** A ledger built inside the test: two scopes, the kinds the pages need, and
 *  a server on its own port (the brief's fixture: 3 needs with one closed,
 *  3 questions, 2 decisions with one closing a question, 4 criteria with one
 *  satisfied, 1 filed requirement). */
export async function start(options: { large?: boolean } = {}): Promise<Ledger> {
  const dir = mkdtempSync(join(tmpdir(), 'gy-e2e-'));
  const data = join(dir, 'data');
  mkdirSync(data);
  writeFileSync(join(dir, 'gy.toml'), 'output = "pub"\n\n[scopes.a]\n[scopes.b]\n');
  const gy = (args: string[]) => run(dir, data, args);
  if (options.large) {
    buildLarge(gy);
  } else {
    build(gy);
  }
  const server = spawn(GY, ['-C', dir, 'serve'], {
    env: { ...process.env, GY_ACTOR: 'e2e', XDG_DATA_HOME: data },
  });
  const url = await address(server);
  return { dir, url, gy, stop: () => server.kill() };
}

/** The one small ledger every G1 scene shares. */
function build(gy: (args: string[]) => string) {
  const id = (args: string[]) => JSON.parse(gy(args)).id as string;
  const a = ['--scope', 'a'];
  const b = ['--scope', 'b'];
  const one = id([...a, 'criterion', 'add', 'measures one']);
  const two = id([...a, 'criterion', 'add', 'measures two']);
  const three = id([...b, 'criterion', 'add', 'measures three']);
  const four = id([...a, 'criterion', 'add', 'measures four']);
  gy([...a, 'criterion', 'satisfy', one, '--evidence', 'measured']);
  const first = id([...a, 'need', 'add', 'the first need', '--targets', one]);
  id([...a, 'need', 'add', 'the second need', '--targets', two]);
  const closed = id([...b, 'need', 'add', 'the closed need', '--targets', three]);
  gy([...b, 'need', 'close', closed, '--by', 'fact', '--evidence', 'not needed']);
  id([...a, 'question', 'add', 'the master question', '--decider', 'master', '--options', 'one', '--options', 'two']);
  const answered = id([...a, 'question', 'add', 'the answered question', '--decider', 'master', '--options', 'one', '--options', 'two']);
  id([...a, 'question', 'add', 'the lead question', '--decider', 'lead', '--options', 'one', '--options', 'two']);
  id([...a, 'decide', 'closes the answered question', '--scope-note', 'the master said so', '--closes', answered]);
  id([...b, 'decide', 'the second decision', '--scope-note', 'another call']);
  id([...a, 'req', 'add', 'the filed requirement', '--need', first]);
}

/** The bigger ledger: 40 needs, 60 decisions, 20 questions (loop). */
function buildLarge(gy: (args: string[]) => string) {
  const one = JSON.parse(gy(['--scope', 'a', 'criterion', 'add', 'the big criterion'])).id as string;
  for (let index = 0; index < 40; index++) {
    gy(['--scope', 'a', 'need', 'add', `need ${index}`, '--targets', one]);
  }
  for (let index = 0; index < 60; index++) {
    gy(['--scope', 'b', 'decide', `decision ${index}`, '--scope-note', 'a call']);
  }
  for (let index = 0; index < 20; index++) {
    gy(['--scope', 'a', 'question', 'add', `question ${index}`, '--decider', 'lead', '--options', 'one', '--options', 'two']);
  }
}

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
