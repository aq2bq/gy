# Changelog

## 1.0.0 - 2026-09-18

### Updating

This release changes how work is recorded, so read this before you upgrade.

- **An achievement needs an approved requirement.** `criterion satisfy` is
  refused unless an approved or done requirement `targets` the criterion,
  and an approved requirement (and the criteria it targets) cannot be
  edited until `req revise`. What a ledger already holds stays valid; the
  rule judges new writes. A need that has no requirement yet gets one when
  you come to satisfy its criteria: gy says which command to type.
- **The ledger format moves from 3 to 4**, automatically and without
  touching the log (`format.3.bak` is kept). An older gy cannot open a
  format-4 ledger, by design: it does not know the rule.
- **On a shared ledger, the first upgraded copy raises the remote's format
  at its next sync.** From then on an older gy stops at sync with
  `the remote ledger is format 4 … update gy`; its unpushed writes stay in
  its copy and land, judged by the rule, once it is upgraded. Upgrade a
  team together.
- **What gy prints is in English**, including the phrases under `missing:`
  and `next:`. An agent that matched the Japanese phrases must match the
  English ones.
- The subcommands and options are those of 0.9.0 plus `share`, `join` and
  `sync`; the only new `gy.toml` key is `remote`.

```
cargo install gy --locked    # gy 1.0.0
```

For an agent driving gy on a ledger that a team shares (a `remote` in
`gy.toml`), this is what to expect:

- **Write as before.** A write lands in your copy at once and is pushed
  in the background. You do not run `gy sync` after writing; run it when
  you want the remote's writes now, or when `handover` says something is
  not pushed.
- **Read `handover` first.** On a shared ledger it fetches the remote
  first and may start with `sync: N writes not pushed; remote last
  reached …`. Two things can appear that did not before: a
  `notice: your write seq N (…) did not land: …` line on stderr of any
  command, and the same line under `handover`'s errors. It means the
  remote changed a node your write touched first; read the node again and
  redo the write if it still applies. The notice goes away when you next
  write to that node.
- **Name yourself.** A write on a shared ledger needs `git config
  user.name` (and `user.email`); without a name it is refused. Writers are
  shown as `<user.name> / <GY_ACTOR>`, and `list --actor` matches either.
- **`undo` is your own.** Undoing another writer's last write is refused.
- **Name a point in time by `seq` or an id**, not by a date; dates print
  in each reader's own time zone.

To start sharing a ledger, create an empty private repository for the
ledger alone and run `gy share <its URL>`; a member gets write access to
that repository and runs `gy join` in a checkout of the project. Both
commands say what is missing and what to do next. The bundled skills changed
with this release: `gy-loop` is new and is the entry point (first agree
once on how to work — who approves requirements, what to do with an
unclear need — then the loop: restore, pick one, advance, ask, stop,
satisfy), the skills are now in English, the
cheat sheet moved next to it (`skills/gy-loop/CHEATSHEET.md`), and
`gy-ledger` keeps the meaning of nodes and edges and gained a section on
shared ledgers. Copy them again as the README says; `cp -R …/skills/gy-*`
brings the new one along.

### Added

- **`gy share <URL>` and `gy join`** (n-57c5, d-b1d4): the two procedures
  of sharing a ledger have names. `share` checks the remote (empty or
  ledger-only, pushable), writes `remote` into `gy.toml`, uploads the
  ledger as it is and prints how to protect the branch and how to invite
  a member; `join` checks what a member needs (git, credentials,
  `user.name` and `user.email`) with the fix for each, fetches the copy
  and says who they write as and what to do next. Both are harmless to
  repeat, and the first gy command on a fresh clone still fetches the
  copy and prints the same "joined" line.
- **Experimental: `gy sync` and a `remote` in `gy.toml`** (n-6f47, n-8a52,
  n-f4cd; d-39f6, d-1e50). A ledger can name a ledger-only git repository as
  its remote. The copy on each machine becomes a working tree of it: the
  first `gy sync` uploads the whole log as one commit, a machine with no
  copy clones it on its first gy command, every later write is pushed by
  `gy sync` as one commit whose subject is the why and whose trailer is
  `Gy-Seq`, and a remote that moved ahead fast-forwards the copy. Writes
  still land locally at once; only `gy sync` talks to the remote for now.
  A write to a synced copy records who wrote it from `git config
  user.name` and `user.email` (`by`, `by_mail` on the event line) and is
  refused without a name. A remote holding anything but a ledger, a
  history changed outside gy (no `Gy-Seq` trailer, a rewritten past, a
  file gy does not write, a rewritten log) and two copies that both moved
  are refused with the way out; gy never force-pushes. Removing `remote`
  from `gy.toml` makes the copy local again after one stderr notice.
  A ledger without a remote is untouched; the only new command is `sync`
  and the only new key is `remote` (put it before the first `[scopes.*]`
  table).
- **gy says when it migrated a ledger** (n-f8a2): the first open that
  moves the format up prints one line to stderr, naming the build, the
  versions and the changelog's Updating section. stdout and `--json` are
  untouched, and the next open is silent. The READMEs now say where the
  bundled skills go and that a release which changed them says so under
  Updating.
- **A write to a synced copy is pushed in the background** (n-ecbf 2A):
  after each successful write, gy starts a detached `gy sync` that pushes
  the unpushed lines, logging to `sync.log` in the copy, never prompting,
  giving up after 30 seconds and never running twice at once. `gy sync`
  holds the ledger lock only while it changes the copy, so a write never
  waits on the network. The copy keeps `sync.state`, and `handover` on a
  synced copy prints `sync: N writes not pushed; remote last reached …`
  with the last error's first line (the full text under `--json`). A copy
  that cannot reach its remote keeps accepting reads and writes.
- **Unpushed writes are re-seated after a remote that moved ahead, and
  only the ones that no longer hold are rejected** (n-ecbf 2B): sync takes
  the remote's lines, then keeps each local line unless a node it touched
  was changed or deleted remotely, an edge points at a node that is gone,
  its id collides, or it depends on a rejected line; kept lines get new
  sequence numbers. A rejected line is kept in the copy's `rejected.jsonl`
  and told to its writer only: the next gy command prints
  `notice: your write seq N (…) did not land: …` on stderr and `handover`
  lists it under errors, until that writer next writes to a node the
  rejected line touched. On a synced copy `handover` fetches first (giving
  up after five seconds), and `undo` refuses to undo another writer's
  last write.
