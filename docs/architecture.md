# Ledger semantics and derived views

## Authority

The canonical node file contains two different kinds of information:

- Frontmatter declares node identity, lifecycle state, relationships, and structured records. Core rules and configured workflow schemas define the meaning of those declarations.
- The Markdown body explains decisions, supplies context, and quotes evidence or code. Its vocabulary does not declare graph relationships, lifecycle states, or counts.

A consistency check evaluates declared records. It must not silently substitute a guess from prose when a declaration is absent. In particular, `waiting_checkout?`, a sentence saying a question is unresolved, and a quotation of an old report cannot create a `waiting-on` edge. Missing declarations cannot be detected by pretending that a prose inference is a declaration.

Body search remains available through `find`. Import may explicitly extract a configured section into an attribute; the import result is persisted and reported before later checks read it. `mark` is an explicit locator into body text: display and link warnings may inspect the body to locate it, without inferring a new relationship. Compression archives original records and generates its summary from declared attributes.

Unknown extension attributes are preserved. The existing L13 convention treats exact node-ID-valued attributes as references; it does not extract references from arbitrary prose strings. Workflow snapshot schemas are configuration, while their recorded values and evidence remain data.

## Scope relabeling

A scope is a label, not a node identity. Its canonical representations are the directory name, each member node's `scope` attribute, and the `[scopes]` entry in `gy.toml`; loading requires the directory and attribute to agree. `scope rename` changes all three together, so node IDs, relationships, lifecycle state, records, and history are preserved. Relationships are keyed by node ID and never embed a scope name, so cross-scope edges survive unchanged.

Renaming rejects a destination that already exists or a name that is not a safe directory component. A collision is not a merge. Free text, `decision_scope`, and arbitrary attributes that mention the old name are author declarations rather than scope references, so rename does not rewrite them; later users verify any such reference. The operation applies the new files and the old directory removal in one transaction, so an interrupted update replays to a consistent state.

## Dependency meaning and lifecycle

`requirement --relies-on--> decision` records the decision on which that requirement's work was based. The edge remains part of the ledger after completion or supersession. `supersedes` declares that a decision has replaced another decision; it does not retroactively replace the decisions used by earlier work.

The role of a requirement's decision dependencies is derived from its declared lifecycle:

| Requirement state | Dependency role | Superseded target |
| --- | --- | --- |
| Any recognized state except `complete` | Current work dependency | L5 reports the conflict; `next` does not offer work blocked by it |
| `complete`, including parenthesized context | Historical basis of completed work | Preserve and display it as history, not an L5 conflict |
| Missing or invalid state | Treated as current for dependency checks | L10 also reports the invalid state; malformed state cannot exempt a dependency |

`Node::is_complete_requirement` defines the completion boundary. `Store::decision_dependencies` is the shared projection of recorded edges into current or historical dependencies and their recorded superseding decisions. Lint, scheduling, display, and handover consume this projection. Neither role nor an acknowledgement flag is written back to the node.

`next` stops traversing work prerequisites when it reaches a completed requirement. Historical decisions and questions from finished work must not become new prerequisites of later work. This does not waive structural checks: missing targets, inverse-link inconsistencies, explicit unresolved references, and invalid completion records remain independently diagnosable.

Reopening a requirement changes its dependency role back to current without editing its edges. Compression preserves the edges and completion state, so it preserves their historical role. Updating a live product after earlier work has completed belongs to another active requirement, or to an explicitly reopened requirement; a completed task is not a declaration that its resulting behavior must remain in production forever.

This classification does not establish that a decision was valid at the time of completion, that an implementation changed, or that someone reviewed a supersession. The ledger cannot reconstruct unrecorded past events from today's state. Historical display is not a verification claim.

## Workflow policy over time

A project's current workflow configuration governs ongoing work and new actions. Introducing or changing a profile does not establish obligations for already completed work. Lint inspects completed requirements' recorded workflow snapshots using the schemas and comparisons saved with those snapshots. It does not demand new records from the current profile, including records marked `required`.

A new transition always evaluates the current destination guards, including reopening completed work and explicitly entering `complete` again. Explicit record submission also uses the current schema. This distinction is represented by separate passive inspection and current-action validation paths, rather than a per-node exemption flag, import marker, timestamp guess, or compression status.

Compression is an archival operation, not a work transition. It validates existing historical snapshots and core completion/archive requirements, then preserves the original data. Completed work without workflow snapshots can be archived without constructing a fictional history. Existing snapshots remain subject to their own recorded schemas and checks even if the current profile changes or is removed. Neither absence nor presence of a snapshot proves external approval or correct implementation.

## Rule contracts

