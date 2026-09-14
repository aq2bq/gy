gy (good,yes) checks internal ledger consistency. It does not verify that the ledger matches reality.

At session start:
  gy handover --json
  gy find --where type=decision
  gy next

Setup (specify --scope when outside the target scope directory):
  gy init demo --parent-issue 6000
  gy criterion add "Prevent duplicate deliveries" --scope demo
  gy need add "Make retries safe" --targets AC-1 --scope demo
  gy req add "Retry control" --issue 6006 --parent-issue 6000 --scope demo
  gy need file N-1 --issue 6006
  gy scope rename old-name new-name   # relabel a scope; IDs, edges, records, and history persist

Questions and decisions:
  gy question add "How should deliveries be ordered?" --decider master --options "Publication time" --options "Arrival time" --scope demo
  Read existing matches. Use --force if this is a different question.
  Bundles require --bundle and --bundle-rationale (why the same intervention closes the questions).
  gy decide "Order deliveries by publication time" --closes Q-1 --scope-note "Production delivery workers; excludes batch replays" --scope demo
  gy question close Q-2 --by fact --note "Why measurements uniquely determined the answer"
  gy question close Q-3 --by decision --decision D-1
  gy question close Q-4 --by non-decision --decision D-2
  For decision and non-decision closures, first record a decision with gy decide.
  gy q is a quick note command for humans only. Agents must not use it. Incomplete records fail lint.

Relationships (saved on both sides):
  question closes decision
  decision narrows|widens|supersedes|completes decision
  need|requirement targets criterion; need spawned-by decision / filed-as requirement / depends-on need
  requirement relies-on decision / raised question
  gate measured-by question
  gy link D-2 narrows D-1 --mark "Affected passage quoted from the older body"
  mark does not change the older body. show/render annotates the matching passage.

States (11 values; parenthesized context is allowed):
  unfiled / defining / awaiting-design / awaiting-approval / awaiting-implementation / awaiting-audit /
  awaiting-pr / awaiting-merge / awaiting-production / awaiting-cleanup / complete
  gy node set '#6006' --set pr_url=https://github.com/org/repo/pull/6007 --set pr_base=main --set pr_files=3
  gy req advance 6006 --to awaiting-merge --evidence "Reviewed the PR diff" --reported-base main --reported-files 3
  gy node set '#6006' --set remaining_work=0 --set deviations=none --set residual=none --set next_evidence="Production verification results" --set responsible=master
  gy req advance 6006 --to complete --evidence "Verified no remaining work" --data-migration false --production-only false --cleanup-done true
  When production work is required, record --production-done true after completing it.

Record attributes:
  waiting-on / unresolved: arrays of IDs referenced as unresolved, e.g. --set 'waiting-on=["Q-1"]'.
  belongs-to: an array containing a single owning node ID for a question. raised-by records originating requirements and may contain multiple IDs.
  bearer_count: an optional nonnegative integer, checked against actual targets links.
  L1/L2 use explicit attributes; body wording does not declare unresolved references or bearer counts.
  relies-on: the decision basis of requirement work, retained after completion. L5 checks current work only.
  Superseded dependencies of completed requirements appear as history in show and handover.historical_superseded_dependencies.
  Reopening restores current checks. Structural and completion integrity checks still apply to historical records.
  residual: a list of destination IDs N-xx / Q-xx / #Issue for transferred work, or 'none'. Blank or missing values trigger L11.
  deviations: deviations from the approved design and additional decisions, or 'none'.
  remaining_work: a legacy remaining-work count or array. If present, completion requires zero or an empty array.
  next_evidence / responsible: next-transition evidence and the responsible party for an active requirement.
  gy criterion satisfy AC-1 --evidence "Acceptance verification record"
  Set arbitrary attributes with gy node set <ID> --set key=value. Values are JSON or strings.
  Update the body with --body-file <path>. IDs are immutable; use gy link for edges.

Search, output, and checks:
  gy find delivery --where type=decision --where 'created>=2026-09-01'
  gy show D-1 / gy show D-1 --graph
  gy lint --json / gy render / gy stats --days 7
  Reads cover all scopes by default. Filter with --scope <name>.
  Configure L1–L14 and edges in [lint] as true / false / "error" / "warn" / "off".
  Exit codes: 0 success, 1 failed check, 2 invalid input or guard violation, 3 ledger corruption.
  gy import docs/adr --scope demo preserves IDs. Set [import] scope_note_section to map a body heading into decision_scope.
  scope_note_placeholders lists exact placeholder-only texts to leave unfilled. Review import_summary for missing scope and marks.
  Imported narrows/supersedes entries carry imported=true; L6 distinguishes their missing marks without changing severity.
  Imported decision nodes carry imported=true; an empty decision_scope on them is L14 (warn) rather than L7 (error). Mark pre-0.4 imports with gy node set <ID> --set imported=true.
  gy skills install .agents/skills / gy mcp serve / gy completions zsh

Compressing completed requirements:
  List downstream constraints in constraints=[{"text":"Constraint","decision":"D-1"}].
  Record each decision's applicability conditions and add requirement relies-on decision links.
  Review each constraint and record constraints_reviewed=true. If there are no constraints, use constraints=[].
  Record six items with node set:
    summary: the outcome in one sentence on one line.
    contracts_changed: free-form text or a list of changed contracts.
    artifacts: {"pr":"https://github.com/org/repo/pull/6007","merge_commit":"a1b2c3d","base_branch":"main"}
    production: an object with measurements, target environment, and verification results; 'none' if no production work was needed.
    deviations: deviations and additional decisions, or 'none'.
    residual: destination IDs (e.g. ["N-2","Q-3","#6010"]), or 'none'.
  gy req compress 6006 > archive.md
  The caller archives the full text in the corresponding Issue comment and verifies its contents.
  gy req compress 6006 --evidence 'https://github.com/org/repo/issues/6006#issuecomment-123'
  gy replaces the body with six items, returns the original full text to stdout, and records the archive URL in compressed_from.
  IDs, graph relationships, and unknown attributes are retained. Full constraints, quality gates, design proposals, and audit records move to the archive.
  gy find --where 'contracts_changed~orders.order_lines'

Configured workflow records (optional):
  Read the workflow configuration in gy handover --json to discover record fields and state guards.
  [workflow.records.<name>] declares kinds, required fields, types, alternatives, and an optional version_field.
  [workflow.guards.<name>] declares states, required records, and equality/set comparisons.
  Author drafts with gy node set <ID> --set '<name>={...}'.
  gy node submit <ID> --record <name> --evidence "Review record" validates and snapshots a report without changing state.
  gy req advance checks every matching destination guard and snapshots the checked records in transitions[].workflow.
  A recorded version cannot be reused with different contents. A revised design needs a matching new approval.
  lint/handover reports omissions under workflow. Disabling that lint rule does not disable guards.
  Gate outcomes, approval identity, and file lists are caller reports; gy does not verify external facts.
  Completed requirements use saved workflow schemas/checks, not today's policy, even before compression.
  New transitions and submissions always use current policy; adopting a profile does not fabricate old approvals.
  Compression validates and archives existing history while retaining core completion and archive checks.