- **serve keeps a synced copy in step and shows it** (n-94bb 3A): while
  `gy serve` runs on a synced copy, gy runs a bounded `gy sync` every ten
  seconds and logs to stderr the startup line (port, ledger directory,
  remote) and each round that changed something or failed, in the local
  time. The Now page shows the last sync (time, sequence), how many writes
  are not pushed and the last error's first line; a write not yet pushed
  carries a small mark in the live rail and the history, and the time band
  draws the unpushed range dotted. `/api/now` carries `resume.sync` and
  `resume.errors` on a synced copy only.
- **serve's sync log says where it stopped and what to do** (n-08ae): a
  round that gives up names the step (`while fetching/rebasing/pushing/
  cloning the remote`), every failure is followed by a `→` line with the
  way out, the same failure is logged once and then every ten rounds as
  `still failing since …`, and a recovery is logged once. A fresh copy is
  cloned beside the ledger directory and moved into place.
- **`gy sync` says what went wrong and what to do** (n-94bb 3B): a copy
  whose log holds a line that cannot be read is reported with the count,
  the readable unpushed writes and the way to take the ledger again (a
  torn last line is not damage; the next write drops it); an emptied
  remote is re-uploaded from the copy; a remote whose format is newer than
  this build is refused with both versions; and an unreachable remote, a
  refused push and the other failures carry a second line with the way
  out. gy never deletes a copy or a remote.
- **A writer is shown as the human first** (n-d36d): on a synced copy,
  `list --actor`/`--since` rows, `handover`'s in-progress rows, and the
  page's live rail, history and recent writes name a writer as
  `<git user.name> / <GY_ACTOR>` (`pememo / lead`); two humans under the
  same actor name are two writers, with their own colours and filter
  chips, and `list --actor <x>` matches either the human or the actor.
  `--json` rows carry `who` next to the unchanged `actor`. A ledger
  without a remote reads exactly as before.
- **A copy inside another repository's work tree is never mistaken for
  that repository** (n-6d6c): a copy is a git repository only when the
  ledger directory is its own top level; inside another work tree (a home
  directory kept in a dotfiles repository, say) gy makes a nested
  repository of its own. Before any commit or push the copy's `origin`
  must equal the recorded remote, else gy refuses to touch the directory.

### Changed

- **A failure says what to type next** (n-1de7). A first-time agent given
  only the bundled skills and the README failed 14 gy calls in three
  sessions; each of these now names the fix in its own output:
  - no `gy.toml`: the message keeps `pass -C <dir>` and adds that a single
    line `[scopes.<name>]` starts a repository and the first write makes
    the ledger. The exit code is unchanged.
  - `GY_ACTOR` unset: the message adds `export GY_ACTOR=<name>`.
  - `question add`: the usage line, in `--help` and after a rejected
    `--options A B`, reads `--options <A> --options <B>`; the flag is
    repeated once per option, as before.
  - every flag that repeats (`need add --targets`, `req add --need`,
    `--relies-on`, `--targets`, `decide --closes`, `edit --set`,
    `--append`) says so in `--help`, and the two required ones show the
    repeated form in the usage line a rejected call prints.
  - `next:` leads with `edit <id> --body-file … --reason …` whenever
    `missing:` names a body, for every kind of node (before, only a need
    showed it, and it showed it even when the need had a body). A need
    that has a body no longer lists the edit; its other hints are the same.
  No subcommand, option or exit code changed.
- **The bundled skills say how to work, not only how to type** (n-d053,
  d-8b39, d-f540). `gy-loop` is the single entry point. It says nothing
  about how many agents you run or how they talk to each other. A question
  matches the level it is asked at: about the need while the need is
  unclear (what it is for, what matters most), about the requirement once
  one is filed; the choice among solutions is the writer's to decide and
  record, not the person's to pick from a menu. Who approves a requirement
  is the user's policy (d-5dfe): the skill asks once and records the answer.
- **What gy prints is in English** (n-00d3, d-e999). The phrases under
  `missing:` and the holes in `next:` (`a body (how to measure)`,
  `a filed-as requirement`, `<title>`, `<older D>` …), undo's notes, `show`'s
  `missing:` line, and everything `publish` writes around your content —
  the index's "How to read", the section names of a node file (`Relations`,
  `Where it holds`, `Body`, `Free attributes` …), the history table's
  header, the diagnostics' preface — were Japanese and are now English,
  with ASCII parentheses. What you wrote (titles, bodies, scope notes, free
  attributes) is untouched: a publication of gy's own ledger before and
  after differs in no title and in none of 714 content sections. An agent
  that matched on the Japanese phrases must match on the English ones; the
  field names (`missing`, `next`) and `--json` keys are the same. The web
  UI still follows the browser's language.
