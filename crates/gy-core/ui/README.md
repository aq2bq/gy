# HTML projection sources

Use Bun **1.4.0** (also pinned in `package.json` and the HTML CI job):

```sh
cd crates/gy-core/ui
bun install
bun run build
```

There are no package dependencies. With none to resolve, `bun install` does
not retain an empty lockfile. Build produces exactly two unminified files:
`../src/html/dist/app.js` and `app.css`. Commit both with their source changes.
`html.rs` embeds them and the templates under `src/templates/`, so Cargo users
can build/package gy and generate offline HTML without Bun or Node.js.
Do not edit dist directly.

After staging the regenerated files, check reproducibility:

```sh
bun run check
```

This rebuilds and runs `git diff --exit-code -- ../src/html/dist`. Locally it
compares against the index; CI compares against the clean checkout. A source
edit without its regenerated artifact fails this check. Building uses the
pinned Bun version without minification, source maps, external imports, or
runtime packages. Bun transpiles TypeScript; it does not perform full semantic
type checking.

## Source responsibilities

| Module | Responsibility |
| --- | --- |
| `data.ts` | Embedded payload, node index, immutable input facts |
| `state.ts` | Display-state types, mutation functions, focus traversal, hash codec |
| `filters.ts`, `filters/query.ts` | Filter controls and shared cached predicates |
| `navigation.ts`, `location.ts` | Focus path and URL-to-UI projection |
| `list.ts` | Permanent sortable table, complete titles, row-wide links |
| `detail/` | Record details and the existing offline Markdown renderer |
| `graph/` | Layout, measured label bounds, SVG, selection, drawing, clusters, viewport |
| `survey/` | Overview, Progress, Blockers |
| `tokens.css`, `tokens.ts` | Shared CSS tokens and their SVG/chart palette |
| `dom.ts`, `components.ts` | Typed template lookups and shared rendering helpers |
| `main.ts` | Explicit initialization and browser event wiring |

Modules import their shared values. Display-state changes go through `state`
functions, including filters, focus history, selection, and viewport changes.
Private render caches are derived from those values. The template still writes
`window.GY_DATA` once; application modules only read this payload boundary and
do not export state onto `window`.

From the repository root, run the isolated module regression tests:

```sh
node --experimental-vm-modules --test tests/html_navigation.test.cjs
```

The fixture uses Bun to strip types and Node VM modules to link the real state,
selection, navigation, and viewport modules with DOM/drawing stubs. Browser hit
testing, URL history, layout, Markdown, and offline behavior remain covered by
the Playwright suite (`cd e2e && npm run test:all`). Local `npm test` runs the
functional checks; `npm run test:perf` runs the performance gates alone.
