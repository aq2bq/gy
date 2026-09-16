/* The guard (n-f227): a spec points at the screen, never at a class or an id.
   It reads the other spec files, so a new scene that slips back to a bare
   selector fails here. The dictionary's guard (crates/gy-serve/tests/assets.rs)
   is the same idea: ask the assets, not the memory of the person writing. */
import { expect, test } from '@playwright/test';
import { readFileSync, readdirSync } from 'node:fs';
import { basename, join } from 'node:path';

test('no spec points at a bare class or id', () => {
  const found: string[] = [];
  for (const name of readdirSync(__dirname)) {
    if (!name.endsWith('.spec.ts') || name === basename(__filename)) continue;
    readFileSync(join(__dirname, name), 'utf8')
      .split('\n')
      .forEach((line, index) => {
        // A locator whose string names a class or an id anywhere inside it.
        if (/locator\(\s*['"][^'"]*[.#]/.test(line)) found.push(`${name}:${index + 1}: ${line.trim()}`);
      });
  }
  expect(found).toEqual([]);
});