- **What waits on a person no longer depends on a title** (n-208f,
  d-b02d). The now view (the web UI's "waiting" and the counts beside it)
  took an open question as waiting on a person only when its decider
  contained `master` or `マスター`, the title of gy's first user. It now
  takes a decider that has never written to the ledger as a person: people
  do not operate gy, so the name that never writes is the person. A decider
  that is one of the ledger's writers (an agent) stays among the other open
  questions. The git name a shared copy records (`by`) is not a writer.
  Both existing ledgers give byte-identical now views before and after;
  a ledger whose questions name their decider by any other name now shows
  them as waiting. No public name or JSON key changed.
- **An achievement needs an approved requirement** (n-f921, d-b0d0,
  d-5dfe). `criterion satisfy` is refused unless a requirement that is
  approved or done points at the criterion with `targets`. gy cannot see
  work that happens outside the ledger, but it can refuse to record its
  result: what was built before anyone approved what would change does not
  count as met. The refusal says the next step (`req add "<title>" --need
  <N> --targets <AC>`, or `req approve <R> …` when a filed requirement
  already targets it). `--revoke` is never refused. The same rule judges an
  undo that would bring a satisfaction back and a rebase on a shared
  ledger: if the other side revised the requirement first, your unpushed
  satisfy is refused with the reason and shows under `did not land`.
  `missing:` on an unmet, uncovered criterion says `an approved requirement
  (targets)`, and `next:` offers `req add`, `req approve` or `criterion
  satisfy` to match. Who approves is not gy's business: `req approve` is
  unchanged, and the bundled gy-loop skill asks the person once. The rule
  looks only at what a write changes, so what a ledger already holds stays
  valid. `gy share` judges its first upload with the same rule.
- **What was approved stays what was approved** (n-a3f2). While a
  requirement is approved, its title, its body and its `targets` and
  `relies-on` edges cannot be edited or re-linked, and neither can the
  title or body of a criterion it targets: `req revise` sends it back to
  filed first, as D-70 always meant. Before, a requirement approved for one
  criterion could quietly take on another and let it be satisfied. State
  moves (done, cancelled, revise), free attributes, a scope move,
  satisfying and revoking pass as before, and filed, done and cancelled
  requirements are untouched. Undo and a shared ledger's rebase are judged
  by the same rule.
- **The ledger format is 4, and a shared ledger keeps the switch**
  (n-96f8, d-bace). An older gy does not know the requirement rule, so
  format 3 would let it write around it. Opening a format-3 ledger copies
  `format` to `format.3.bak` and writes 4; the log and the snapshot are
  untouched. On a shared ledger nothing used to raise the remote's
  `format`, so a pull would have reset the copy to 3 and older builds
  could keep pushing: the copy that is ahead now adds one commit that
  changes `format` alone (trailer `Gy-Format`, no `Gy-Seq`), before its
  writes, or right after taking in a remote that is ahead. The check on
  incoming commits admits exactly that shape and refuses one that lowers
  the format, touches another file, or carries both trailers. gy's own
  ledger and a second real ledger migrated on copies with byte-identical
  logs and publications.

### Fixed

- **Writes typed in quick succession no longer come back as "did not land"**
  (n-6b44). Each write starts a background sync; two of them could fetch the
  same remote ref, one pushed, and the other, refused, rebased and took its
  own already-pushed writes for the remote's change — a false notice that
  told the writer to redo a write that had landed. A rebase now recognises a
  remote event identical to one of its uncommitted writes (everything but
  the sequence) and counts it as pushed, and one sync runs at a time per
  copy (an exclusive lock on the already-ignored `sync.pid`, held for the
  whole sync; a write still never waits on a fetch).

## 0.9.0 - 2026-09-18

### Updating

For an agent that drives gy from the command line, this is the whole
migration. Nothing has to be done to a ledger by hand.

```
cargo install gy --locked    # gy 0.9.0
```

- **What you run does not change.** Every subcommand and option is the
  same as in 0.8.1, and `gy.toml` accepts the same keys. The output shape
  of a write is unchanged: `id: <ID>` first for a write that mints a node,
  then `changed:`, `missing:`, `next:`; the bare id for the others; and the
  same keys under `--json`.
- **What you read changes in four places.**
  1. Dates are moments, shown in your own time zone. `show` and `list`
     print `created` (and a criterion's `satisfied_at`, a requirement's
     approval and completion) as `YYYY-MM-DD HH:MM` in the time zone of
     the process (`TZ`); `--json` and `publish` carry the stored UTC
     instant, `2026-09-18T07:28:56Z`. Do not compare a printed date with a
     stored one; when you need to name a point in time to another agent,
     use the write sequence (`seq`) or the node id.
  2. `--since <YYYY-MM-DD>` starts at that day's midnight in your own time
     zone, not in UTC.
  3. `handover` may print two more warning lines, `open questions nobody
     waits on: N` and `criteria unmet with every need closed: N`. They are
     counts, like the other warnings; find the nodes with `list` and read
     the gap in `show`'s `missing:`.
  4. `missing:` and `next:` say more. An open question nobody waits on
     lists `待つニーズ（waits-on）か生んだ要求（raised）` and offers
     `link <need> waits-on <q>`; a closed need lists the unmet criteria it
     leaves as `未達の受け入れ条件 <ac>` and offers `criterion satisfy`.
     Neither is a rule: a question still needs no need, and a close is not
     refused.
- **The ledger migrates itself.** The first time 0.9.0 opens a ledger it
  moves the format from 2 to 3: the log is not rewritten, an old
  `YYYY-MM-DD` reads as `T00:00:00Z`, the snapshot is rebuilt, and
  `format.2.bak` keeps the old version file. Every ledger a project has
  must then be opened with 0.9.0 or later: a 0.8.x build says `format 3 is
  newer than this build supports` and stops. Update every machine that
  shares a ledger at the same time.
- **A write that races another writer is retried for you.** You no longer
  see `another writer advanced the ledger; reopen and retry`; only a write
  that lost five times fails, with `gave up after 5 retries`. The event
  line then carries `"retries": n`, which a reader may ignore.

### Changed

- **Incompatible: a node's `created` is an instant, not a day** (n-b6b6,
  d-b1f8). It is stored as UTC, RFC 3339 with seconds (`2026-09-18T07:28:56Z`),
  and a new node takes the moment of the write that made it. `show` and
  `list` print it in your own time zone as `YYYY-MM-DD HH:MM`; `--json` and
  `publish` keep the UTC value. `--since <date>` now starts at that day's
  midnight in your own time zone, not UTC. The shared word for "when" in a
  team is the write sequence, not a date. The ledger format moves from 2
  to 3: the log is not rewritten, a reader raises an old `YYYY-MM-DD` to
  `T00:00:00Z`, the snapshot is rebuilt, and `format.2.bak` keeps the old
  version file. Nothing to do by hand; a build older than this one refuses
  the ledger with "format 3 is newer than this build supports". The
  `satisfied_at` of a criterion and a requirement's own dates stay days for
  now (n-86cc).
- **Incompatible: the other recorded dates are instants too** (n-86cc):
  a criterion's `satisfied_at` and a requirement's approval, revision,
  completion and cancellation `at` are stored as UTC RFC 3339 with seconds
  and take the moment of the write. `show` prints them in your own time
  zone; `--json` and `publish` keep UTC. The ledger format stays 3; a
  reader raises an old `YYYY-MM-DD` in any of these fields, in the log and
  in an older snapshot alike, so nothing needs doing by hand. A value
  migrated from 0.4 with a `+00:00` offset is kept as it is and still
  reads as an instant.
