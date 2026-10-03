# The design of the new gy

This document states the design contract of the new gy (0.5). The contract of use is in the [README](../README.md), the migration from 0.4 is in [migration-0.5](migration-0.5.md), and the release procedure is in [release](release.md).

## Four layers and the direction of dependency

The crate `gy-ledger` is divided into four layers. `store` (persistence, transactions, ID allocation, history, the format version) → `model` (the five kinds of node and the invariants) → `ops` (operations = intents) → `views` (the projections show / list / next / handover / publish). Dependencies flow in this direction only. `views` does not read `store` directly, and `ops` does not call `views`. The CLI (`gy`) only calls `views` and `ops`.

The responsibilities of each layer are as follows. `store` uses no other layer. `model` uses only `store`. `ops` uses `store` and `model`. `views` uses `model` and `ops`. Inside one layer, references go through `super::`.

## Always valid

An invalid state is refused at the moment of the write, not by a pass that checks later. Every node in the ledger satisfies the invariants of that moment.

What is refused when a node is built: an empty scope, an empty title, a `created` that is not a UTC instant (`YYYY-MM-DDTHH:MM:SSZ`), a decision with an empty scope note (except the unrecorded marker that only the migration attaches), a pair of nodes the relation does not allow, a `narrows` or `supersedes` mark that is not found in the body or the scope note of the older decision, a second registration of the same edge (whether the write is a create operation or a rebase line), closing a closed question again, making a `closes` edge with `link`, a `closes` edge on a question whose closure is not a decision, an edge to a node that does not exist, and a `depends-on` or lineage edge (`narrows` / `widens` / `supersedes` / `completes`) that would close a cycle, a self-loop included, and adding a `filed-as` requirement to a closed need.

Every write is one transaction. One intent corresponds to one command, and if it fails partway, nothing is written. The history of the change rides in the same transaction: it stays when the transaction commits and is discarded when it fails.

An achievement needs an approved requirement (d-b0d0, d-5dfe). A requirement is the promise of what will change outside the ledger, and approval is the event that the promise was accepted. gy cannot see the outside work, but it can refuse to record its result: a criterion turns satisfied only while a requirement that is approved or done points at it with `targets`. While a requirement stays approved, its title, body, and its `targets` and `relies-on` edges are frozen, and so are the title and body of a criterion it targets; `req revise` sends it back to filed first. Who approves is the user's policy and gy does not judge it. The rule looks only at what a write changes, so what a ledger already holds stays valid.

