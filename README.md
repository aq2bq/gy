# gy — good,yes

English | [日本語](README.ja.md)

gy records the state before a requirement is confirmed. It holds the needs, questions, decisions, requirements, and acceptance criteria that a piece of work rests on, as a graph of nodes and edges, with one way to use it and almost nothing to configure. Agents write it as they work, and `publish` writes the record to a file that is committed and read back later. The canonical ledger lives outside the repository, and gy makes no network calls.

## What gy is for

Work handed to an agent does not need to be read while it goes well. Because it is not read, it stops being read. How far the work gets then depends on how much fits in the context window, how large the target is, and how strong the model is, and none of that shows while things go well.

It shows at the end. A requirement has to be declared complete. A decision that earlier work relied on has been replaced without anyone noticing. Leftover work has to go somewhere. And the one person accountable is holding several projects at once and cannot read them all. None of the four happens while things go well. When one does, nobody remembers what the work was based on.

gy exists for that end. It holds no plan and no schedule. It records what each piece of work rests on and what must hold before the work can be called done, and it reports where those records contradict each other. There is no `lint` pass to run later: an invalid write is refused when it is made, and what needs attention is counted by `handover`.

## The five nodes and twelve edges

There are five kinds of node. Four of them form a ring that follows one turn of the work: a decision spawns needs, a need is filed as a requirement, the work raises questions, and a question closes as the next decision. The fifth kind, the acceptance criterion, measures the work from outside the ring.

| Node | ID | Meaning |
| --- | --- | --- |
| need | `n-…` | Work that must be done, aimed at acceptance criteria |
| question | `q-…` | Something undecided, with a decider and at least two options |
| decision | `d-…` | A decision with the conditions under which it applies |
| requirement | `r-…` | A need turned into an approval request, with an outward reference |
| criterion | `ac-…` | An acceptance criterion that work is counted against |

Edges are stored on the node they start from; the reverse direction is derived, never written twice. An edge is one of twelve relations, and each relation is only allowed between the kinds shown here.

| From → to | Relation | Reverse |
| --- | --- | --- |
| question → decision | `closes` | `closed-by` |
| decision → decision | `narrows`, `widens`, `supersedes`, `completes` | `narrowed-by`, `widened-by`, `superseded-by`, `completed-by` |
| need, requirement → criterion | `targets` | `targeted-by` |
| need → decision | `spawned-by` | `spawns` |
| need → requirement | `filed-as` | `files` |
| need → need | `depends-on` | `depended-on-by` |
| requirement → decision | `relies-on` | `relied-on-by` |
| requirement → question | `raised` | `raised-by` |
| need → question, requirement | `waits-on` | `awaited-by` |

A decision is stored with its applicability conditions, so a later reader can tell where it does and does not hold. `narrows` and `supersedes` also name the passage of the older decision that loses effect; the mark is checked against that decision's text when the edge is written.

## How a requirement moves

A requirement has four states: `filed` (registered), `approved` (design confirmed), `done` (shipped), and `cancelled`. Approval, revision, completion, and cancellation are each one command that records who heard the design, the evidence, and the reason, and every other transition is refused.

| From | To | Command |
| --- | --- | --- |
| filed | approved | `req approve` |
| approved | filed | `req revise` |
| approved | done | `req done` |
| filed, approved | cancelled | `req cancel` |

gy holds nothing about the work after approval except these four records. Where the implementation is tracked, how it is designed, and when it is audited belong outside gy, in the issue and pull request that the requirement's reference points at.

## Where the ledger lives

The canonical ledger is an append-only event log outside the repository, under `$XDG_DATA_HOME/gy/<hash of the repository root>/`. The repository itself holds only `gy.toml`. Every write is one transaction appended to the log with the sequence number, time, actor, reason, and source; nothing is edited in place.

`undo --reason <text>` inverts the last transaction as a new one, so the history keeps both the mistake and the correction. It undoes one transaction only; a second undo undoes the first undo (a redo). The log is the ledger; a snapshot file alongside it only speeds up opening and can be deleted.

## The twenty operations

Reads (6):

| Operation | Result |
| --- | --- |
| `show <ID\|ref>... [--full]` | One or more nodes by ID, alias, or reference, with what each still lacks |
| `list [--type] [--status] [--targets] [--grep] [--actor] [--since]` | Node rows, or write units when `--actor` or `--since` is given |
| `next` | The needs whose prerequisites are settled |
| `handover` | In-progress requirements and the counts a session needs to resume |
| `publish [--scope] [--since] [--out]` | Write the record at a point and range into a directory: one file per node and a scope index |
| `serve` | Read the ledger in a browser, on 127.0.0.1 until stopped |

Writes (16):