- **The page shows every stored instant in the reader's own time zone**
  (n-648d): the history headings, a node's created date, a requirement's
  steps, the list's created column (now `YYYY/MM/DD`) and the "nobody waits
  on this" age all count the browser's own calendar day, through one date
  component (`time.js`). No UTC date is shown anywhere.
- **A write that loses its race with another writer is tried again**
  (n-fe59): where it used to fail with "another writer advanced the ledger;
  reopen and retry", gy now reopens the ledger and runs the same intent
  again, up to five times with a short pause, and the event line carries
  `"retries": n` when it took more than one try (absent otherwise; the
  format stays 3). A write whose intent no longer holds after the reopen
  still fails on its own reason, and a write that loses five times fails
  with "gave up after 5 retries". Two writers creating a ledger at the same
  moment no longer clobber each other's temporary file.
- **`question add` and `show` name the need or requirement an open
  question still lacks** (n-fa11, d-09b6): `missing:` gains
  `待つニーズ（waits-on）か生んだ要求（raised）` until some node points a
  `waits-on` or `raised` edge at the question, and `next:` gains
  `link <need> waits-on <q>`. A question still needs no need to be filed;
  this is advice, not a rule.
- **`handover` counts the open questions nobody waits on** as one more
  warning line, `open questions nobody waits on: N`, absent when N is 0.
  The list itself is not printed, as with the other warnings.
- **`/api/now` and `/api/list` rows carry `created`, and `unwaited: true`**
  on an open question nobody waits on (the key is absent otherwise). The
  page marks such a question on the waiting card and in the list with
  "nobody waits on this" and its age in days.
- **`need close` and `show` name the unmet criteria a closed need leaves
  behind** (n-f7ef, d-f7b6): when every need that targets a criterion is
  closed and the criterion is not satisfied, the close prints
  `missing: 未達の受け入れ条件 <ac>` and `next: criterion satisfy <ac>
  --evidence …`. The close itself is not refused. To withdraw a criterion
  instead, remove the need's `targets` edge to it (`link --remove`).
- **`handover` counts those criteria** as one more warning line,
  `criteria unmet with every need closed: N`, absent when N is 0.

## 0.8.1 - 2026-09-18

### Fixed

- **A write could vanish while another process was opening the ledger**
  (n-6b71). Opening the ledger measured the log's length after parsing it
  and cut anything past the last newline as a torn tail. Between the two
  measurements another process could finish a whole line, which the
  reader then deleted: the writer had already printed its success and
  the next writer reused the sequence, so the loss left no trace in the
  log. With `gy serve` open, which reopens the ledger after every write,
  a write fired right after another was lost 9 times in 40. Readers now
  leave the log alone; only a writer drops a torn tail, under the
  exclusive lock, right before it appends, and a line is appended in one
  write. Nothing in the API, the CLI, the store format, `gy.toml`, the
  diagnostics or the exit codes changed, and no ledger needs migrating.
  A write lost before this fix cannot be recovered; the only sign is an
  id in an agent's output that `show` cannot find.

### Updating

```
cargo install gy --locked    # gy 0.8.1
```

## 0.8.0 - 2026-09-17

### Changed

- **The first line of a write that mints a node is labelled**: `id:
  n-1d12` where it used to be the bare `n-1d12`. Anything reading the id
  out of the output has to read it as `^id: ` now — gy's own test helper
  made exactly that assumption and had to be fixed. `--json` is
  untouched and still carries `id`.
- **Incompatible**: `NeedAdd`, `QuestionAdd`, `CriterionAdd` and `ReqAdd`
  each carry a `body`, so a caller that builds one as a struct literal
  has a field to fill (`None` keeps the old behaviour). Nothing else in
  the API, the CLI, the store format, `gy.toml`, the diagnostics or the
  exit codes changed, and no ledger needs migrating.

### Updating

```
cargo install gy --locked    # gy 0.8.0
```

The bundled skills (`gy-ledger`, `gy-question`) changed with this
release: they no longer tell you to write a node and then edit the body
in. A copy kept outside the crate still says the old thing.

### Added

- `need add`, `question add`, `criterion add` and `req add` take
  `--body-file`, the way `decide` already did: a node with a body is one
  write instead of two, and the log stops filling with edits whose only
  reason is that the body had to go in afterwards.
- `list --type` and `--status` match a kind or a status whatever its
  case, so the spellings on screen — `Need`, `Requirement` — work as
  typed, and a word that fits neither is answered with the ones that do.
- `list --since` takes a date (`YYYY-MM-DD`, UTC, that day's start
  onwards) as well as a write sequence. A sequence is a number inside
  the ledger; a date is what anyone asking for today's writes has.
- `serve`: a write in the rail or the history page is a link across its
  whole width, not only on the node's name.

### Internal

- The command line's types are `cli.rs`, so `main.rs` wires and
  `writes.rs` runs; a test walks the binary's own `--help` and fails if
  the cheatsheet or either README names an option the binary does not
  offer, or misses one it does. It found `decide --source`, undocumented
  since it was added.

## 0.7.0 - 2026-09-17

### Changed

- **Incompatible**: `gy-serve`'s public functions take the name shown in
  the browser tab: `server::serve`, `server::run`, `server::answer` and
  `api::route`. A caller passes the directory that holds `gy.toml`; the
  `gy` binary does. Nothing else in the API, the CLI, the store format,
  `gy.toml`, the diagnostics or the exit codes changed, and no ledger
  needs migrating.

### Added

- `serve`: a rail down the right side, from 1560px wide, showing the ten
  newest writes as they land — a write reaches it in about a quarter of
  a second — with the search at its top.
- `serve`: the three sidebar boxes open the nodes they count
  (`#/eye/wait`, `#/eye/next`, `#/eye/resume`), so the number pressed and
  the rows shown are one reading of one answer.
- `serve`: a zero that is good news is said out loud — nobody waiting,
  nothing half-done, a quiet ledger — while a ready column emptied by
  blocked needs stays plain. A ledger with no writes at all says so and
  how to start it, instead of `no ledger at …`.
- `serve`: a scope wears a badge, black with a coloured frame, the same
  colour on every page and never the same as its neighbour's.
- `serve`: an id can be copied with one press, on the node page and the
  graph's card.
