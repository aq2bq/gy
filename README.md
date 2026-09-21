<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/logo.svg" alt="gy" width="200">

# gy — good,yes

English | [日本語](README.ja.md)

gy records the state before a requirement is confirmed. It holds the needs, questions, decisions, requirements, and acceptance criteria that a piece of work rests on, as a graph of nodes and edges, with one way to use it and almost nothing to configure. Agents write it as they work, and `publish` writes the record to a file that is committed and read back later. The canonical ledger lives outside the repository. gy makes no network calls unless you share the ledger with a team (experimental), and sharing goes through git.

## What changes for you

You do not operate gy; your agent does. What you get is the work that stops being yours.

- **No re-onboarding.** After a reset you no longer type "here is the issue we are on, the process is in this file, last time we finished that PR, next is X". The agent starts from `gy handover` and `gy next`.
- **Reset any agent at any time.** A lead, a requirements agent and an implementer can each be cleared without a handoff note, because none of them was holding the state.
- **A correction is one sentence wide.** When you change your mind, or ask for something that contradicts what you said earlier, the new decision has to quote the passage of the old one that loses effect. The rest stays in force, and every later session sees that passage marked as retracted. gy does not find the contradiction for you: the agent meets the old decision because the work it picks up is linked to it.
- **What stays with you is deciding.** Goals, needs, and the questions the agent brings back with options and a recommendation. The entry skill, `gy-loop`, opens with the same sentence: "A person cannot escape the critical decisions. gy frees them from everything else."

This is the author's experience of daily use, not a guarantee. gy checks the record, not the work outside it.

## Try it

Two steps, both done by talking to your agent.

1. Tell your agent once:

   > Install gy with `cargo install gy --locked`, then copy the skills under the installed crate's `skills/` directory to where you read skills (Claude Code: `~/.claude/skills/`). "Install" and "Where the skills go" in gy's README have the paths.

2. Add one line to the project's `CLAUDE.md` or `AGENTS.md`:

   > This project tracks its progress in **gy**. Before you start or resume anything, read the `gy-loop` skill and follow the record.

Then ask for work as you always do. The first time, the agent runs `gy init` and asks you two things once: whether you want to read and approve requirements yourself, and whether it should ask or go on when a need is unclear. After any reset it resumes from the record.

To look at the record yourself, run `gy serve`. The screenshots show a demo ledger built by `scripts/demo-ledger.sh`; run it and `gy serve` to see one before you have your own.

The now page

<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/serve-now-en.png" alt="The now page" width="100%">

The fractal graph

<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/serve-graph-en.png" alt="The fractal graph" width="100%">

One node

<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/serve-node-en.png" alt="One node" width="100%">

## What gy is for

Work handed to an agent does not need to be read while it goes well. Because it is not read, it stops being read. How far the work gets then depends on how much fits in the context window, how large the target is, and how strong the model is, and none of that shows while things go well.

It shows at the end. A requirement has to be declared complete. A decision that earlier work relied on has been replaced without anyone noticing. Leftover work has to go somewhere. And the one person accountable is holding several projects at once and cannot read them all. None of the four happens while things go well. When one does, nobody remembers what the work was based on.

gy exists for that end. It holds no plan and no schedule. It records what each piece of work rests on and what must hold before the work can be called done, and it reports where those records no longer fit together: a criterion left unmet after every need has closed, a question nobody waits on. It does not judge whether two decisions contradict in meaning. There is no `lint` pass to run later: an invalid write is refused when it is made, and what needs attention is counted by `handover`.

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

A decision is stored with its scope note (where it holds), so a later reader can tell where it does and does not hold. `narrows` and `supersedes` also name the passage of the older decision that loses effect; the mark is checked against that decision's text when the edge is written.

## How a requirement moves

A requirement has four states: `filed` (registered), `approved` (design confirmed), `done` (shipped), and `cancelled`. Approval, revision, completion, and cancellation are each one command that records who heard the design, the evidence, and the reason, and every other transition is refused.

| From | To | Command |
| --- | --- | --- |
| filed | approved | `req approve` |
| approved | filed | `req revise` |
| approved | done | `req done` |
| filed, approved | cancelled | `req cancel` |

