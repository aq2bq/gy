# The design of the new gy

This document states the design contract of the new gy (0.5). The contract of use is in the [README](../README.md), the migration from 0.4 is in [migration-0.5](migration-0.5.md), and the release procedure is in [release](release.md).

## Four layers and the direction of dependency

The crate `gy-ledger` is divided into four layers. `store` (persistence, transactions, ID allocation, history, the format version) → `model` (the five kinds of node and the invariants) → `ops` (operations = intents) → `views` (the projections show / list / next / handover / publish). Dependencies flow in this direction only. `views` does not read `store` directly, and `ops` does not call `views`. The CLI (`gy`) only calls `views` and `ops`.

The responsibilities of each layer are as follows. `store` uses no other layer. `model` uses only `store`. `ops` uses `store` and `model`. `views` uses `model` and `ops`. Inside one layer, references go through `super::`.

## Always valid

An invalid state is refused at the moment of the write, not by a pass that checks later. Every node in the ledger satisfies the invariants of that moment.

What is refused when a node is built: an empty scope, an empty title, a `created` that is not a UTC instant (`YYYY-MM-DDTHH:MM:SSZ`), a decision with an empty scope note (except the unrecorded marker that only the migration attaches), a pair of nodes the relation does not allow, a `narrows` or `supersedes` mark that is not found in the body or the scope note of the older decision, a second registration of the same edge, closing a closed question again, making a `closes` edge with `link`, and an edge to a node that does not exist.

Every write is one transaction. One intent corresponds to one command, and if it fails partway, nothing is written. The history of the change rides in the same transaction: it stays when the transaction commits and is discarded when it fails.

An achievement needs an approved requirement (d-b0d0, d-5dfe). A requirement is the promise of what will change outside the ledger, and approval is the event that the promise was accepted. gy cannot see the outside work, but it can refuse to record its result: a criterion turns satisfied only while a requirement that is approved or done points at it with `targets`. While a requirement stays approved, its title, body, and its `targets` and `relies-on` edges are frozen, and so are the title and body of a criterion it targets; `req revise` sends it back to filed first. Who approves is the user's policy and gy does not judge it. The rule looks only at what a write changes, so what a ledger already holds stays valid.