- `serve`: the tab is titled `gy - <the directory that holds gy.toml>`,
  so two ledgers side by side can be told apart.
- `serve`: a node's body is open from the start, with no fold.

### Fixed

- `serve`: choosing a scope no longer takes the search box with it, and
  the page no longer throws on the next redraw.
- `serve`: the first click after the graph page opens is no longer read
  as the second half of a double click.
- `serve`: an empty history page showed the dictionary key
  `emptyWrites`; the list headings mixed English into Japanese; a
  satisfied criterion said `met` twice; the connection map drew its edge
  labels on top of its boxes; the sidebar counts ignored the chosen
  scope on every page but the first.
- `serve`: plainer Japanese for three labels — `通し番号`, `いまの空`,
  `最近の動き`.

### Internal

- `serve`: the browser assets are one state, one place that draws, one
  that listens, one that reads and one that waits; every region draws
  its own element and nothing else. Six tests hold that shape — who may
  reach for whom, who may touch the document, who may open a socket, who
  may build a state — so the rules are checked rather than remembered.
- e2e asks by test id, role and text, with a test that fails on a bare
  class selector; the suite no longer reaches into the page's internals
  for the camera or the rewind point.

## 0.6.2 - 2026-09-16

### Added

- `serve`: the now page is three eyes of equal weight — what waits on
  the master, the needs whose prerequisites are settled (`gy next`) with
  their remaining criteria, and the requirements in progress with the
  handover counts — then a small constellation of the nodes that matter
  now; the sidebar keeps the three counts on every page. `GET /api/now`
  carries `ready` and `resume` from the `next` and `handover` views.
- `serve`: the history page (`#/history`, `GET /api/history`) lists the
  writes newest first, grouped by day, filtered by author and date.
- `serve`: the wheel zooms about three times faster, `+` and `-` zoom,
  and a double-click on a bubble or empty space zooms in there.
- `serve`: a logo — the wordmark with the band's marks and the live dot —
  in the sidebar, the favicon, and both READMEs; one gothic face
  everywhere; the wordmark links back to the now page.
- `serve` opens the browser when started from a terminal.
- `scripts/demo-ledger.sh` builds a small English demo ledger; the
  English README's screenshots come from it.

### Fixed

- `serve`: a key that arrives while an IME is composing is not a page
  command, so the Enter that confirms Japanese no longer opens the
  search hit.
- `serve`: a node without an alias opens; query values and the node
  path are percent-decoded, so Japanese titles and `#` aliases can be
  searched.

### Removed

- `gy-migrate` and `gy-core` leave the workspace: every 0.4 ledger has
  moved. They remain in the history up to `v0.6.1`.

## 0.6.1 - 2026-09-16

### Added

- `serve`: a read-only web view of the ledger on 127.0.0.1. It answers
  the six questions first, then shows now, lists, one node, search on
  `/`, a time band that replays the log, live updates, and a fractal
  graph. Every read takes `at=<seq>`; the HTTP layer is the standard
  library alone; the end-to-end tests live under `e2e/`. It opens the
  browser when started from a terminal. Twenty-two terminal commands.
- `scripts/demo-ledger.sh` builds a small English demo ledger to try
  `serve` on.
- A logo: the gothic `gy` with the history band's ticks and a live dot,
  as the sidebar mark, the favicon, and the README head.

### Fixed

- An open repository's history no longer says `put`: a write is
  `created` or `updated`, and a live history reads the same as a
  reopened one.
- `undo` says whether the next undo would be a redo, and the cheatsheet
  explains that a second undo undoes the first.

## 0.6.0 - 2026-09-15

### Changed (incompatible)

- The ledger format is 2. Opening a format-1 ledger keeps `format.1.bak`
  and `events.jsonl.1.bak` and upgrades it in place.
- `publish` writes a directory per scope: one file per node (relations
  and marks with the other node's title, the applicability conditions,
  the body verbatim, records, free attributes) and `README.md` as the
  index (how to read, lists with links, history, diagnostics). It
  rewrites only the target scope directories under `--out` or the
  configured `output`. The single-file form is gone.

### Added

- `scope rename <old> <new>`: one transaction, one history line
  ("scope renamed old -> new (n nodes)"), and gy.toml rewritten; `undo`
  reverts it. Twenty-one terminal commands.
- `edit --set scope=<name>` moves a node to a scope declared in gy.toml.
- `edit --set decision_scope=<text>` records the applicability
  conditions of a decision the migration left unrecorded, once.
- `edit --set <key>=` removes a free attribute.

### Fixed

- `show`, `list`, and `publish` derive a need's state (open / closed /
  done) the way `next` does; `list --status done` matches needs.
- `gy-migrate` carries every remaining attribute as a free attribute
  and names them in its report instead of dropping unknown ones.
- Resolving an id tries the exact id before aliases, so a hash that is
  all digits no longer collides with an old id; new hashes contain a
  hex letter.
- Writing to a closed pipe ends quietly.

## 0.5.1 - 2026-09-15

### Changed

- A need may wait on a requirement as well as a question (`waits-on`);
  the wait resolves once the requirement is done or cancelled.
- `gy-migrate` turns a 0.4 need's `waiting-on` entries that point at a
  requirement, and a question's `belongs-to`, into `waits-on` edges, so
  `next` excludes exactly what 0.4 excluded.

## 0.5.0 - 2026-09-15

0.5 is incompatible with 0.4 in the storage format, the operation set, and the
configuration. A 0.4 ledger moves in one run of `gy-migrate`; see
[docs/migration-0.5.md](docs/migration-0.5.md).

### Removed

- The per-node Markdown ledger with YAML frontmatter and its dedicated git
  repository and worktree. The canonical ledger is now an append-only JSONL
  event log under `$XDG_DATA_HOME/gy/<hash of the repository root>/`; the
  repository keeps only `gy.toml`.
- `render` and the HTML projection (already removed). Human-facing output is
  `publish` now.
- The configurable workflow: `[workflow.records]`, `[workflow.guards]`,
  `waived_by`, and per-rule severities. No key other than `[scopes.<name>]` and
  `output` is accepted in `gy.toml`.
- The lint pass and its L1–L14 rules. Invalid state is refused when it is
  written, and what needs attention is counted by `handover`.