The rule is one function in the model, and one gate carries it to every path a write can take (n-557f): the commit of an ordinary write, the commit of an undo or redo, and the placing of each pending write when a shared ledger rebases. The store defines the gate in its own terms (the JSON values it holds and the log's changes), the model holds the rule behind a small view of the previous state, and the operations join them, so the store still calls neither model nor operations. A transaction that carries no criterion, no requirement, and no edge a need or decision adds reads no previous state. Replaying history (opening, reading a point in time) never reaches the gate. The coverage judgement is a single function that the rule and the advice (`missing`, `next`) share.

Every date is an instant. A node's `created`, an acceptance criterion's `satisfied_at` and the `at` of a requirement's record are stored as UTC instants, and the CLI and serve show them as the date and time of the viewer's place (`TZ`, the browser). `--json` emits the stored value as it is; `publish` writes the date alone. `--since <date>` starts at 0:00 of that day in the viewer's place. The shared word a team uses to point at "when" is not a date but `seq` and the ID (d-b1f8).

## The event log, the snapshot and the format version

The canonical store is an append-only log named `events.jsonl`, and one transaction corresponds to one line. A line has `seq`, `at`, `actor`, `why`, `source` and `changes`; a write that was retried after a conflict has `retries` as an optional field; and each item of `changes` describes the change to one node (`node`, `change`, `value`). `change` is one of `created`, `updated`, `deleted` and `scope-renamed`. `scope-renamed` changes the scope name of several nodes in one line; its `node` is empty and its `value` is `{from, to, nodes}`.

Atomicity comes from an append that writes one line and its newline in a single write, and from `fsync`. A tail that was cut off partway is dropped by a writer, after it takes the exclusive lock and just before it appends. A reader does not change the log (n-6b71). A conflict between writers writing at the same time is detected by the exclusive lock and a comparison of `seq`, and the writer that lost reopens the ledger and runs the same intent again, up to 5 times. If the intent no longer holds, it fails with the reason of that invariant, and after losing 5 times it gives up (n-fe59). The history is the log itself, and `undo` appends a new line that walks the last transaction backwards.

`snapshot.json` is derived and holds the current nodes and `seq`. It only makes opening faster; if it is deleted, the log alone gives the same result. The `format` file holds the format version, and a ledger has this version from the moment it is born. Opening a version this build does not support is an error. A migration that raises the version does not rewrite the log, advances one step at a time, and leaves the file as it was before the change at `<name>.<version>.bak`. In version 3 (0.9.0) the reader reads a date-only value as the instant `T00:00:00Z` and rebuilds the snapshot. Version 4 (1.0.0) changes no data: it exists because an older build does not know the requirement rule and would write around it, so the step only keeps `format.3.bak` and writes the version.

The canonical store is located at `$XDG_DATA_HOME/gy/<hash of the repository root>/`. The only thing placed in the repository is `gy.toml`. `gy init <scope>` writes that file and nothing else, before any root has been resolved and without opening a ledger; the store still appears with the first write (n-29b3, d-0f6d). It searches upward to the nearest `.git` boundary first and reports the root above within the same repository rather than start a second store under it, because one repository has one canonical store (d-4221, d-0a47). A `.git` without a `gy.toml` below stops the search, so a nested repository never silently uses the record above. Its output is where a first-time agent learns what to read and type next, since the `gy-loop` skill only fires where a `gy.toml` already exists.

## ID generation and collisions

An ID is made of a prefix for the kind and a short hash. There is no central allocator, so several agents may write at the same time. The seed mixes the prefix, the time in nanoseconds, the process ID, the number of nodes and a salt, and a different seed is hashed for each width. On a collision the ID widens from 4 digits to 6 and then to 8. This is because merely cutting out the low bits of the same 32 bits leaves collisions at the short widths.

The IDs of 0.4 are kept as aliases. `show` resolves a full ID, a zero-padded ID, an alias and a requirement's outward reference, by exact match or by suffix match. A requirement's reference (`ref`) is an opaque value, and gy does not read what it points at.

## Edges are placed on the from side only

A node holds only the edges that start from itself. The reverse direction is not stored; `Repository::incoming` assembles it by scanning all nodes. The same edge is not written in two places.

The relations are a closed set of 12 values, and the reverse names are derived. The allowed pairs of node kinds are gathered in one table, and an edge that is not in the table is refused at the time it is built. Only the forward side of `narrows` and `supersedes` holds a `mark`, and it is carried over when the reverse direction is assembled.

## Derived values

Some values are not stored and are computed from the graph every time.

- The state of a need: `closed` if it is closed, `done` if at least one of its `filed-as` requirements is `done`, none of them is `filed` or `approved`, and every acceptance criterion it `targets` is satisfied, and `open` otherwise. A `cancelled` requirement counts on neither side, so it neither completes the need nor holds it open (d-bf90).
- Whether it can be started (the condition of `next`): it is `open`, its `depends-on` needs are `closed` or `done`, and what it `waits-on` is settled (a question is closed; a requirement is `done` or `cancelled`).
- `bearer_count`: the number of needs that have that acceptance criterion in their `targets`.
- Requirements in progress: requirements that are `filed` or `approved`.
- A decision's retraction (n-0c6f, d-bde9): which newer decisions `narrow` or `supersede` it, and, from each `mark`, which passage of its body or scope note stops applying. It is derived whether or not `--full` is asked for, from one pass over the graph per invocation, and it is the one judgement every read path shows: `show` and serve render it and decide nothing of their own. A `narrows` wraps the passage where it stands (`[[retracted by <id>: <mark>]]`) so a reader meets it in the text rather than having to connect a line elsewhere; a `supersedes` covers the whole decision and only names itself above the body. A decision whose mark falls outside the `## Decision` section shows its whole body rather than hide the mark. The marking is a projection: the stored body is what `--json` carries, and the marked text rides beside it; `publish` writes the marked text. A mark is refused where an edge is made (`link`, `decide`) when it is not in the older decision's body or scope note; an `edit` is never refused this way, since it cannot tell a passage that follows its retraction from one that misses it. It applies the edit and names, on the write's `unresolved` line, each incoming `narrows` / `supersedes` mark that named a passage before and does not after, from one pass over the graph; a mark that never resolved stays silent (n-847d, d-8f76).

## Invariants of the nodes and edges

This section lists what gy holds true of the five nodes and the twelve edges (n-1a61, reviewed three times by a second reader, ac-3e58). Each item gets a property test in `crates/gy-ledger/tests/` (n-1a61); the table at the end of the section maps an item to its test as the tests land.

Each item says where it is held: **W** refused by the operation at write time; **G** refused by the gate, so on every path a write takes (an ordinary write, `undo` and redo, and the rebase of a shared ledger); **D** derived on read.

There are two forms, and a property test checks each in its own way:

- A **rule** judges a change: "a write that does X is refused". It looks only at what the write changes, so what a ledger already holds stays valid, and replaying the log never runs a rule (d-b0d0).
- A **state invariant** holds of every ledger that started empty and was written only through this build's operations: "starting from a ledger where it holds, it still holds after any sequence of operations". A ledger written by an older build may break it; such a ledger still opens and reads.

"Through its own operation" means the operation that names the change (`need close`, `req done`, …). `undo` is not one of them: it takes back the last transaction as a whole and restores the previous values, and it passes the gate like any write (T4).

### Every node

- **N1** An ID is the kind's prefix (`n`, `q`, `d`, `r`, `ac`) and a nonempty hash, unique in a ledger; a node's ID never changes, including across a rebase. W — `model/kind.rs`, ID minting in `store`.
- **N2** The scope is nonempty; a write puts a node only in a scope named in `gy.toml`. W — `model/node.rs:build`, scope checks in `ops`.
- **N3** The title is nonempty. W — `model/node.rs:build`.
- **N4** `created` is a UTC instant `YYYY-MM-DDTHH:MM:SSZ` and never changes. Every other stored instant (`satisfied_at`, the `at` of an approval, a revision, a completion and a cancellation) has the same form. W — `model/node.rs:valid_created`, `model/state.rs:valid_instant`.
- **N5** `edit` changes only: the title, the body, the scope (to a scope in `gy.toml`), free attributes, and a decision's scope note once while it is the unrecorded marker. Every other typed field (requirement state, reference, approval / revision / completion / cancellation, question closure, decider, options, need closure, criterion satisfaction, a recorded scope note) changes only through its own operation; `edit` refuses those keys. W — `ops/edit.rs:set_one`, `RESERVED`.
- **N6** A reference to a node (full ID, padded ID, alias, a requirement's `ref`) resolves to exactly one node or is refused. W/D — `ops/snapshot.rs`.

### Edges

- **E1** A relation is one of the 12, and its (from kind, to kind) pair is in the one table. W — `model/links.rs:ALLOWED`.
- **E2** An edge is stored on its from side only; the reverse is derived. — `model/links.rs`, `Repository::incoming`.
- **E3** Rule: a write that would store the same edge (from, relation, to) a second time is refused, on every path. W/G — `model/edges.rs` (n-d2a5, d-fb49, d-ab9c).
- **E4** Rule: an edge's target exists when the edge is written, on every path that writes one. W/G — `ops/*`, `store/remote/rebase.rs`.
- **E5** `narrows` and `supersedes` carry a mark on the forward side only. Rule: the mark is refused when the edge is made (`link`, `decide --relate`) unless it is found in the older decision's body or scope note. `edit` is never refused for a mark (see D2). W — `ops/marks.rs`.
- **E6** Rule: a `closes` edge comes only from a question closed by decision (`question close --by decision --decision D`, `decide --closes Q`); `link` never makes one, and a close by fact or non-decision makes none. W/G — `ops/link.rs`, `ops/question_close.rs`, `model/edges.rs` (n-ac83, d-fb49).
- **E7** Rule: a write that adds a `depends-on` edge, or a lineage edge (`narrows`, `widens`, `supersedes`, `completes` together), that closes a cycle, a self-loop included, is refused. State invariant: both graphs stay acyclic. G — `model/shape.rs` (n-e299, d-858d).
- **E8** Rule: a write that adds a `filed-as` to a need that is closed after the write is refused. G — `model/rule.rs:check_closed_need` (n-d5a2, d-ee34).

### Need

- **Nd1** A need is created with at least one `targets`. W — `ops/need_add.rs`.
- **Nd2** Through its own operation a need closes once, with `by` (fact or external) and nonempty evidence; `need close` on a closed need is refused, and no operation reopens it. W — `ops/need_close.rs`.
- **Nd3** Its state is derived: `closed` if closed; `done` if at least one `filed-as` requirement is done, none is `filed` or `approved`, and every criterion it targets is satisfied (a cancelled requirement counts neither way); `open` otherwise. D — `views/derive.rs:need_state` (d-85c6, d-bf90).
- **Nd4** State invariant (from Nd3): a done need has no unmet targeted criterion. D.
- **Nd5** It is ready iff it is open, every `depends-on` need is closed or done, and everything it `waits-on` is settled (a question closed; a requirement done or cancelled). D — `views/derive.rs:ready`.

### Question

- **Q1** A question has a nonempty decider and at least two options. W — `ops/question_add.rs`.
- **Q2** Through its own operation a question closes once, with `by` and evidence; closing by decision names the decision and makes `closes` to it (E6). Closing a closed question is refused. W — `ops/question_close.rs`, `ops/decide.rs`.
- **Q3** A question is open iff it has no closure. D — `views/derive.rs:question_open`.

### Decision

- **D1** A decision has a nonempty scope note, except the unrecorded marker only the migration sets; an unrecorded note can be recorded once with `edit --set decision_scope=` (N5), and a recorded one never changes. W — `model/scope.rs`, `ops/decide.rs`, `ops/edit.rs`.
- **D2** Retraction, derived (n-0c6f, d-bde9, d-8f76):
  - an incoming `narrows` retracts the passage its mark names; an incoming `supersedes` retracts the whole decision;
  - the stored body and scope note never change by this; the marked text is returned beside them (`--json` carries the stored body), and `show`, serve and `publish` render the same marked text from one function;
  - a mark that falls outside the `## Decision` section shows the whole body;
  - an `edit` names, on `unresolved`, each incoming mark that resolved before the edit and does not after; a mark that never resolved is not named.
  D — `views/retraction.rs`, `ops/edit.rs:69,87`.

### Requirement

- **R1** A requirement is created for at least one need, which files it (`filed-as`). W — `ops/req_add.rs`.
- **R2** Through its own operations its state moves only filed→approved, approved→filed (revise), approved→done, filed|approved→cancelled; no operation moves it out of done or cancelled. W — `model/state.rs:advance`.
- **R3** Each move stores its record with nonempty fields: approval (design, heard_by, evidence), revision (reason, source), completion (evidence), cancellation (reason, source). A revised requirement keeps its approval record. W — `ops/req_*.rs`.
- **R4** A reference is opaque, and no two requirements in progress (filed or approved) share one. W — `ops/req_add.rs:check_reference`.
- **R5 (I2)** Rule: while a requirement stays approved, a write that changes its title, body, `targets` or `relies-on`, or the title or body of a criterion it targets, is refused. A done requirement freezes nothing. G — `model/rule.rs`.

### Criterion

- **C1 (I1)** Rule: a write that turns a criterion satisfied is refused unless an approved or done requirement targets it. It judges the change, not the state: cancelling or revising the covering requirement later leaves the criterion satisfied (n-5b94). G — `model/rule.rs:check_criterion`, `covered`.
- **C2** Satisfying needs evidence and refuses an already satisfied criterion; `--revoke` turns it back to unsatisfied. W — `ops/criterion_satisfy.rs`.
- **C3** Coverage is one function: a criterion is covered iff an approved or done requirement targets it. The rule (C1) and the advice (V2) call the same function. — `model/rule.rs:covered`.
- **C4** `bearer_count` of a criterion is the number of needs that target it (needs, not edges and not requirements). A criterion is **orphaned** iff it is unmet, its bearer count is at least one, and every bearing need is closed through `need close` (a derived `done` does not count). D — `ops/advice.rs:unmet_orphaned`, `views/handover.rs:orphaned_criteria`.

### Writes and history

- **T1** One intent is one transaction and one line of the log; a failed write leaves nothing. W — `store`.
- **T2** Every write names its actor; a write to a shared copy also carries the git user (`by`). W — `store/mod.rs`, n-64be.
- **T3** The log is append-only, and `seq` rises by one per line; the snapshot equals the replay of the log. — `store`.
- **T4** `undo` takes back exactly the last transaction, only when it is the writer's own (the same actor, and on a shared copy the same `by`), and undo again redoes it. The reverse change passes the gate and may be refused by a rule (for example E7). G — `ops/undo.rs`, `store/file/undo.rs`, `store/file/mod.rs:95`.
- **T5** The gate runs the model's rule on every path a write takes: commit, undo/redo, rebase. — `ops/judge.rs`, n-557f.

### Derived judgements shared by every reader

- **V1** The CLI and serve read every derived value through the same function of `gy-ledger`; serve judges nothing of its own.
- **V2** `missing` (D — `ops/advice.rs`, `ops/advice/need.rs`), by kind and state. `missing` is advice: it never refuses a write.
  - a need not closed through `need close` (derived `open` or `done`): a body if empty; `a filed-as requirement` if every `filed-as` requirement is cancelled or there is none; otherwise each targeted criterion that is unmet and that no filed or approved requirement covers (`unmet criterion <id>`, n-f60a). A derived `done` need with an empty body still reports the body;
  - a closed need: each of its orphaned criteria (C4);
  - an open question: a body if empty; a need that waits on it or a requirement that raised it, if neither exists;
  - a decision: a body if empty;
  - a filed requirement that has never been approved: a `relies-on` decision, a `targets` criterion, and a `ref`, each if absent; a revised requirement (it keeps its approval) reports none;
  - an unmet criterion: a body if empty; an approved requirement (targets) if it is not covered (C3);
  - every other node (closed question, approved / done / cancelled requirement, satisfied criterion): nothing.
- **V3** The step toward satisfying an unmet criterion: `criterion satisfy <it>` if it is covered (C3); `req approve <R>` if a filed requirement targets it; otherwise `req add … --need <N> --targets <it>`, where `<N>` is a bearing need not closed through `need close`, or the literal placeholder `<N>` when there is none. `next` for a criterion is: for an unmet one, that step; for a satisfied one, the same step for an unmet criterion that shares a bearing need with it, or nothing when there is none (it never suggests satisfying it again, C2) (n-5a63). For an open need with a `filed-as`, `next` adds the step for each criterion V2 lists. For every kind, when the node reports an empty body, `edit <id> --body-file …` comes first.
- **V4** `next` (the command) lists the ready needs (Nd5) ordered by `created`, then ID, and holds no priority.
- **V5** handover's warnings are counts: a requirement in progress relying on a superseded decision or on an unrecorded scope note; a criterion with an empty body; an open question nobody waits on; an orphaned criterion (C4).

### Item to test

| Item | Test |
| --- | --- |

## What handover and next judge

`handover` reports the errors (an edge to a node that does not exist, a `filed-as` edge whose target is not a requirement), the requirements in progress (`ref`, state, `next_evidence`, `responsible`), the number of open questions, the number of needs that can be started, and the number of warnings. The warnings are: a requirement in progress that relies on a superseded decision, a requirement in progress that relies on a decision whose scope note is unrecorded, an acceptance criterion with an empty body, an open question that nobody waits on (neither `waits-on` nor `raised`), and an unmet acceptance criterion whose bearing needs are all closed (counts only; d-09b6, d-f7b6). It lists the states that need attention in an order a person can read.

`next` lists the needs that can be started in the order of `created` and ID. It holds no priority. The agent chooses which need to advance and offers it to the person.

## publish writes the record as a Markdown wiki

`publish` writes the record as a Markdown wiki a person reads on GitHub (d-90a1, d-50d1): one directory per scope under the output directory, an entry `README.md`, and one page per need and per decision. The nodes a vertex reaches — a need's criteria, its requirements, and the questions those raised or it waits on, a decision's closed questions — are shown in full on that page. A node no need and no decision reaches goes to `loose.md`, so every node of the scope is readable in full. The output directory is kept outside git's control (in gy's own repository it is excluded by `.gitignore`). The content of a ledger is internal development information, and the wiki is for the people the agents work for, not a publication of gy as a tool.

- A page: `<scope>/<ID>.md`, named for the vertex's ID. Front matter (`id`, `kind`, `state` for a need and a requirement, `scope`, `created` as `YYYY-MM-DD`, and its edges by relation, with `-` written `_`), the title, a link back to the entry, and the sections. The body is written with its narrowed passages struck through (`~~…~~ *(retracted by d-…)`), the one judgement `show` and serve share (d-bde9). Another scope's node is plain text with its scope named, never a link.
- The entry: `<scope>/README.md`. The count of nodes and pages, the record's `seq`, the open questions, the work being built, the newest vertices, and the full decision and need lists. It carries no legend, no history and no diagnostics.
- `loose.md`: the nodes no vertex's page expands, in full.
- The output directory is `--out` or `output` in `gy.toml` (a directory). Only the directory of the target scope is deleted and rebuilt; nothing else is touched.
- `--since` is accepted for the 1.0.1 CLI and has no effect (d-3c54): the wiki always shows the record as it is now, and says so once on standard error.
- Publishing twice from the same ledger gives the same files byte for byte; no generated time is written.

## Team sync takes the git remote as the authority (experimental)

When `remote` in `gy.toml` names a git repo dedicated to the ledger, the default branch of that repo becomes the canonical store, and the ledger directory on each machine becomes a copy of it (d-39f6). The authority is not "the one who judges which is right after a conflict" but "the one place that decides the order before a conflict", and this keeps the straight line of seq and the single validation at write time (always valid). There is no merge, and there is no branch.

- A copy is a git working tree whose top level is the ledger directory itself, and it tracks only `events.jsonl`, `format`, `README.md` and `.gitignore`. Even when the ledger directory is inside another working tree, it makes a nested repo, and before a commit and a push it checks that origin matches the recorded remote. If it does not match, it touches nothing (n-6d6c).
- The remote's format is raised by the copy that is ahead of it (d-bace): one commit that changes `format` alone, with a `Gy-Format` trailer and no `Gy-Seq`, placed before the copy's unpushed writes, or right after taking in a remote that is ahead. The check on incoming commits admits exactly that shape without advancing the sequence, and refuses one that lowers the format, touches another file, or carries both trailers. From then on an older build stops at sync and is told to update; its unpushed writes stay in its copy.
- A write lands in the copy at once, as before (d-1e50). The push is done by a detached `gy remote sync` in the background, every 10 seconds while `gy serve` is running, and also by an explicit `gy remote sync`. It does not hold the ledger's lock during communication and takes it exclusively only for the span that changes the copy. 1 write = 1 commit: the subject is the why, the body is the actor, the human and the writing gy's release (`Gy-Version`, n-670a), and the trailer is `Gy-Seq`. A sync keeps the newest `Gy-Version` it pulls in `sync.state`, read in one git call, and when it is newer than the running gy, `remote sync`, `handover` and the `gy serve` log say so once. Only the first upload is 1 commit with `Gy-Seq: 1-<seq>`.
- When the remote is ahead and there are unpushed lines locally, the remote's lines are taken in and each unpushed line is rebased after them. The only lines refused are a line whose touched node changed or disappeared on the remote side, a line that changes a node the ledger does not hold, a line whose edge has no target, a line whose ID collides, and a line that depends on a refused line. A node the same line creates counts as held, both for its edges and for the changes it makes (n-ef56); a rebased line gets a new seq (an unpushed seq is provisional; a node's ID does not change). A refused line stays in the copy's `rejected.jsonl`, appears on the standard error of that writer's next command and in the errors of `handover`, and does not go away until the same writer next writes to a node it touched. `handover` fetches the remote before it reports (it gives up after 5 seconds), and `undo` covers only one's own writes.
- Before taking in, the shape of the remote's history is checked: that it is a descendant of HEAD, that each commit has a `Gy-Seq` in sequence, that only gy's 4 files are touched, and that `events.jsonl` is only appended to. If it deviates, gy prints the sha, the author, the reason, and the concrete command that restores it with `--force-with-lease`. gy does not force push. A remote with content other than a ledger is refused.
- A write to a copy that has been shared — a synced copy, or one that is its own git repository with an origin while its marker is away — reads `git config user.name` and `user.email` on every write and puts them in the line's optional fields `by` and `by_mail` (without them it is refused). A never-shared ledger is not a git repository and never reads them. A sync does not push a line without `by`: it keeps such lines in the copy, pulls nothing until they are gone, and says so with the way out (n-64be). Reads show the human first, as `<by> / <actor>`, and the same actor name under a different human is a different writer.
- A copy has `sync.state` (the last sync, the last error), and `handover` and the screen show the number of unpushed writes and the last sync. Reads and writes go through while the remote is unreachable, and when it comes back the accumulated lines are pushed together. A broken copy, an emptied remote, a newer format version and tampering each print the cause and the remedy in 2 lines, and gy never deletes a copy or a remote on its own.
- Apart from a shared ledger, gy's only request off the machine is a detached `cargo info gy`, at most once a day, run from gy's data directory outside any workspace (d-1f57). `handover` and `next` only read the small cache it leaves there and say on stderr when crates.io has a newer release; a check that runs past 30 seconds is given up on but never killed.
- A ledger without `remote` does not change at all. Removing `remote` turns the copy back into a local-only ledger (the next command says so once). `gy.toml` lives in the project's git, so checking out an older commit removes the line and coming back restores it. When the same `remote` returns and the copy's origin still names it, the copy is bound again with no step from the person, and what was written meanwhile is pushed as usual (d-dc39). A copy whose origin differs, or a separate ledger, is refused. The failure `gy serve` logs says why the sync stopped; the advice on the URL and the credentials is given only for a failed fetch.