The rule is one function in the model, and one gate carries it to every path a write can take (n-557f): the commit of an ordinary write, the commit of an undo or redo, and the placing of each pending write when a shared ledger rebases. The store defines the gate in its own terms (the JSON values it holds and the log's changes), the model holds the rule behind a small view of the previous state, and the operations join them, so the store still calls neither model nor operations. A transaction that carries no criterion and no requirement reads no previous state. Replaying history (opening, reading a point in time, publishing) never reaches the gate. The coverage judgement is a single function that the rule and the advice (`missing`, `next`) share.

Every date is an instant. A node's `created`, an acceptance criterion's `satisfied_at` and the `at` of a requirement's record are stored as UTC instants, and the CLI and serve show them as the date and time of the viewer's place (`TZ`, the browser). `--json` and publish emit the stored value as it is. `--since <date>` starts at 0:00 of that day in the viewer's place. The shared word a team uses to point at "when" is not a date but `seq` and the ID (d-b1f8).

## The event log, the snapshot and the format version

The canonical store is an append-only log named `events.jsonl`, and one transaction corresponds to one line. A line has `seq`, `at`, `actor`, `why`, `source` and `changes`; a write that was retried after a conflict has `retries` as an optional field; and each item of `changes` describes the change to one node (`node`, `change`, `value`). `change` is one of `created`, `updated`, `deleted` and `scope-renamed`. `scope-renamed` changes the scope name of several nodes in one line; its `node` is empty and its `value` is `{from, to, nodes}`.

Atomicity comes from an append that writes one line and its newline in a single write, and from `fsync`. A tail that was cut off partway is dropped by a writer, after it takes the exclusive lock and just before it appends. A reader does not change the log (n-6b71). A conflict between writers writing at the same time is detected by the exclusive lock and a comparison of `seq`, and the writer that lost reopens the ledger and runs the same intent again, up to 5 times. If the intent no longer holds, it fails with the reason of that invariant, and after losing 5 times it gives up (n-fe59). The history is the log itself, and `undo` appends a new line that walks the last transaction backwards.

`snapshot.json` is derived and holds the current nodes and `seq`. It only makes opening faster; if it is deleted, the log alone gives the same result. The `format` file holds the format version, and a ledger has this version from the moment it is born. Opening a version this build does not support is an error. A migration that raises the version does not rewrite the log, advances one step at a time, and leaves the file as it was before the change at `<name>.<version>.bak`. In version 3 (0.9.0) the reader reads a date-only value as the instant `T00:00:00Z` and rebuilds the snapshot. Version 4 (1.0.0) changes no data: it exists because an older build does not know the requirement rule and would write around it, so the step only keeps `format.3.bak` and writes the version.

The canonical store is located at `$XDG_DATA_HOME/gy/<hash of the repository root>/`. The only thing placed in the repository is `gy.toml`. `gy init <scope>` writes that file and nothing else, before any root has been resolved and without opening a ledger; the store still appears with the first write (n-29b3, d-0f6d). It searches upward first and reports the root above rather than start a second store under it, because one repository has one canonical store (d-4221). Its output is where a first-time agent learns what to read and type next, since the `gy-loop` skill only fires where a `gy.toml` already exists.

## ID generation and collisions

An ID is made of a prefix for the kind and a short hash. There is no central allocator, so several agents may write at the same time. The seed mixes the prefix, the time in nanoseconds, the process ID, the number of nodes and a salt, and a different seed is hashed for each width. On a collision the ID widens from 4 digits to 6 and then to 8. This is because merely cutting out the low bits of the same 32 bits leaves collisions at the short widths.

The IDs of 0.4 are kept as aliases. `show` resolves a full ID, a zero-padded ID, an alias and a requirement's outward reference, by exact match or by suffix match. A requirement's reference (`ref`) is an opaque value, and gy does not read what it points at.

## Edges are placed on the from side only

A node holds only the edges that start from itself. The reverse direction is not stored; `Repository::incoming` assembles it by scanning all nodes. The same edge is not written in two places.

The relations are a closed set of 12 values, and the reverse names are derived. The allowed pairs of node kinds are gathered in one table, and an edge that is not in the table is refused at the time it is built. Only the forward side of `narrows` and `supersedes` holds a `mark`, and it is carried over when the reverse direction is assembled.

## Derived values

Some values are not stored and are computed from the graph every time.

- The state of a need: `closed` if it is closed, `done` if all of its `filed-as` requirements are `done`, and `open` otherwise.
- Whether it can be started (the condition of `next`): it is `open`, its `depends-on` needs are `closed` or `done`, and what it `waits-on` is settled (a question is closed; a requirement is `done` or `cancelled`).
- `bearer_count`: the number of needs that have that acceptance criterion in their `targets`.
- Requirements in progress: requirements that are `filed` or `approved`.
- A decision's retraction (n-0c6f, d-bde9): which newer decisions `narrow` or `supersede` it, and, from each `mark`, which passage of its body or scope note stops applying. It is derived whether or not `--full` is asked for, from one pass over the graph per invocation, and it is the one judgement every read path shows: `show` and serve render it and decide nothing of their own. A `narrows` wraps the passage where it stands (`[[retracted by <id>: <mark>]]`) so a reader meets it in the text rather than having to connect a line elsewhere; a `supersedes` covers the whole decision and only names itself above the body. A decision whose mark falls outside the `## Decision` section shows its whole body rather than hide the mark. The marking is a projection: the stored body is what `--json` and `publish` carry, and the marked text rides beside it.

## What handover and next judge

`handover` reports the errors (an edge to a node that does not exist, a `filed-as` edge whose target is not a requirement), the requirements in progress (`ref`, state, `next_evidence`, `responsible`), the number of open questions, the number of needs that can be started, and the number of warnings. The warnings are: a requirement in progress that relies on a superseded decision, a requirement in progress that relies on a decision whose scope note is unrecorded, an acceptance criterion with an empty body, an open question that nobody waits on (neither `waits-on` nor `raised`), and an unmet acceptance criterion whose bearing needs are all closed (counts only; d-09b6, d-f7b6). It lists the states that need attention in an order a person can read.

`next` lists the needs that can be started in the order of `created` and ID. It holds no priority. The agent chooses which need to advance and offers it to the person.

## publish is a publication of the record

`publish` writes the record and the diagnostics of a given point and range into a directory per scope under the output directory (d-7c64): one file per node, and one index file for the scope. The output directory is kept outside git's control (in gy's own repository it is excluded by `.gitignore`), and the decisions and how they came about are reviewed later. The content of a ledger is internal development information and is not a publication of gy as a tool. It is used by agents and is not reading matter for the person (d-edb0).