Approval is the gate for outside work. gy cannot see what is built outside the ledger, but it refuses to record its result: `criterion satisfy` is refused unless an approved or done requirement `targets` the criterion, and the refusal says the command that comes next (`req add …` or `req approve …`). While a requirement is approved, its title, body, `targets` and `relies-on`, and the title and body of the criteria it targets, cannot be edited; `req revise` sends it back to filed first. Who approves is your policy, not gy's: the bundled `gy-loop` skill has the agent ask once whether you read every requirement, only the first, or leave it to the agent.

gy holds nothing about the work after approval except these four records. Where the implementation is tracked, how it is designed, and when it is audited belong outside gy, in the issue and pull request that the requirement's reference points at.

## Where the ledger lives

The canonical ledger is an append-only event log outside the repository, under `$XDG_DATA_HOME/gy/<hash of the repository root>/`. The repository itself holds only `gy.toml`. Every write is one transaction appended to the log with the sequence number, time, actor, reason, and source; nothing is edited in place.

`undo --reason <text>` inverts the last transaction as a new one, so the history keeps both the mistake and the correction. It undoes one transaction only; a second undo undoes the first undo (a redo). The log is the ledger; a snapshot file alongside it only speeds up opening and can be deleted.

## The twenty-six operations

A write adds a transaction of your own to the ledger: it names who wrote and why, and `undo` applies to it. A read leaves the ledger as it is. Three operations are neither; they decide where the ledger lives.

Before the first write (1):

| Operation | Result |
| --- | --- |
| `init <scope>` | Start a repository here: write `gy.toml` with one scope, then name the skill to read and the first node to file. A `gy.toml` already here is reported, not touched |

Reads (6):