- The `req advance`, `req compress`, `node set`, `node submit`, `q`, `find`,
  `stats`, `init`, `scope rename`, `gate`, `import`, `cheatsheet`,
  `completions`, `skills`, and `mcp` subcommands, together with `--quiet` and
  `--verbose`. The closed set is 20 leaf operations (15 writes, 5 reads).
- Gates as a node kind, `parent_issue`, Issue-number requirement IDs, the
  `pr_url` / `pr_base` / `pr_files` requirement fields, and `bearer_count` as a
  stored value.
- The eleven requirement states and the per-transition records, guards, and
  snapshots.

### Changed

- The operation set is closed at 20 leaves: `show`, `list`, `next`, `handover`,
  `publish`; `need add` / `need close`, `question add` / `question close`,
  `criterion add` / `criterion satisfy`, `req add` / `req approve` /
  `req revise` / `req done` / `req cancel`, `decide`, `link`, `edit`, `undo`.
- A requirement has four states: `filed` → `approved` → `done`, and
  `cancelled`. After approval gy keeps only the revision, completion, and
  cancellation records.
- A requirement has a gy-issued ID and one opaque `ref`. gy never reads the
  reference.
- IDs are a kind prefix and a short hash (`n-3f9a`), minted locally with no
  central counter; a collision mints a longer hash. Old IDs are kept as aliases
  and resolve through `show`.
- Edges are stored on the node they start from; the reverse is derived. The
  set of relations is closed, and `waits-on` (need → question) replaces the
  dependency on a gate.
- Every write is one transaction in the log, carrying the sequence, time,
  actor, reason, and source. `undo --reason` inverts the last transaction.
- Every write names its actor in `GY_ACTOR`; an unset value is an error.
- Every write prints what it changed, what the node still lacks, and the shape
  of the command that could follow.
- `show` prints what a node still lacks. `publish` writes a one-page reading;
  naming nodes adds their descriptions.
- The workspace is `gy-ledger`, `gy`, and `gy-migrate`.

### Migration

- Run `gy-migrate <0.4 ledger> --root <repository root> --ref-base <Issue URL
  prefix> --publication <directory>`. It writes the new ledger in one
  transaction, freezes post-approval records as a publication, maps the eleven
  states to four, and generates `[scopes.<name>]` in `gy.toml`.
- `gy-migrate` and its private `gy-core` reader ship in this version so the
  remaining 0.4 ledgers can move. They are not the new core and are removed
  once those migrations are complete.
- Old IDs resolve as aliases (`show D-164`, `show '#6027'`). `--ref-base`
  turns an old `#N` into the requirement's `ref`.
- Follow [docs/migration-0.5.md](docs/migration-0.5.md) for the full steps and
  for what cannot be carried.

## 0.4.2

### Added

- A `[workflow.guards.<name>]` entry may set `waived_by` to the name of a
  project-defined record whose schema applies to requirements. A requirement
  carrying that record validly is not asked for the guard's `records` or
  `checks`. The waiver is preserved in the transition snapshot as a `waived`
  map, and checks from waived guards are not stored. An absent record leaves
  the guard in force, and an invalid one is reported while the guard still
  applies. `gy handover --json` includes `waived_by` in `workflow.guards`.
  A project that does not configure `waived_by` is unaffected.

## 0.4.1

### Added

- `gy import` marks every decision node it creates with the node attribute
  `imported: true`, whether or not `decision_scope` was filled. An imported
  decision whose applicability conditions are empty is reported as L14
  (default `warn`) instead of L7, so a legacy ledger taken in with
  intentionally empty scopes no longer fails `gy lint` and `gy handover`.
  L14 is configured with `[lint] L14` like any other rule. Decisions imported
  before this release carry no mark; collect their IDs from L7 findings in
  `gy lint --json` and run `gy node set <ID> --set imported=true` to bring
  them under the rule. Setting `decision_scope` clears the finding.

## 0.4.0

### Breaking

- Mutation results now return node summaries (`id`, `type`, `scope`,
  `changed_attributes`, `body_changed`) instead of full nodes. Attribute names
  describe actual value changes, including removals; creation lists all names.
  Repeating an unchanged update returns an empty list and `false`.
  This applies to CLI and MCP, including human output. Existing `node`,
  `need`/`requirement`, and `source`/`target` containers now hold summaries;
  import's `imported` array holds summaries and scope rename returns `nodes`.
  Warnings and import review information remain available. Submit returns
  `submission.record` and `submission.revision` (null without a version field),
  not the submitted records, schemas, or evidence. Init retains `root` and `scope`.
- Migrate consumers of mutation responses to `gy show <ID> --json` for all
  attributes and body text, or `gy find` for search. Compression with evidence
  also returns a summary without `archive`; obtain the full archival text with
  `gy req compress <issue>` before supplying evidence. That read-only preview
  and other query results are unchanged.