- A node file: `<scope>/<kind>/<ID>-<title>.md`. The ID with its aliases and ref, the title, scope, created, the state, the scope note, the body, the edges in both directions (the other side's ID, alias, title and mark), the closure and the evidence, the requirement's records, and the free attributes. A decision puts its lineage relations first.
- The index: `<scope>/README.md`. The generated time, seq, scope, since, the writer and the canonical store; how to read; a list per kind (links and states); the history; the diagnostics.
- The output directory is `--out` or `output` in `gy.toml` (a directory). Only the directory of the target scope is deleted and rebuilt; nothing else is touched.
- The order is kind → created → ID. Publishing twice from the same ledger gives the same set of files (except the generated time in the index).

## Team sync takes the git remote as the authority (experimental)

When `remote` in `gy.toml` names a git repo dedicated to the ledger, the default branch of that repo becomes the canonical store, and the ledger directory on each machine becomes a copy of it (d-39f6). The authority is not "the one who judges which is right after a conflict" but "the one place that decides the order before a conflict", and this keeps the straight line of seq and the single validation at write time (always valid). There is no merge, and there is no branch.

- A copy is a git working tree whose top level is the ledger directory itself, and it tracks only `events.jsonl`, `format`, `README.md` and `.gitignore`. Even when the ledger directory is inside another working tree, it makes a nested repo, and before a commit and a push it checks that origin matches the recorded remote. If it does not match, it touches nothing (n-6d6c).
- The remote's format is raised by the copy that is ahead of it (d-bace): one commit that changes `format` alone, with a `Gy-Format` trailer and no `Gy-Seq`, placed before the copy's unpushed writes, or right after taking in a remote that is ahead. The check on incoming commits admits exactly that shape without advancing the sequence, and refuses one that lowers the format, touches another file, or carries both trailers. From then on an older build stops at sync and is told to update; its unpushed writes stay in its copy.
- A write lands in the copy at once, as before (d-1e50). The push is done by a detached `gy sync` in the background, every 10 seconds while `gy serve` is running, and also by an explicit `gy sync`. It does not hold the ledger's lock during communication and takes it exclusively only for the span that changes the copy. 1 write = 1 commit: the subject is the why, the body is the actor and the human, and the trailer is `Gy-Seq`. Only the first upload is 1 commit with `Gy-Seq: 1-<seq>`.
- When the remote is ahead and there are unpushed lines locally, the remote's lines are taken in and each unpushed line is rebased after them. The only lines refused are a line whose touched node changed or disappeared on the remote side, a line whose edge has no target, a line whose ID collides, and a line that depends on a refused line; a rebased line gets a new seq (an unpushed seq is provisional; a node's ID does not change). A refused line stays in the copy's `rejected.jsonl`, appears on the standard error of that writer's next command and in the errors of `handover`, and does not go away until the same writer next writes to a node it touched. `handover` fetches the remote before it reports (it gives up after 5 seconds), and `undo` covers only one's own writes.
- Before taking in, the shape of the remote's history is checked: that it is a descendant of HEAD, that each commit has a `Gy-Seq` in sequence, that only gy's 4 files are touched, and that `events.jsonl` is only appended to. If it deviates, gy prints the sha, the author, the reason, and the concrete command that restores it with `--force-with-lease`. gy does not force push. A remote with content other than a ledger is refused.
- A write to a synced copy reads `git config user.name` and `user.email` on every write and puts them in the line's optional fields `by` and `by_mail` (without them it is refused). Reads show the human first, as `<by> / <actor>`, and the same actor name under a different human is a different writer.
- A copy has `sync.state` (the last sync, the last error), and `handover` and the screen show the number of unpushed writes and the last sync. Reads and writes go through while the remote is unreachable, and when it comes back the accumulated lines are pushed together. A broken copy, an emptied remote, a newer format version and tampering each print the cause and the remedy in 2 lines, and gy never deletes a copy or a remote on its own.
- A ledger without `remote` does not change at all. Removing `remote` turns the copy back into a local-only ledger (the next command says so once), and reconnecting is refused if both sides have advanced.