| Consumer | Authoritative input | Result |
| --- | --- | --- |
| L1 and question-close warnings | `waiting-on` / `unresolved` references and the target question's `status` | An explicit unresolved reference conflicts with a closed question |
| L2 | Optional nonnegative integer `bearer_count` and needs' `targets` edges | A declared count must have the right type and equal the supporting-need count |
| L5 | Current requirement decision dependencies and recorded `supersedes` / `superseded-by` edges | Work still to be done depends on a replaced decision |
| Workflow lint on completed requirements | Saved workflow inputs, schemas, and comparisons | Check historical records without retroactive application of the current profile |
| New transitions and submissions | Current workflow configuration and supplied records | Validate the action now, regardless of the node's previous completion state |
| L10 / L11 / L13 / `edges` | State, completion records, references, and edge declarations | Structural integrity continues to apply to completed work |
| `show` / Markdown `render` | The dependency projection | Label superseded dependencies as current or historical without modifying canonical files |
| `handover` | The same projection and lint results | Separate `historical_superseded_dependencies` from actionable diagnostics |

Handover's historical entries and show's `decision_dependencies` contain `requirement`, `decision`, `role`, and `superseded_by`. They are derived output, not replacement frontmatter. Historical entries do not affect handover's exit code. Recorded successors are collected from both edge directions so a broken inverse link cannot silently erase supersession; the `edges` rule still reports that structural defect.

CLI and MCP invoke the same core operations. New checks should define their authoritative fields, relationship meaning, lifecycle applicability, and consuming views before implementation. Verification should test those contracts across consumers, including changes that must leave results invariant, rather than only a reported example.

## Compatibility and rejected approaches

L1 no longer interprets body keywords, and L2 no longer reads a prose `bearers 3` count. Users who intend those assertions must explicitly record `waiting-on`, `unresolved`, or `bearer_count`. No automated rewrite can reliably distinguish an assertion from a quote, code identifier, or negated sentence, so this change does not fabricate attributes during migration. Use `find` to review prose and write declarations based on evidence.

A word-boundary adjustment would retain two competing sources of authority. A separate heuristic review feature could be designed later with explicit provenance and separate results, but it does not belong in these integrity rules.

A per-node L5 acknowledgement would obscure whether a dependency is historical or current. Treating every completed node as exempt from all lint would instead hide broken history. The lifecycle projection avoids both: it preserves recorded relationships and applies each rule according to the meaning of its assertion.

The regression contract covers prose-edit invariance, structured assertions overriding prose, completion and reopening, scoped historical output, scheduling through completed work, and persistence of structural errors. Existing compression, workflow, and MCP tests verify the shared storage and execution paths.

## HTML projection

`render --format html` writes a single self-contained HTML file for reading the whole ledger. The supported browser for this local projection is Chromium. It is a derived view: the page never computes a lint, scheduling, dependency, or handover judgment itself. It embeds diagnostics from `lint`, needs from `next`, handover facts (missing records, dangling references), the `stats` criteria and question-arrival numbers, and the `decision_dependencies` projection, all produced by the same core functions the CLI and MCP use. Filters and full-text search select the embedded data at display time; the graph routes forward edges exactly as `dot` does, so no relationship is drawn twice through its reverse label.

Bodies are embedded verbatim so the page carries the ledger's full record. The graph never renders body text; the detail panel shows it, rendered as Markdown on demand. Escaping replaces `<`, `>`, `&`, U+2028, and U+2029 in the embedded payload so a body containing `</script>`, a comment opener, or a line separator cannot terminate or comment out the data script. Unknown attributes are preserved in node payloads alongside their structured values.

Judgments are the only reason the page carries curated lists; everything else is the ledger's own data. This mirrors the `handover` boundary: derived output is never written back to canonical files, and the page cannot edit the ledger.

The graph renders each of the six node types with a distinct shape and color, so type identification does not depend on color alone. Genealogy mode restricts the displayed set to decision nodes and lays them out by generation, so a supersede chain reads directionally.

Drawing is split into two views by the number of visible nodes, not by zoom level. Without a focus, more than 60 visible nodes are drawn as clusters (scope × type) and aggregated edge counts per cluster pair, plus internal edge counts per cluster; individual nodes and edges are withheld so the overview stays readable. Clicking a cluster opens a member list from which a start node is chosen, and the neighborhood hop count is then computed adaptively: the largest value up to a ceiling whose reachable set fits the individual-view limit, displayed on screen. With the visible set at or below the limit, nodes are drawn individually in a force-directed layout that reflects connectivity, so a short edge means adjacent nodes, and node labels are measured (via `getComputedTextLength`), truncated to the available width, and skipped when they would overlap a neighbour. Edge labels are off by default and appear only for a small individual set or an explicitly selected edge; full titles always remain in the detail panel.

The count banner reports the nodes actually drawn and the visible total in every mode, including search, filters, and genealogy; culling always surfaces how many are hidden rather than silently dropping them. Fit computes the transform from the content's bounding box in the current mode, so the projected content fits the viewport instead of being a fixed scale.

### Focus navigation and viewport ownership

