---
name: gy-ledger
description: "Use when you operate nodes and edges in a gy ledger: how needs, requirements and acceptance criteria connect and close, and how a ledger shared with a team behaves. How to work is in gy-loop."
---

# The gy ledger

How to work (restore, pick one, advance, ask, stop, satisfy) is in `gy-loop`. This is what the nodes and edges mean. The shape of every command is in `gy cheat`.

## Needs, requirements, acceptance criteria

For a criterion that must wait, use `gy edit <AC> --reason "date the wait"`
with `--set due=YYYY-MM-DD --set responsible=<who> --set then="<action>"`.
`due` is the reader's local date; if unknown, record when you will decide it.
`responsible` or `then` without `due`, and invalid dates, are refused. You may
still satisfy early with evidence under the existing approval rule.
`gy show <AC>` displays all three without `--full`; `gy handover` lists due unmet
criteria and summarizes future ones by count and earliest date; `gy next` shows
each ready need's targeted waits as due or waiting. The latter two honor
`--scope` and `--json` and omit satisfied waits. See `gy cheat` for the command shape.

- A need points at its acceptance criteria with `targets`. `need add` prints `id: <ID>` first, and `next` shows the command that could follow. The body goes in with `--body-file` at creation.
- A requirement is filed with `req add` and tied to the needs it serves (`--need`), the decisions it relies on (`--relies-on`) and the criteria it targets (`--targets`).
- What must come first is an edge: `depends-on` and `waits-on`. `next` reads them.
- A need becomes `done` by derivation when at least one of its `filed-as` requirements is `done`, none of them is still `filed` or `approved`, and every acceptance criterion it `targets` is satisfied; a `cancelled` requirement counts on neither side, so it neither completes the need nor holds it open, and a need whose requirements are all `cancelled` asks for a new `filed-as` requirement. A targeted criterion that no requirement still being built (`filed` or `approved`) covers shows as `unmet criterion <id>` under `missing`, with the command that would cover it under `next`. Closing a need with `need close` does not satisfy its criteria: the close lists the unmet ones under `missing`, so satisfy them with evidence (`criterion satisfy`) or drop them (`link --remove <need> targets <ac>`).
- Dates print in your local time. When you tell another writer a point in time, use the `seq` or an ID, not a date.

## A team's ledger (a `remote` in gy.toml)

The team is people, each with their own machine and their own copy; `gy remote set` and `gy remote join` are the person's steps (the README's "Working as a team"), and an agent does not join. Agents on one machine already share the local ledger and need no remote. (The old names `share` / `join` / `sync` still work but are deprecated.)

- Write as before. The push happens in the background, and `gy remote sync` does it on request. `handover` starts with the number of writes not yet pushed and the last sync.
- `notice: your write seq … did not land` on stderr means the other side changed the same node first and your write was not placed. Read the ledger again and redo it if it still applies.
- `undo` takes back only your own write.