Basis: d-39f6 d-1e50 d-f7b6 n-6f47 n-8a52 n-f4cd n-ecbf n-94bb n-d36d n-6d6c

## gy serve is the eye that reads the ledger

gy serve is a read-only local server that shows the canonical store as it is now. First it answers the six questions, and second it looks good. Where publish freezes and emits the record of a given point and range, serve always shows the canonical store as it is now. A human does not operate gy, so serve has no write path; it is for looking only.

The HTTP layer is written by hand on the standard library alone and is contained in the single file server.rs. It accepts only GET, reads only the request line and the headers, and does not read a body. It replies with Content-Length and Connection: close, and binds only to 127.0.0.1. There is 1 thread per connection, a read is cut off after 5 seconds, the headers are at most 64 KB in total, and anything other than GET gets 405. The handlers are written against Request and Response structs that depend on no framework, and the tests call the handlers directly without a socket. The place to move to is whatever is the de facto standard at that time (today, bun + hono), and there are 4 criteria for moving. Any one of them starts the review: this build can no longer be built with the workspace's rust-version; RustSec issues an advisory and there is no fixed version; a requirement that a synchronous 1 thread per connection cannot meet (more than 10 people viewing at once, a need for HTTP/2 or TLS, a need for two-way communication) is raised as a need; the requirements cannot be met unless 1 file exceeds 300 lines.

serve judges neither states nor relations. The judgement belongs to the views of gy-ledger (now, list, show). now derives "what is waiting on the person now" and "which questions are not yet decided", and list and show derive a list and one item. The API of serve only takes the same Repository and Request and turns the result into JSON. The CLI and serve use the same judgement.

Every read API can take at=<seq>. The given point is represented by a MemoryStore that has replayed the log up to that seq. The ledger of that point is opened in place of the present canonical store and passed through the same views, so the present and the past are seen with the same way of reading.

serve watches the log file of the canonical store and checks for writes every 0.5 seconds. /api/wait?after=<seq> returns a response that waits long, until the next write. The screen makes the pulse, the lists and the time band follow without reloading, and they do not move while the point in time is rewound.

The graph is a single picture that zooms fractally. The server computes and holds the layout deterministically per seq and scope, and sends the browser only coordinates and edges. The items drawn increase with the step of the scale: from above, the bubble and the dots of each scope; closer, the edges; then the IDs; closer still, the titles. Dots and edges outside the screen are not drawn, and titles and bodies are fetched when zoomed in.

The language of the screen is English by default, and becomes Japanese through the browser's Accept-Language and a switch in the screen. There is no CLI option. Only the UI text and the names of the relations are translated; a node's title, body and scope note are shown in the language they were written in.

The E2E of the screen is kept in Playwright, and only chromium runs, on ubuntu-latest in CI. Each scene fetches the API's JSON and checks it against the DOM's counts, text and transitions, and guards the key operations. Images are only kept as artifacts and are not compared. It is not part of cargo test.

Basis: d-25c4 d-799e d-e6c4 d-685d d-995c d-9751 d-b93b d-9c5f d-244b