Basis: d-39f6 d-1e50 d-f7b6 d-dc39 n-6f47 n-8a52 n-f4cd n-ecbf n-94bb n-d36d n-6d6c n-9f9d n-64be

## gy serve is the eye that reads the ledger

gy serve is a read-only local server that shows the canonical store as it is now. First it answers the six questions, and second it looks good. Where publish writes the record as a Markdown wiki for a person to read on GitHub, serve always shows the canonical store as it is now. A human does not operate gy, so serve has no write path; it is for looking only.

The HTTP layer is written by hand on the standard library alone and is contained in the single file server.rs. It accepts only GET, reads only the request line and the headers, and does not read a body. It replies with Content-Length and Connection: close, and binds only to 127.0.0.1. There is 1 thread per connection, a read is cut off after 5 seconds, the headers are at most 64 KB in total, and anything other than GET gets 405. The handlers are written against Request and Response structs that depend on no framework, and the tests call the handlers directly without a socket. The place to move to is whatever is the de facto standard at that time (today, bun + hono), and there are 4 criteria for moving. Any one of them starts the review: this build can no longer be built with the workspace's rust-version; RustSec issues an advisory and there is no fixed version; a requirement that a synchronous 1 thread per connection cannot meet (more than 10 people viewing at once, a need for HTTP/2 or TLS, a need for two-way communication) is raised as a need; the requirements cannot be met unless 1 file exceeds 300 lines.