The page stores an in-memory stack of `{id, radius}` entries beginning with
`{id: null, radius: null}`. Entering a node chooses the adaptive radius from the
undirected EDGES adjacency and stores it; repeated entry to the current node
leaves the stack unchanged. Back removes one entry; choosing a path entry removes
all entries after it. The stack records the navigation route, not graph ancestry.
Reloading the generated HTML starts with no focus.

Type, scope, lifecycle filters, full-text search, and genealogy are independent
of that stack. They intersect the neighborhood selection. A filtered-out focus
is retained and explicitly reported in the navigation bar. An empty adjacency
for the focus produces “No connections in this graph”; this makes no claim
about references excluded from EDGES or connections outside the embedded graph. All nodes removes
only the focus; Reset everything also clears filters, search, and genealogy.
Genealogy continues to draw only its four relationship types among displayed
decisions, using the generation layout.

Node clicks, cluster member choices, and ID entry share the same navigation
operation: update the focus, open details, then draw once. Details occupy a
separate region beside or below the graph and can be closed without changing the
focus. Their visibility is not part of navigation history.

Wide layouts give the graph and details equal shares of the available width;
at 1100 CSS pixels or below, details move beneath the graph. Type, scope,
requirement state, criterion satisfaction, question closure, and decision
supersession use labeled badges. Requirement state and supersession come from
core projections; status supplements and unfamiliar values are not normalized
away. Applicability and body are separate reading sections; other declarations
use labels, comparable dependency fields use a table, and additional attributes
retain full structured values, including false, zero, and null. Lineage relations
and other relations have separate headings, preserving targets and marks.

The private HTML payload includes exact body-mark locations derived from the same
reverse declarations and `edge_mark` lookup as the core display path. Matching
marks annotate the corresponding passage for display without modifying the raw
body. Missing or unlocated marks remain in a separate affected-passages section;
they are never assigned to another passage.

Reading text uses 16px type and a 1.75 line height. Long paragraphs are tested
at viewport widths 1400, 1920, and 2560 CSS pixels with DOM Range rectangles:
Japanese full lines contain 30–45 fullwidth characters, Latin full lines 45–90.
These measurements exclude padding and final lines; short paragraphs, headings,
and code are not subject to the lower bound. Narrow screens necessarily wrap
more tightly. The synthetic paragraphs test actual wrapping, not a width-to-font
estimate or a claim that every mixed-script paragraph has identical line lengths.

Self-containment prohibits external resource fetches during loading and page
interaction, not URL strings in data or explicit user-followed HTTP(S) links.
Rust checks literal resource-reference tokens as a coarse guard. Chromium E2E
observes attempted HTTP(S) requests, including when URL-bearing details render;
zero requests is the runtime contract.

The viewport tracks whether its transform comes from fit or manual zoom/pan.
A focus, displayed-node set, or genealogy change always refits. A size-only
change refits an automatic view, while a manual view keeps its scale and the
world coordinate at the viewport center. The draw operation observes the final
focus and pane size, so simultaneous changes cause one fit. When a readable fit
cannot contain all selected nodes, it centers the focus if that node belongs to
the selected set; otherwise it centers the selected nodes' bounding box. Both
ordinary and genealogy views report off-screen nodes. Automatic individual
fit is limited to scale 1–2; manual zoom retains its existing 0.1–4 range. Content
that does not fit at readable scale remains reachable by pan/zoom.

Adjacency is built once in O(V + E). Choosing a radius traverses at most five
bounded neighborhoods, each O(V + E) in the worst case. Returning uses the stored
radius without recomputing its choice. History uses O(H) space; rebuilding the
path is O(H) only when it changes. Existing filtering, layout, and drawing still
run over their selected nodes; navigation does not scan ledger history.

When a focus has more than 60 filtered neighborhood candidates even at one hop,
individual drawing continues with at most 60 nodes. Selection orders candidates
by distance from the focus, then descending degree in the embedded undirected
edge graph, then ID. The focus is included if it passes the filters; it is never
restored against a filter. Omitted candidates have a separate count and member
list for navigation. They differ from drawn nodes outside the viewport, which
remain accessible by pan or zoom. The limit applies to focused lineage as well;
unfocused lineage retains its generation layout.

### HTML source assembly

`gy-core/src/html.rs` assembles the HTML sources with `concat!` and
`include_str!` at compile time, then replaces the payload placeholder with
escaped JSON. The ordered fragments in `gy-core/src/html/` contain the HTML
shell, CSS, and JavaScript responsibilities: data, summary, controls, state and
neighborhood traversal, layout, SVG primitives, selection, drawing, navigation,
cluster lists, viewport, details, blockers, progress and legend, and startup.
All JavaScript fragments share one closure; their order in `TEMPLATE` preserves
initialization order. Fragment boundaries do not add whitespace or script tags.
The generated artifact remains one offline HTML file, and neither building gy
nor generating HTML requires a JavaScript bundler or Node.js.