- Add `CheckKind::MatchesDeclaredFiles` and mark `DependencyRole`, `RuleConfig`,
  `FieldType`, and `CheckKind` as `#[non_exhaustive]`. Downstream matches must
  include a wildcard arm. Known enum variants remain constructible, including
  `RuleConfig::Detail { enabled: Some(true), severity: None }`. This differs
  from the non-exhaustive structs introduced in 0.3, whose downstream struct
  literals remain forbidden. See the [compiled migration example](docs/migration-0.4.md#update-rust-clients).
- The shipped workflow example now declares design files as `literal` or
  `generated` objects and compares them with `matches-declared-files`.
  Existing profiles keep their current behavior. To adopt the new shape,
  replace the `design_proposal.fields.files` schema using the
  [minimal profile](crates/gy/examples/file-scope.toml), change the file guard
  to `matches-declared-files` with implementation files on the left and design
  declarations on the right, and remove comparison keys or `[]` projections.
  Convert fixed design paths to `{ "kind": "literal", "path": "..." }` and
  generated names to fixed directory/prefix/suffix plus token class and length.
  Keep implementation paths as concrete strings. Already recorded designs need
  a new revision and matching approval; do not rewrite old snapshots. See the
  [complete migration steps](docs/migration-0.4.md#update-workflow-profiles-and-records).

### Added

- Restore HTML reading state from the URL hash, including filters, the selected
  record, tab, focus path, and view controls. Browser Back/Forward and reload
  restore the corresponding view; missing IDs are reported explicitly.
- Keep a permanent toolbar and sortable record table available while reading
  graph neighborhoods and details. Overview counts link to their corresponding
  record sets, derived from the HTML's embedded ledger data.
- Render Markdown tables, nested lists, fenced code, emphasis, and safe links
  in record bodies while retaining the offline, self-contained HTML boundary.
- Provide fixture caching keyed by CLI binary and generator SHA-256 hashes,
  explicit forced regeneration, and `test:spec`, `test:all`, and `test:perf`
  E2E entry points.

- Optional `description` strings on workflow records and fields appear in
  handover JSON and saved schemas without affecting checks or record revisions.
  Existing data needs no migration. Upgrade every ledger reader before using
  descriptions: older binaries reject configurations and histories containing
  them. Removing descriptions from today's configuration does not remove them
  from saved history. Older binaries also reject the new file comparison.
- The opt-in file comparison permits one generated filename token with an exact
  ASCII digit or lowercase hexadecimal length. Each declaration must match one
  reported file and each file one declaration. Undeclared, missing, duplicate,
  and ambiguous entries fail. No glob interpretation or file lookup occurs;
  existing `equal`, `same-set`, and `subset` semantics remain unchanged.

### Changed

- Record links open details; neighborhood navigation remains an explicit
  action. Cluster links filter the permanent table. Selected type chips,
  Lineage on/off, and a single status strip make the active view visible.
- Split the private workflow implementation into internal modules without
  changing existing `gy_core` public paths or type shapes.
- Organize HTML source as TypeScript modules and templates under `ui/`.
  Commit and package the Bun-built `app.js` and `app.css` embedded by Rust;
  end users do not need Bun, Node.js, or a browser automation installation.
- Run local `npm test` as 42 parallel functional checks. Use
  `npm run test:all` for the complete 45-check release/completion gate, or
  `npm run test:perf` for the three unchanged serial performance checks.
  CI continues to run all checks with one worker and zero retries.

### Fixed

- Align pointer cursors and visible controls with native links or registered
  button actions, including actual mouse hit targets across every table column.
- Wrap graph titles below IDs for up to three lines and include their measured
  bounds when fitting ordinary individual views. Preserve readable Lineage
  generation layout and panning. Full titles remain available in the table
  and details.
- Place individual edge labels away from node labels, omitting them when no
  nearby placement fits. All relationships remain available in node details;
  no edge-selection mode is introduced.
- Wait for URL persistence before history traversal in browser tests, then
  verify the destination URL, displayed tab, and unchanged history length.

- The example workflow accepts successful gates without a population through
  `passed-without-population`, requiring a reason and execution evidence.
  `passed` still requires its population and integer denominator. Existing
  users must add `passed-without-population` with required string `reason`
  and `evidence` fields to
  `workflow.records.quality_gates.fields.results.items.fields.result.variants`
  in their ledger configuration, preserving `passed`, local guards and past
  records. Use it only when no population concept exists; see the
  [workflow migration guide](docs/workflows.md#quality-gates-with-and-without-a-population).

### Migration

- Consumers that previously read a full node from a mutation response must use
  the returned ID to query `gy show <ID> --json`; read attributes and body from
  that query's `node`. See the Breaking section above for affected containers.
  This applies equally to CLI and MCP clients. For workflow profiles and saved
  records, follow the 0.4 migration steps described above; upgrade all readers before using the new
  schema descriptions or file comparison.
- Regenerate existing projections with `gy render --format html` to obtain the
  new UI. It is safe to repeat: rendering replaces derived HTML and does not
  change ledger records. Previously exported HTML retains its embedded UI.
- Contributors must use `npm run test:all` for completion evidence; local
  `npm test` intentionally omits performance checks. After editing `ui/`, run
  its `bun run build` and include the updated bundle in the change. See
  [the UI guide](crates/gy-core/ui/README.md) and
  [E2E commands](e2e/README.md).

## 0.3.1

### Changed

- Give details the same width as the graph on wide screens. Separate applicability
  and body from labeled declarations, show finite metadata as badges, and retain
  complete additional attributes. Relationship marks keep exact body annotations
  or explicit missing-location notices. Add browser checks for width, actual
  Japanese/Latin line wrapping, metadata, and lossless attributes.

- Focused neighborhoods remain individual even above 60 candidates. The graph
  selects up to 60 by distance, degree, and ID, with an omitted-node list for
  reaching the remainder. Omission and off-screen counts have separate actions.
- Add development-only Playwright checks for Chromium,
  including synthetic high-degree and 1000-node fixtures and automatic known-
  defect injection on every CI run. Node.js and browser binaries are not needed
  to build, install, or use gy.

- Clicking an individual graph node now moves the focus and opens details in a
  separate pane. Back, the focus path, and All nodes return through stored
  node/radius history without clearing filters, search, or lineage. A hidden
  focus remains in the path with an explicit notice. Reset everything clears
  navigation and filters together. Isolated nodes explicitly show that this
  graph contains no connections for the focus.
- Graph resizing preserves manual zoom and the graph point at the viewport
  center, while automatic views refit. Focus/displayed-node changes refit once
  after updating the detail pane. Automatic individual zoom is capped at 2
  to avoid over-enlarging isolated nodes; manual zoom still reaches 4. When a
  readable view cannot fit all selected nodes, it centers the focus (or the
  selected bounds if the focus is absent/filtered out) and reports off-screen
  nodes, including in lineage views.


- Split the HTML projection sources by responsibility and assemble them at
  compile time, preserving the generated HTML byte for byte. HTML generation
  still requires only the gy binary.

- The HTML graph now switches between two views by the number of visible
  nodes instead of by zoom level. Without a focus, more than 60 nodes form clusters
  with aggregated edge counts (per cluster pair) and internal edge counts per
  cluster, without individual nodes or edges. With 60 or fewer it draws
  individual nodes laid out by a force-directed layout that reflects
  connectivity, so an edge's length matches its endpoints' proximity.
- Clicking a cluster opens a member list (sorted by degree, title, or id,
  with a "current" mark for decisions not superseded); picking a member starts
  an adaptive neighborhood. The hop count is chosen per start as the largest
  value up to 5 whose reachable set fits the individual-view limit, and the
  chosen value is shown on screen.
- Node labels are measured with `getComputedTextLength`, truncated to the
  available width, and skipped when they would overlap a neighbour; the full
  title remains readable in the detail panel. Edge labels are off by default
  and drawn only when the individual set is small enough (20) or an edge is
  explicitly selected.
- Changing what is displayed refits the view, and the fit keeps individual
  labels at a legible size rather than shrinking them to fit every node; the
  count banner reports what is drawn in either view.
- The decision genealogy keeps its generation-layered layout.

### Upgrade

`cargo install gy --version 0.3.1 --locked`. No ledger migration is required
and the on-disk format and public API are unchanged. Regenerate existing HTML
with `gy render --format html` to use the new navigation and detail layout.

## 0.3.0

### Added

- `gy render --format html` projects the whole ledger into one self-contained,
  offline HTML file. It embeds the ledger's data and core judgments (lint, next,
  handover facts, stats, decision dependencies), draws all six node types and
  twelve relationship labels with direction, filters/scans the ledger, shows
  per-node details, and renders the decision lineage by generation.
- New `RenderConfig::html_output` (`gy.toml [render]`), defaulting to
  `gy.html` at the ledger root. Writing `{scope}` in it splits the output per
  scope. `split_threshold` never applies to HTML.
- `gy init` appends the default HTML output to the ledger root's `.gitignore`.
- Lint findings do not change `render`'s exit code: a broken ledger still
  renders so its problems can be seen from the projection.

### Changed

- **Breaking:** the public config structs `Config`, `ImportConfig`,
  `ScopeConfig`, `RenderConfig`, `WorkflowConfig`, `RecordSchema`,
  `FieldSchema`, `StateGuard`, and `RecordCheck` are now `#[non_exhaustive]`.
  Downstream crates can no longer build them with struct expressions of any
  form (exhaustive or functional update), because `#[non_exhaustive]` forbids
  struct expressions outside the defining crate. The new
  `RenderConfig::html_output` field is one reason; future render settings can
  now be added without another breaking release.
- **Breaking:** `RenderConfig` grows the `html_output` field (see Added). Rust
  library users constructing it exhaustively must update their code.
- The `--format html` value is accepted by `render` and `gy_render`.

### Upgrade

Install or update the CLI with `cargo install gy --version 0.3.0 --locked`.
No ledger migration is required: existing `gy.toml` files continue to parse
and the on-disk data format is unchanged.

#### Rust library users (`gy-core`)

Six config structs derive `Default`: `Config`, `ImportConfig`, `ScopeConfig`,
`RenderConfig`, `WorkflowConfig`, and `StateGuard`. For these, construct with
`Default::default()` and assign the public fields you need:

```rust
let mut render = gy_core::RenderConfig::default();
render.html_output = "gy.html".into();
```

The other three — `RecordSchema`, `FieldSchema`, and `RecordCheck` — have no
meaningful defaults and do not implement `Default`. They are read-only
configuration types: construct them by deserializing a `gy.toml`/JSON value
via `serde`. `RecordSchema::default()` and `FieldSchema::default()` do not
exist; this is deliberate, not a gap to fill in downstream code.

#### Existing ledgers: ignoring the HTML output

`gy init` appends `html_output` (default `gy.html`) to the ledger root's
`.gitignore`. When `html_output` contains `{scope}` it appends the expanded
pattern. For an existing ledger, either run `gy init <existing-scope>` again
(append-only, idempotent; nodes, relationships, records, and history are
preserved), or add the line by hand.

Re-running `init` also rewrites `gy.toml` in normalized form. Attribute values
are preserved, but comments and formatting are rewritten: inline table entries
expand to `[table]` sections, and the `html_output`, `[import]`, and
`[workflow]` defaults may appear if absent. If you keep operating notes as
comments in `gy.toml`, add the `.gitignore` line by hand instead of re-running
`init`.

## 0.2.1

### Added

- `gy scope rename <old> <new>` relabels a scope across its directory, member
  `scope` attributes, and `[scopes]` configuration without changing node IDs,
  relationships, records, or history. MCP exposes the same operation as
  `gy_scope`. CLI and MCP share the core operation; an existing destination is
  rejected rather than merged.

### Upgrade

Install or update the CLI with `cargo install gy --version 0.2.1 --locked`.
No ledger migration is required: the on-disk format and existing journal
entries are unchanged. Rust library users can set `gy-core = "0.2.1"`.

## 0.2.0

### Added

- Configurable workflow records, field types, explicit exceptions, and state
  guards, with matching CLI and MCP behavior.
- Stateless record submission and snapshots of validated records, schemas, and
  comparisons. Version checks prevent reuse of changed designs and approvals.
- Contract/gate coverage and declared file-scope comparisons.
- Workflow examples and shared authority/lifecycle architecture documentation.
- Historical superseded dependencies in show and handover output.

### Changed

- **Breaking:** `gy-core::Config` has new import and workflow fields. Callers
  using exhaustive struct literals must initialize them or use `Default`.
- **Breaking:** L1 uses explicit `waiting-on` / `unresolved` attributes; L2 uses
  the optional nonnegative integer `bearer_count`. Body text no longer declares
  unresolved references or counts. Record intended assertions in frontmatter.
- L5 checks current dependencies of unfinished requirements. Completed work
  retains its historical edges; reopening restores current checks.
- Current workflow policy governs ongoing work and new actions. Completed
  snapshots are validated against their saved schemas and comparisons, so
  adopting a profile does not require reconstructing past approvals.
- Scheduling stops following work prerequisites through completed requirements.

### Fixed

- ADR import can map a configured body section to `decision_scope` and reports
  missing scope and relationship marks. Imported missing marks remain visible
  as migration gaps.
- Multiple `raised-by` references are allowed; L4 checks `belongs-to` ownership.
- Code identifiers such as `waiting_checkout?` no longer cause L1 findings.
- Completed records can be compressed after profile adoption without fabricating
  historical workflow records. Existing completion/archive checks still apply.

### Upgrade

Install or update the CLI with `cargo install gy --version 0.2.0 --locked`.
Rust library users should set `gy-core = "0.2"` and update `Config` construction.
See `docs/architecture.md` for semantics and `docs/workflows.md` for adoption.