serve judges neither states nor relations. The judgement belongs to the views of gy-ledger (now, list, show). now derives "what is waiting on the person now" and "which questions are not yet decided", and list and show derive a list and one item. The API of serve only takes the same Repository and Request and turns the result into JSON. The CLI and serve use the same judgement.

Every read API can take at=<seq>. The given point is represented by a MemoryStore that has replayed the log up to that seq. The ledger of that point is opened in place of the present canonical store and passed through the same views, so the present and the past are seen with the same way of reading.

serve watches the log file of the canonical store and checks for writes every 0.5 seconds. /api/wait?after=<seq> returns a response that waits long, until the next write. The screen makes the pulse, the lists and the time band follow without reloading, and they do not move while the point in time is rewound.

The graph is a single picture that zooms fractally. The server computes and holds the layout deterministically per seq and scope, and sends the browser only coordinates and edges. The items drawn increase with the step of the scale: from above, the bubble and the dots of each scope; closer, the edges; then the IDs; closer still, the titles. Dots and edges outside the screen are not drawn, and titles and bodies are fetched when zoomed in.

The language of the screen is English by default, and becomes Japanese through the browser's Accept-Language and a switch in the screen. There is no CLI option. Only the UI text and the names of the relations are translated; a node's title, body and scope note are shown in the language they were written in.

The E2E of the screen is kept in Playwright, and only chromium runs, on ubuntu-latest in CI. Each scene fetches the API's JSON and checks it against the DOM's counts, text and transitions, and guards the key operations. Images are only kept as artifacts and are not compared. It is not part of cargo test.

Basis: d-25c4 d-799e d-e6c4 d-685d d-995c d-9751 d-b93b d-9c5f d-244b
