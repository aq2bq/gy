gy keeps the state of work, up to the point a requirement is confirmed, as a graph. Every write is appended as one transaction, and an invalid write is refused when it is made.

A write that creates a node (`need add` / `question add` / `criterion add` / `decide` / `req add`) prints `id: <ID>` first. Other writes answer with the ID you already know. A flag shown with `...` takes one value each time: repeat the flag (`--targets A --targets B`). Every write then prints `changed:`, `missing:`, `next:` and `unresolved:`, empty where it has nothing to say; `unresolved` names a `narrows` / `supersedes` mark an edit left without its passage.

Starting a repository:
  gy init <scope>                                      write gy.toml with one scope here; a gy.toml already here is reported, not touched

Starting a session:
  gy handover
  gy next
  gy show <ID>

Reads:
  gy show <ID|ref>... [--full]                         show nodes, with what each still lacks
  gy list [--type] [--status] [--targets] [--grep] [--actor] [--since]   list nodes; with --actor / --since, list write units. Spelling ignores case. --since takes a seq or a date (YYYY-MM-DD, from midnight where you are)
  gy next                                              the needs whose prerequisites are settled
  gy handover                                          requirements in progress and the counts a session needs to resume
  gy publish [--scope] [--since] [--out]                the publication: every node in range verbatim, the history, the diagnostics. Commit it to look back later
  gy serve                                              read the ledger in a browser: 127.0.0.1, GET only, no path that writes, until stopped. Opens the browser when started from a terminal

Where the ledger lives (experimental; only for sharing with a team. With no remote in gy.toml none of these is ever used and gy stays local. remote set is the person's step, not yours: it writes gy.toml and uploads the ledger. The old names share / join / sync still work but are deprecated):
  gy remote set <URL>                                  start sharing (experimental): checks the remote, writes remote into gy.toml, uploads the ledger, says how to protect it and how to invite
  gy remote join                                       join (experimental): what is needed and how to fix it, fetches the copy, says who you write as and what comes next. Idempotent
  gy remote sync                                       sync with the remote (experimental): fetch the copy, push what is not pushed, pull and rebase. On trouble, the cause and the way out

Writes:
  gy need add "<title>" --targets <AC>... [--spawned-by <D>] [--body-file <path>]
  gy need close <ID> --by fact|external --evidence <text>
  gy question add "<title>" --decider <name> --options <text>... [--body-file <path>]        at least two options
  gy question close <ID> --by fact|decision|non-decision --evidence <text> [--decision <D>]
  gy criterion add "<title>" [--body-file <path>]
  gy criterion satisfy <AC> --evidence <text> [--revoke]
  gy req add "<title>" --need <N>... [--relies-on <D>]... [--targets <AC>]... [--ref <URL>] [--body-file <path>]
  gy req approve <ID|ref> --design <text> --heard-by <name> --evidence <text>
  gy req revise <ID> --reason <text> --source <text>
  gy req done <ID> --evidence <text>
  gy req cancel <ID> --reason <text> --source <text>
  gy decide "<title>" --scope-note <text> [--body-file <path>] [--closes <Q>]... [--relate <relation> <D> --mark <text>] [--source <text>]
        --mark is not a note: it is the exact text in the older decision that stops applying
  gy link <from> <relation> <to> [--mark <text>] [--remove]
        relations: targets, filed-as, relies-on, depends-on, waits-on, raised, spawned-by, closes, narrows, widens, supersedes, completes. --mark only with narrows and supersedes
  gy edit <ID> --reason <text> [--title] [--body-file] [--set k=v] [--append k=v]
        free attributes are strings. --set overwrites, --set k= removes, --append adds one line
        --set scope=<name> moves the node to a scope in gy.toml. --set decision_scope=<text> records a missing scope note, once
        the edit is applied; a later decision's mark that stops resolving is named on unresolved:
  gy scope rename <old> <new>
        moves every node of the old scope to the new name and rewrites gy.toml, keeping comments and order
  gy undo --reason <text>
        takes back the last write only. Typed again, it takes back the undo (redo). There is no way to take back two

gy.toml holds the scope names and, if you want, where publish writes. Keys outside a table come before the first [scopes.*]:

  output = "docs/publication"

  [scopes.myproject]

Name yourself before a write. A read needs no name.
  export GY_ACTOR=<name>
