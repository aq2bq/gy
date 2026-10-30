import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* The crossing screen's placement is finite by construction (ac-afba, d-6d90):
   a node with five hundred edges gives finite coordinates and radii. The pure
   function is lanes.js; this asks the page that loaded it. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

type Node = { id: string; kind: string; scope: string; state: string; created: number; title: string; degree: number };
type Edge = { from: string; to: string; relation: string };
type Result = { points: number; bad: number; width: number; height: number; minX: number; maxX: number; minY: number; maxY: number };

const place = (page: import('@playwright/test').Page, nodes: Node[], edges: Edge[]) =>
  page.evaluate(
    ({ nodes, edges }) => {
      const lanes = (window as unknown as { GyLanes: { layout: (input: unknown) => unknown } }).GyLanes;
      const core = Object.fromEntries(nodes.map(node => [node.id, true]));
      const lay = lanes.layout({ nodes, edges, width: 900, scopes: ['a'], core }) as {
        points: { x: number; y: number; r: number }[];
        width: number;
        height: number;
      };
      const xs = lay.points.map(point => point.x);
      const ys = lay.points.map(point => point.y);
      const bad = lay.points.filter(point => !Number.isFinite(point.x) || !Number.isFinite(point.y) || !Number.isFinite(point.r)).length;
      return {
        points: lay.points.length,
        bad,
        width: lay.width,
        height: lay.height,
        minX: Math.min(...xs),
        maxX: Math.max(...xs),
        minY: Math.min(...ys),
        maxY: Math.max(...ys),
      } as Result;
    },
    { nodes, edges },
  );

const star = (edges: number, seed = 1): { nodes: Node[]; edges: Edge[] } => {
  const at = Math.floor(Date.parse('2026-10-30T00:00:00Z') / 1000);
  const nodes: Node[] = [{ id: 'n-centre', kind: 'Need', scope: 'a', state: 'open', created: at, title: 'the centre', degree: edges }];
  const links: Edge[] = [];
  let state = seed;
  for (let index = 0; index < edges; index++) {
    state = (state * 16807) % 2147483647;
    nodes.push({ id: `n-${index}`, kind: 'Need', scope: 'a', state: 'open', created: at + (state % 300), title: `leaf ${index}`, degree: 1 });
    links.push({ from: 'n-centre', to: `n-${index}`, relation: 'depends-on' });
  }
  return { nodes, edges: links };
};

test('a node with 50, 200, and 500 edges keeps every coordinate finite', async ({ page }) => {
  await page.goto(`${gy.url}?lang=en`);
  for (const edges of [50, 200, 500]) {
    const set = star(edges);
    const result = await place(page, set.nodes, set.edges);
    expect(result.points).toBe(edges + 1);
    expect(result.bad).toBe(0);
    expect(result.minX).toBeGreaterThanOrEqual(0);
    expect(result.maxX).toBeLessThanOrEqual(result.width);
    expect(result.minY).toBeGreaterThanOrEqual(0);
    expect(result.maxY).toBeLessThanOrEqual(result.height);
    expect(Number.isFinite(result.height)).toBe(true);
  }
});

test('a mixed set with several scopes and times keeps every coordinate finite', async ({ page }) => {
  await page.goto(`${gy.url}?lang=en`);
  let state = 7;
  const random = () => (state = (state * 16807) % 2147483647) / 2147483647;
  const at = Math.floor(Date.parse('2026-09-01T00:00:00Z') / 1000);
  const nodes: Node[] = [];
  const edges: Edge[] = [];
  for (let index = 0; index < 240; index++) {
    nodes.push({
      id: `n-${index}`,
      kind: index % 3 === 0 ? 'Question' : 'Need',
      scope: ['a', 'b', 'c'][index % 3],
      state: index % 4 === 0 ? 'closed' : 'open',
      created: at + Math.floor(random() * 50) * 86400,
      title: `node ${index}`,
      degree: 0,
    });
  }
  for (let index = 1; index < nodes.length; index++) {
    edges.push({ from: nodes[Math.floor(random() * index)].id, to: nodes[index].id, relation: 'depends-on' });
  }
  const result = await place(page, nodes, edges);
  expect(result.points).toBe(nodes.length);
  expect(result.bad).toBe(0);
  expect(Number.isFinite(result.minX)).toBe(true);
  expect(Number.isFinite(result.maxX)).toBe(true);
  expect(Number.isFinite(result.minY)).toBe(true);
  expect(Number.isFinite(result.maxY)).toBe(true);
});

/* The property half of ac-afba: a fixed seed makes the same varied set every
   run — the count of points and edges, the scopes, and the times all differ. */
const varied = (seed: number): { nodes: Node[]; edges: Edge[] } => {
  let state = seed;
  const random = () => (state = (state * 16807) % 2147483647) / 2147483647;
  const count = 30 + Math.floor(random() * 470);
  const scopes = ['a', 'b', 'c', 'd', 'e'].slice(0, 1 + Math.floor(random() * 5));
  const at = Math.floor(Date.parse('2026-06-01T00:00:00Z') / 1000);
  const nodes: Node[] = [];
  const edges: Edge[] = [];
  for (let index = 0; index < count; index++) {
    nodes.push({
      id: `n-${index}`,
      kind: ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'][Math.floor(random() * 5)],
      scope: scopes[Math.floor(random() * scopes.length)],
      state: random() > 0.4 ? 'open' : 'closed',
      created: at + Math.floor(random() * 120) * 86400 + Math.floor(random() * 86400),
      title: `node ${index}`,
      degree: 0,
    });
  }
  const links = Math.floor(random() * count * 2);
  for (let index = 0; index < links; index++) {
    const from = Math.floor(random() * count);
    const to = Math.floor(random() * count);
    edges.push({ from: nodes[from].id, to: nodes[to].id, relation: 'depends-on' });
  }
  return { nodes, edges };
};

test('a seeded varied set stays finite at every size and shape', async ({ page }) => {
  await page.goto(`${gy.url}?lang=en`);
  for (const seed of [3, 11, 29, 101, 977]) {
    const set = varied(seed);
    const result = await place(page, set.nodes, set.edges);
    expect(result.points).toBe(set.nodes.length);
    expect(result.bad).toBe(0);
    expect(Number.isFinite(result.minX)).toBe(true);
    expect(Number.isFinite(result.maxX)).toBe(true);
    expect(Number.isFinite(result.minY)).toBe(true);
    expect(Number.isFinite(result.maxY)).toBe(true);
    expect(result.minX).toBeGreaterThanOrEqual(0);
    expect(result.maxX).toBeLessThanOrEqual(result.width);
    expect(result.maxY).toBeLessThanOrEqual(result.height);
  }
});