| Operation | Result |
| --- | --- |
| `need add "<title>" --targets <AC>... [--spawned-by <D>]` | File a need against acceptance criteria |
| `need close <ID> --by fact\|external --evidence <text>` | Close a need without a requirement |
| `question add "<title>" --decider <name> --options <text>...` | Open a question with a decider and at least two options |
| `question close <ID> --by fact\|decision\|non-decision --evidence <text> [--decision <D>]` | Close a question and, when decided, record the decision |
| `criterion add "<title>"` | Add an acceptance criterion |
| `criterion satisfy <AC> --evidence <text> [--revoke]` | Record that a criterion holds, or revoke it |
| `req add "<title>" --need <N>... [--relies-on <D>]... [--targets <AC>]... [--ref <ref>]` | File a requirement against needs, decisions, and criteria |
| `req approve <ID\|ref> --design <text> --heard-by <name> --evidence <text>` | Confirm the design of a requirement |
| `req revise <ID> --reason <text> --source <text>` | Send an approved requirement back to filed |
| `req done <ID> --evidence <text>` | Record that an approved requirement shipped |
| `req cancel <ID> --reason <text> --source <text>` | Cancel a requirement that was not done |
| `decide "<title>" --scope-note <text> [--body-file <path>] [--closes <Q>]... [--relate <relation> <D> --mark <text>]` | Create a decision, close questions, and record one lineage edge |
| `link <from> <relation> <to> [--mark <text>] [--remove]` | Add or remove one edge |
| `edit <ID> --reason <text> [--title] [--body-file] [--set k=v] [--append k=v]` | Change a node's title, body, or free attributes. A free attribute is a string; `--set` overwrites it, `--set k=` drops it, and `--append` adds one line, separated by a newline. `--set scope=<name>` moves the node to a scope gy.toml declares, and `--set decision_scope=<text>` records a decision's unrecorded applicability conditions once |
| `scope rename <old> <new>` | Move every node of a scope to a new name and rewrite gy.toml, keeping its comments and order |
| `undo --reason <text>` | Invert the last transaction |

Every write prints what it changed, what the node still lacks, and the shape of the command that could come next, so the next step is visible without a separate instruction sheet. Pass `--json` for the same content as data.

## Resuming a session

A new session starts with three commands. `handover` shows the in-progress requirements with their references, the number of open questions, the number of ready needs, and the errors and warning counts. `next` lists the needs whose prerequisites are settled, and the agent presents one of them to the master. `show` reads one node in full.

```sh
gy handover
gy next
gy show n-3f9a
```

## gy.toml

The only configuration is a scope name and, if wanted, an output path for `publish`. Anything else is refused when the file is read.

```toml
[scopes.myproject]

# Optional. publish writes here when --out is not given.
output = "docs/publication"
```

Reads cover every scope. A write needs a scope only when the file names more than one; pass `--scope <name>` to choose. The first write creates the ledger, and reads never do.

## Who writes

Every write names its actor in `GY_ACTOR`. An unset or blank value is an error, and the name is kept in the history beside the reason and source.

```sh
export GY_ACTOR=leader
```

Each agent writes its own records with its own name, so the history shows who changed what and why.

## IDs, aliases, and refs

gy allocates each ID itself, as a kind prefix and a short hash, such as `n-3f9a`. A hash always contains at least one letter a–f, so it never looks like an old ID. No counter is shared, so two agents writing at once cannot collide; a collision just mints a longer hash. `show` accepts the exact ID, an alias (zero-padding and case are ignored, so `D-8` = `D-08`), or a requirement's outward reference by exact or suffix match.

An existing ledger keeps its old IDs as aliases, so `show D-164` and `show '#6027'` both reach the renamed node. A requirement's reference (`--ref`) is opaque: gy stores it and never reads what it points at.

## publish

`publish` writes the record at a point and range into a directory to commit: one file per node under `<out>/<scope>/<kind>/`, and a scope index at `<out>/<scope>/README.md`. It is a development artifact: later, an agent reads it to review what was decided and why, and diffs one publication against the next. It is not a reading for the master, and gy adds no human-facing output format.

A node file holds the id with its old aliases and reference, the title, scope, creation date, state, applicability conditions, the body, both edge directions with the other side's id, alias, title, and mark, the closure and evidence, the requirement records, and the free attributes. Every reference carries the target's title, so each file stands on its own. A decision file puts its lineage relations first.

The index holds the generated time, the log sequence, the scope and range, the writer, and the canonical location; a short "how to read" section; a per-kind list with a link and state for each node; the write history; and the diagnostics.

`--out` names the output directory, `gy.toml`'s `output` names a default, and without either publish is an error. Only the target scopes' directories are removed and rewritten; other files under `--out` and other scope directories are left alone.

## Install

```sh
cargo install gy --locked
```

From a checkout, run `cargo install --path crates/gy --locked`. The binary is `gy`.

Coming from 0.4, move the ledger with `gy-migrate`; see [docs/migration-0.5.md](docs/migration-0.5.md).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The workspace holds `gy-ledger` (the store, model, and operations) and `gy` (the CLI). `gy-migrate` and its private `gy-core` reader stay until the remaining 0.4 ledgers have moved, then are removed. CI tests on Linux; other platforms are unverified. The screens of `gy serve` have their own end-to-end tests under `e2e/` (Playwright, chromium): see [e2e/README.md](e2e/README.md); they are not part of `cargo test`.
