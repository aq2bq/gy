---
name: gy-ledger
description: "Use when you operate nodes and edges in a gy ledger: how needs, requirements and acceptance criteria connect and close, and how a ledger shared with a team behaves. How to work is in gy-loop."
---

# The gy ledger

How to work (restore, pick one, advance, ask, stop, satisfy) is in `gy-loop`. This is what the nodes and edges mean. The shape of every command is in `gy-loop`'s `CHEATSHEET.md`.

## Needs, requirements, acceptance criteria

- A need points at its acceptance criteria with `targets`. `need add` prints `id: <ID>` first, and `next` shows the command that could follow. The body goes in with `--body-file` at creation.
- A requirement is filed with `req add` and tied to the needs it serves (`--need`), the decisions it relies on (`--relies-on`) and the criteria it targets (`--targets`).
- What must come first is an edge: `depends-on` and `waits-on`. `next` reads them.
- A need becomes `done` by derivation when its `filed-as` requirement is done. Closing a need with `need close` does not satisfy its criteria: the close lists the unmet ones under `missing`, so satisfy them with evidence (`criterion satisfy`) or drop them (`link --remove <need> targets <ac>`).
- Dates print in your local time. When you tell another writer a point in time, use the `seq` or an ID, not a date.

## A team's ledger (a `remote` in gy.toml)

- Write as before. The push happens in the background, and `gy sync` does it on request. `handover` starts with the number of writes not yet pushed and the last sync.
- `notice: your write seq … did not land` on stderr means the other side changed the same node first and your write was not placed. Read the ledger again and redo it if it still applies.
- `undo` takes back only your own write.
