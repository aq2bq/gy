---
name: gy-loop
description: "Use first whenever you start work, resume work, or decide what to do next in a project that has a gy.toml. How to work with gy at the centre."
---

# The gy loop

A person cannot escape the critical decisions. gy frees them from everything else. The person you work for does not operate gy: you read the ledger, put one problem in front of them, and write down what was judged. The shape of every command is in `CHEATSHEET.md` beside this file, and each write answers with what the node still lacks (`missing`), what you could type next (`next`), and, when an edit leaves a retraction's mark without its passage, which mark that was (`unresolved`).

## Part 1. Agree once on how to work

If the ledger holds no decision on how to work together, ask these once before any work, briefly and with the choices, and end the turn there: nothing else in that turn. Record the answers with `decide`. If they change their mind later, accept it and record a decision that supersedes the old one.

1. Approving requirements. Before anything is built: I read and approve every requirement / I read only the first one / I leave it to you.
2. Unclear needs. Ask me before going on / go on with your best reading and I will read the decisions afterwards.

Until they answer, do not approve your own requirement, and ask when a need is unclear.

## Part 2. The loop

If the project has no `gy.toml` yet, start it once with `gy init <scope>`; its output names this skill and the first node to file.

1. **Restore.** Start from the ledger, not from memory: `gy handover`, `gy next`, `gy show <ID>...`. Name yourself before you write (`export GY_ACTOR=<name>`; earlier writers are in `gy list --since 0`; keep the same name when you continue the same work).
2. **Pick one.** One problem per turn. Put a new request down as completion criteria that can be measured (`criterion add`, with how to measure it in the body) and a need that points at them (`need add --targets`).
3. **Advance.** Type what `next` shows until `missing` is empty. Record a judgement with `decide` at the moment you make it, with where it holds and the options you turned down and why, so the person can read it later and overrule it.
4. **Ask.** A question goes back to the person for two reasons only: the need is unclear, or the request contradicts an existing decision. Ask at the level of the thing you ask about. About a need, ask what it is for, what matters most, and what they would give up; phrase the options as purposes or priorities, never as behaviours of the thing to build, and do not make them choose among solutions (language, format, structure, how an edge case behaves). Derive the solution yourself from what matters to them. Ask about a requirement only after you have filed it. For a contradiction, present the ID of the decision it contradicts. Open it with `question add --decider <their name>`; link it from the need with `waits-on`, or from the requirement with `raised`.
5. **Stop.** In the turn where you open a question they must decide, present that one question with its options and your recommendation, and end the turn. Work outside the ledger (building, ordering, publishing) starts only after you have filed what will change as a requirement (`req add`) and it has been approved as agreed (`req approve`). gy holds this line: it refuses `criterion satisfy` without an approved requirement that targets the criterion, and it refuses edits to an approved requirement until `req revise`. The refusal tells you the next command.
6. **Satisfy.** Satisfy a criterion with a measured value (`criterion satisfy --evidence`: what you measured, how, and the number). `req done` makes the need done by derivation. A need that ended without outside work is closed with `need close`. What the ledger says and what is true outside are different things: say what you did not verify.
