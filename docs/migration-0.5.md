# Moving a ledger from 0.4 to 0.5

gy 0.5 is an incompatible version that changes the storage format and the set of operations. An existing 0.4 ledger is moved once to the 0.5 canonical store with the bundled `gy-migrate`. This document covers what changes, how to move, and what cannot be carried over.

`gy-migrate` is in the repository up to 0.6.1 and can be taken from the tag v0.6.1.

## What changes

| 0.4 | 0.5 | What the user does |
| --- | --- | --- |
| The ledger is a set of files inside the repository, managed with its own git and worktree | The canonical store is an append-only log outside the repository (`$XDG_DATA_HOME/gy/<hash>/`). The only thing placed in the repository is `gy.toml` | The git and worktree for the ledger are no longer needed |
| IDs are `N-18`, `D-164` or Issue numbers | gy assigns IDs of a prefix + a short hash, such as `n-3f9a` and `d-a1b2` | Old IDs remain as aliases and can be looked up with `show`. Old IDs in documents need not be rewritten |
| A requirement's ID is the Issue number. The parent Issue is held in the configuration | A requirement has a gy ID, and its outward reference is one `ref` | `req add ... --ref <URL>`. It is refused if an unfinished requirement with the same URL exists |
| Requirements have many states, and the record for each transition is set up in the configuration | The states are four: filed → approved → done, and cancelled. After approval gy holds only the records of revision, completion and cancellation | Put the records of implementation, audit and PRs in the external tracker. Do not copy them into gy |
| `gy.toml` holds many settings | Only `[scopes.<name>]` and the output directory. Other keys are refused at load time | Nothing needs to be done about the settings that went away |
| Operated with the writer limited to one | Each agent names itself with `GY_ACTOR` and writes for itself | `export GY_ACTOR=<name>` in each pane |
| There is a pass that checks consistency later | An invalid write is refused at write time, and `handover` reports the states that need attention as counts | The habit of checking later is not needed |
| The number of bearers and the state of a need are updated by hand | Derived from the graph (not stored) | Stop updating by hand |
| Output for people to read is made by a separate projection | `publish` writes the record into a directory per scope (one file per node + an index) | The way to read changes to `publish` |

## How the operations correspond

| Intent | 0.4 | 0.5 |
| --- | --- | --- |
| Create a need | Add a need and update the number of bearers by hand | `need add "<title>" --targets <AC>...` |
| Close a need | There was no way | `need close <ID> --by fact\|external --evidence <text>` |
| Create and close a question | Add a question and close it in one of 3 ways | The same. When closing with a decision, `--decision <D>` also makes the edge |
| A question belongs to a need | The question's `belongs-to: [N-x]` | A `waits-on` edge from that need to the question (`link <N> waits-on <Q>`) |
| Other attributes | Attributes of each node | Carried over as free attributes (an array joins its elements with newlines, an object becomes a JSON string) |
| Number a decision | Write an ADR and import it | `decide "<title>" --scope-note <scope note> [--body-file <path>] [--closes <Q>] [--relate <relation> <D> --mark <text>]` |
| File a requirement | Add a requirement, associate the needs, and update the attributes by hand | `req add "<title>" --need <N>... [--relies-on <D>]... [--targets <AC>]... [--ref <URL>]` |
| Approve, revise, complete, cancel | Transition commands and declaration options | `req approve` / `req revise` / `req done` / `req cancel` |
| Acceptance criteria | Add and satisfy | The same. `--revoke` takes back a satisfaction |
| Edges | Record the same edge on both sides | `link <from> <relation> <to> [--mark <text>] [--remove]`. Stored on the from side only; the reverse direction is derived |
| Fix the body or attributes | Set an attribute | `edit <ID> --reason <text> [--title] [--body-file] [--set k=v] [--append k=v]`. States and edges cannot be changed |
| Undo | Revert with git | `undo --reason <text>` |
| Read | List, search, show, handover, consistency check | `show` / `list` / `next` / `handover` / `publish` |

## The migration procedure

The migration is done once, and is run while the new canonical store does not exist yet.

```sh
export GY_ACTOR=<name>
gy-migrate <0.4 ledger directory> \
  --root <repository root> \
  --ref-base <Issue URL prefix> \
  --publication <freeze destination>
```

- `<0.4 ledger directory>` is the place that holds the 0.4 `gy.toml` and the scope directories.
- `--root` is the repository root where the new `gy.toml` is placed.
- `--ref-base` makes a `ref` (`<prefix>N`) from a requirement's old `#N`. If it is omitted, no ref is attached.
- `--publication` is where the frozen records and `migration-report.md` are written; if it is omitted, it is `<root>/.gy-migration/`.
- `--dry-run` writes nothing and only reports what it found.
- The write is done in one transaction. It is refused when the new canonical store already exists.

The migration reports the following.

- The counts per node kind before and after the migration, the number of aliases attached, the number of decisions whose scope note is unrecorded, the number of edges turned into `waits-on`, and the total number of edges.
- The requirement states that could not be mapped to the four states, and the mapping of each requirement's state.
- The number of frozen records and where they are placed.
- It writes `[scopes.<name>]` into `<root>/gy.toml`. When a `gy.toml` already exists it is not overwritten, and it is an error if that file does not declare the scopes the ledger uses.

After the migration, confirm with `gy handover` that the requirements in progress appear with their `ref`. They can also be looked up by the old ID, as in `gy show '#6027'` or `gy show N-18`.

## What cannot be carried over

- When what a need was waiting on is a requirement, it is not made into a `waits-on` edge (the target of the edge is limited to questions). If needed, keep it as a free attribute with `edit`.
- A decision with an empty scope note is moved with the unrecorded marker attached. It appears as a count in `show` and `handover`. Whether to fill it in is judged per ledger.
- gy assigns the numbers of decisions. Whether to match the numbers of the ADR files is left to how that project is run.
