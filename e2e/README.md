# HTML browser contracts

From this directory, run `npm ci`,
`npx playwright install --with-deps chromium`, then `npm test`.
Node 22 and Rust stable are used in CI. Playwright is pinned in the lockfile;
its downloaded browsers are development tools, not gy runtime dependencies.
The supported test target is Chromium. Firefox and Safari are outside the current
support scope.

`fixtures.mjs` builds the local CLI, writes synthetic Markdown ledgers, and uses
that CLI to generate offline HTML. It checks embedded node and edge counts so an
empty fixture cannot pass. No private or real ledger is checked into this suite.
The 1000-node fixture contains disjoint 15-node chains across six scopes. The
1000-node star also exercises a capped focused view with 940 omitted nodes. The
high-degree fixture has 65 nodes, 64 edges, and a center with 61 immediate
neighbors; an unequal-degree neighbor tests the degree tie-breaker.

The reading fixture has 24 nodes and one edge. It covers all six types,
requirement states (including an unfamiliar value and a status supplement),
closure and satisfaction variants, reverse-only supersession, matching/missing
body marks, Japanese/Latin paragraphs, URL values, and false/zero/null/nested
additional attributes. Width tests cover 1280, 1440, 1920, and 2560 CSS pixels.
The Markdown reading case measures three header cells and six data cells,
left/center/right alignment, a three-level mixed list, a multi-line code block
including a tab, inline code, emphasis, and safe links. Raw HTML and unsafe URL
schemes remain inert.
At 1400, 1920, and 2560, DOM Range measurements count actual wrapped characters
in long applicability/body paragraphs: Japanese 30–45 and Latin 45–90, excluding
final lines from the lower bound. These are actual glyph positions after padding,
not estimates from outer pane widths. Headings, code, and short paragraphs do not
have a minimum line length. JSON measurements and screenshots are attached.

Toolbar checks cover 1280, 1440, and 1920 CSS pixels: one control height,
one row at the two wider sizes, a single state region, and no rectangle
intersection between the legend and graph nodes. D-44 / AC-31 change the
previous verbose button labels and move graph controls out of the filter strip;
existing assertions now inspect both groups and the consolidated status.
The 1440-pixel default and focused screenshots are retained for visual review.

List checks cover all six columns, full wrapping titles with at least 30 actual
full-width characters per complete line at 1440px, row-wide native-link clicks,
shared filter counts, column-order restoration, and 1000-row initial rendering.
D-40 / AC-27 replace cluster-panel assertions with scope/type and table-row
assertions. The omitted-node control now reveals an actual omitted row in that
same table; the test verifies its ID against the focused node's neighbors.

The list reserves vertical space, so a restored tall viewport may still have
off-screen nodes. The resize test compares its measured original count with
the restored count and requires the shorter viewport to hide more nodes. Legend
checks use the SVG clipping rectangle and painted node rectangles; off-canvas
DOM bounds are not visible overlaps. The legend must be outside the entire SVG.

Tests re-resolve each locator and read its DOM `getBoundingClientRect` after
scrolling, then send browser
mouse input. No element click is dispatched from page JavaScript. The cluster
label test clicks the label's physical position, exercising pointer hit testing.
Read-only page evaluation measures the resulting DOM. Text filling uses the
browser automation input API.

Two tests run the **same assertions** used by normal tests against injected
faults and require those assertions to reject for the specified reason:

- A runtime CSS rule restores label interception (`pointer-events:auto`).
- A generated test copy allows the initial fit but omits subsequent fits. The
  unique replacement anchor is checked before execution. After actual zoom and
  pan input, entering a focus must restore readable zoom and a visible node.
  The mutant fails the readable-zoom assertion and leaves that node off-screen.

An unexpectedly successful contract makes the injection test fail. Browser or
JavaScript errors cannot count as successful detection. The production template
and canonical generated fixtures are never modified by injection.

Every test checks browser errors and attempted HTTP(S) requests, including the
URL-bearing reading fixture. Links are inspected without navigating away; no
request may be attempted while loading or operating the page. This runtime check
complements Rust's coarse literal-reference scan; URL text itself is valid. CI runs the
suite with one worker and no retries, and retains its JSON report, failed-test
traces, high-degree screenshots, and generation measurements as an artifact.

The performance tests check 1000-node overview and star-shaped focused views,
and a 3000-node star for a second scale point. Focus entry includes candidate
filtering, ranking, and layout. Every sample requires a changed transform.
After four warmups, 20 alternating pan/wheel samples produce two timing series:

- Mouse command through transform assertion and a subsequent animation frame,
  including driver, IPC, and assertion waiting.
- A browser capture-phase input observer through the next animation-frame
  callback, excluding that automation round trip. Each input must yield exactly
  one sample so an empty observer cannot pass.

Reports include raw samples, median, p95, maximum, browser/OS/CPU metadata, and
focus-entry time. Neither series measures physical display latency. CI checks
10 seconds for initial navigation and 100 ms p95 for the browser-side series.
The automation round trip remains in the report for diagnosis, without a latency
gate: shared-runner driver/IPC/assertion overhead is not product response time.
Performance tests have a 120-second whole-test watchdog for their 24 input
round trips; other tests retain 30 seconds. This bounds stalled tests, not UI
latency. Cluster assertions use the same 3-second wait as other assertions,
including when the injected defect is expected to prevent opening. Actual values
and measurement conditions
should accompany these regression limits; they do not promise performance for
every device or topology.

Force layout is capped at 60 nodes and cached by the drawn ID set. Overflow
selection is ranked with BFS distance, degree, and ID, caching its latest result.
Pan/zoom therefore does not repeat ranking or force iterations when the set is
unchanged. Toolbar predicates are cached until their values change. Neighborhood traversal,
row-ID cache checks, and edge selection still scan ledger data; at most 60 nodes are rebuilt in the SVG DOM. Similar timing at
two scales does not establish scale-independent cost.

## Viewing generated HTML by hand

A browser driven from the terminal is for looking at the current output; the
suite above is the reproducible check. Reuse one tab and reload it after
regenerating a file, keep at most two tabs for a comparison, close tabs when
done, and shut down the browser you opened at the end. A large ledger's HTML is heavy per
tab, and stacked tabs stop responding.

Take element coordinates from the DOM's `getBoundingClientRect`, not from the
driver's bounding box: on WebKit the two disagreed (x873/y546 versus x570/y306
for the same SVG element). Confirm clicks with real mouse input (move, down,
up), since a dispatched click skips hit testing and hides an overlapping element
that steals the pointer. Check that an action succeeded and, separately, that
its result is visible to a reader. Verify the target of a result, not only that
a result occurred: an opened panel is confirmed by its identifier and counts,
which is the only way a coordinate mix-up is caught. When a metric says a
condition holds, confirm the metric can fail: zero overlapping labels means
nothing when the labels are too small to read. A fault-injection test that does
not fail after injection may be missing its target rather than proving the
product correct.

N-27 action coverage checks Overview populations and lint severity, requirement-state
links, URL restoration of the typed next source, and pointer elements against
native links or registered role handlers. Focus assertions use the single graph
navigation action after selecting a record; the duplicate detail action is removed.
The 1440px screenshot includes Overview, the table, and selected type chips.
