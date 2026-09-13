// Node's isolated VM module fixtures use the same source modules as the browser.
import { readdir, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const root = resolve(import.meta.dir, '../src');
const transpiler = new Bun.Transpiler({loader:'ts', target:'browser'});
const sources = {};
for (const file of await readdir(root, {recursive:true})) {
  if (file.endsWith('.ts')) sources[resolve(root,file)] = transpiler.transformSync(await readFile(resolve(root,file), 'utf8'));
}
process.stdout.write(JSON.stringify(sources));