| Operation | Result |
| --- | --- |
| `show <ID\|ref>... [--full]` | One or more nodes by ID, alias, or reference, with what each still lacks |
| `list [--type] [--status] [--targets] [--grep] [--actor] [--since]` | Node rows, or write units when `--actor` or `--since` is given. The type and status words ignore case, and `--since` takes a sequence or a date (`YYYY-MM-DD`, from that day's start in your own place) |
| `next` | The needs whose prerequisites are settled |
| `handover` | In-progress requirements and the counts a session needs to resume |
| `publish [--scope] [--since] [--out]` | Write the record at a point and range into a directory: one file per node and a scope index |
| `serve` | Read the ledger in a browser, on 127.0.0.1 (GET only, no write path), until stopped. It opens the browser when started from a terminal |

Where the ledger lives (3, experimental):

These exist only for sharing with a team. With no `remote` in `gy.toml`, none of them is ever used and gy stays local. They change things outside the ledger's own history: `share` writes `gy.toml` and uploads the ledger, and `sync` pushes commits.

| Operation | Result |
| --- | --- |
| `share <URL>` | Start sharing (experimental): check the remote, write `remote` into `gy.toml`, upload the ledger, print the protection and the invitation |
| `join` | Join (experimental): check what you need with the fixes, fetch the copy, say who you write as and what is next; harmless to repeat |
| `sync` | Sync with the remote (experimental): fetch a missing copy, push unpushed writes one commit each, take in a remote that moved ahead and re-seat your writes; on failure, say why and what to do |

Writes (16):

| Operation | Result |
| --- | --- |
| `need add "<title>" --targets <AC>... [--spawned-by <D>] [--body-file <path>]` | File a need against acceptance criteria |
| `need close <ID> --by fact\|external --evidence <text>` | Close a need without a requirement |
| `question add "<title>" --decider <name> --options <text>... [--body-file <path>]` | Open a question with a decider and at least two options |
| `question close <ID> --by fact\|decision\|non-decision --evidence <text> [--decision <D>]` | Close a question and, when decided, record the decision |
| `criterion add "<title>" [--body-file <path>]` | Add an acceptance criterion |
| `criterion satisfy <AC> --evidence <text> [--revoke]` | Record that a criterion holds, or revoke it |
| `req add "<title>" --need <N>... [--relies-on <D>]... [--targets <AC>]... [--ref <ref>] [--body-file <path>]` | File a requirement against needs, decisions, and criteria |
| `req approve <ID\|ref> --design <text> --heard-by <name> --evidence <text>` | Confirm the design of a requirement |
| `req revise <ID> --reason <text> --source <text>` | Send an approved requirement back to filed |
| `req done <ID> --evidence <text>` | Record that an approved requirement shipped |
| `req cancel <ID> --reason <text> --source <text>` | Cancel a requirement that was not done |
| `decide "<title>" --scope-note <text> [--body-file <path>] [--closes <Q>]... [--relate <relation> <D> --mark <text>] [--source <text>]` | Create a decision, close questions, and record one lineage edge |
| `link <from> <relation> <to> [--mark <text>] [--remove]` | Add or remove one edge |
| `edit <ID> --reason <text> [--title] [--body-file] [--set k=v] [--append k=v]` | Change a node's title, body, or free attributes. A free attribute is a string; `--set` overwrites it, `--set k=` drops it, and `--append` adds one line, separated by a newline. `--set scope=<name>` moves the node to a scope gy.toml declares, and `--set decision_scope=<text>` records a decision's unrecorded applicability conditions once |
| `scope rename <old> <new>` | Move every node of a scope to a new name and rewrite gy.toml, keeping its comments and order |
| `undo --reason <text>` | Invert the last transaction |

Every write prints what it changed, what the node still lacks, and the shape of the command that could come next, so the next step is visible without a separate instruction sheet. A write that creates a node (`need add`, `question add`, `criterion add`, `decide`, `req add`) prints `id: <ID>` as its first line. Pass `--json` for the same content as data.

## Dates and times

A node's `created`, a criterion's `satisfied_at` and a requirement's recorded dates are stored as UTC instants; `show` and `list` print them in your own time zone (`TZ`) as `YYYY-MM-DD HH:MM`, while `--json` and `publish` keep the stored value (`2026-09-18T07:28:56Z`). To name a point in time to another agent, use the write sequence or a node id, not a date.

## Resuming a session

A new session starts with three commands. `handover` shows the in-progress requirements with their references, the number of open questions, the number of ready needs, and the errors and warning counts. `next` lists the needs whose prerequisites are settled, and the agent presents one of them to the person it works for. `show` reads one node in full.

```sh
gy handover
gy next
gy show n-3f9a
```

## gy.toml

The only configuration is a scope name and, if wanted, an output path for `publish`. Anything else is refused when the file is read.

```toml
# Optional. publish writes here when --out is not given.
output = "docs/publication"
# Optional (experimental): the ledger-only git repository. See "Working as a team".
remote = "https://github.com/you/yourproject-ledger.git"

[scopes.myproject]
```

Reads cover every scope. A write needs a scope only when the file names more than one; pass `--scope <name>` to choose. The first write creates the ledger, and reads never do.

## Working as a team (experimental)

A team shares one ledger through a ledger-only git repository. The members are people, each on their own machine and with their own copy of the ledger: the person starts it with `gy share` and another joins with `gy join`, and each person's agent writes to that person's copy. Two agents on one machine already share the local ledger and need no remote. There are two procedures, and in both gy says what to do next.

**Start sharing (the one who used gy alone).** Create an empty private repository on GitHub and run, in a checkout of the project:

```sh
gy share https://github.com/you/yourproject-ledger.git
```

It checks the remote (empty or ledger-only, and that you can push), writes `remote` into `gy.toml`, uploads the ledger you have as it is, and prints how to protect the branch (require linear history, block force pushes; gy changes no settings) and the invitation to send a member. Commit `gy.toml` with the project.

This step is yours, not your agent's. `gy share` uploads the whole ledger to that repository; an agent's runtime may treat the upload as sending data out and refuse it, and an agent cannot grant itself the permission. Run `gy share` yourself, once, as you create the repository and protect its branch yourself, or allow `gy share` and `gy sync` in the agent's settings yourself. After that the agent writes as before and the pushes happen in the background.

**Join (the one invited).** Get write access to the ledger repository, clone the project, and run:

```sh
gy join
```

It checks what you need in one go (git, credentials that can read the repository, and a name to write under — `git config user.name` and `user.email`, or `GIT_AUTHOR_NAME` and `GIT_AUTHOR_EMAIL`) and, if something is missing, lists each with the fix and stops. When all is there it fetches your copy, says who you will write as (`user.name / GY_ACTOR`) and what to do next (`gy handover`). Running it again is harmless. Starting with `gy handover` instead also fetches the copy and prints the same "joined" line.

**From then on, use gy as before.**

- A write lands in your copy at once and is pushed in the background (right after the write, and every ten seconds while `gy serve` runs). `gy sync` syncs explicitly. While the remote is unreachable, reads and writes keep working, and what piled up is pushed when it is back.
- If the remote moved ahead, your unpushed writes are re-seated after it. Only a write to a node the other side changed first is rejected, and only you are told: on stderr at your next gy command and in `handover`. Whether to redo it is your call.
- A writer is recorded and shown as `user.name / GY_ACTOR`; the same agent name under two humans is two writers. The name is the one git signs your commits with, resolved git's way — `GIT_AUTHOR_NAME` first, then `git config user.name` — so the ledger and the git history never name two people for one write.
- The remote is written by gy alone: one write is one commit, and a history changed outside gy is refused with the way back. A repository holding anything but a ledger is refused.
- Remove the `remote` line from `gy.toml` and the copy is local again (the next command says so once). Reconnecting after both sides moved is refused: there is no merge.

A ledger holds the exchanges behind decisions. Putting it on a remote means that record is on GitHub.

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

`publish` writes the record at a point and range into a directory to commit: one file per node under `<out>/<scope>/<kind>/`, and a scope index at `<out>/<scope>/README.md`. It is a development artifact: later, an agent reads it to review what was decided and why, and diffs one publication against the next. It is not reading matter for the person the agents work for, and gy adds no human-facing output format.

A node file holds the id with its old aliases and reference, the title, scope, creation date, state, scope note (where it holds), the body, both edge directions with the other side's id, alias, title, and mark, the closure and evidence, the requirement records, and the free attributes. Every reference carries the target's title, so each file stands on its own. A decision file puts its lineage relations first.

The index holds the generated time, the log sequence, the scope and range, the writer, and the canonical location; a short "how to read" section; a per-kind list with a link and state for each node; the write history; and the diagnostics.

`--out` names the output directory, `gy.toml`'s `output` names a default, and without either publish is an error. Only the target scopes' directories are removed and rewritten; other files under `--out` and other scope directories are left alone.

## Install

```sh
cargo install gy --locked
```

From a checkout, run `cargo install --path crates/gy --locked`. The binary is `gy`.

Coming from 0.4, move the ledger once; [docs/migration-0.5.md](docs/migration-0.5.md) has the command and what cannot be carried over.

## Where the skills go

The agent skills (`gy-loop`, the entry point, with the cheat sheet beside it; `gy-ledger`, `gy-question`, `gy-decide`) ship inside the crate under `skills/`, but `cargo install` does not place them. Copy them to where your agent reads skills (for example `~/.agents/skills`).

```sh
# from a checkout
cp -R crates/gy/skills/gy-* ~/.agents/skills/
# from the registry copy (match the version)
cp -R ~/.cargo/registry/src/*/gy-1.0.1/skills/gy-* ~/.agents/skills/
```

A release that changed the skills says so under Updating in the changelog; copy them again when you move to that release, and check that your agent lists every gy skill afterwards: where an agent reads skills through per-skill links (for example `~/.claude/skills/gy-ledger` → `~/.agents/skills/gy-ledger`), a new skill needs a link of its own. When the ledger format moves up, gy prints one line to stderr right after the migration, pointing at the changelog's Updating section.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The workspace holds `gy-ledger` (the store, model, and operations), `gy-serve` (the read-only web view), and `gy` (the CLI). CI tests on Linux; other platforms are unverified. The screens of `gy serve` have their own end-to-end tests under `e2e/` (Playwright, chromium): see [e2e/README.md](e2e/README.md); they are not part of `cargo test`.
