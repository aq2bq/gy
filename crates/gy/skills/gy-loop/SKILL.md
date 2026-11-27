---
name: gy-loop
description: "Use first whenever you start work, resume work, or decide what to do next in a project that has a gy.toml. How to work with gy at the centre."
---

# gy-loop

gy is a tool that records what happens until it is settled what to build. The whole of that record is called the ledger. This skill gives the steps for resuming work from the ledger and advancing one problem at a time.

You are the one who reads and writes the ledger. For how to talk with the person, follow the gy-communicate skill.

## What goes in the ledger

- Need: what the person wants.
- Acceptance criterion: a measurable condition that tells the need is met.
- Requirement: a proposal of what to change to meet the need. Work outside the ledger, such as building, starts after it is approved.
- Decision: a judgement about the thing being built.
- Question: something the person decides.

## Decide once, at the start

If the ledger does not hold these two decisions, ask the person before any work and record each answer as a decision.

- Approval policy: who approves a requirement. The person reads every one, reads only the first, or leaves it to you.
- Unclear needs: you ask the person before going on, or you go on with your reading and they read the decisions afterwards.

## Starting

- If there is no ledger, create one with `gy init <scope>`. The scope is the name of a section of the ledger.
- Run `gy handover` and `gy next` to read the requirements in progress and the needs you can start next.
- Read the records you need with `gy show <ID>`. An ID is the identifier of one record.
- If the writer name `GY_ACTOR` is already set, do not change it. If it is not set, set a name that identifies you.

## Advancing

| Situation | Action |
| --- | --- |
| There is a new need | Record its acceptance criteria and the need |
| You made a judgement about the thing being built | Record a decision with the gy-decide skill |
| A need is unclear, or a request contradicts an existing decision | Record one question with the gy-question skill, show it to the person, and stop |
| A requirement is not approved | Get approval according to the approval policy |
| A requirement is approved | Do the work the requirement describes |
| You verified an acceptance criterion | Record the measured result |

After each write, gy answers with what is still lacking (`missing`) and the commands that could follow (`next`). Follow them.

## Finishing

Verify the real result and record the evidence against the acceptance criteria. Do not take done in the ledger for done in reality. Say what you did not verify.

## References

- Talking with the person: `../gy-communicate/SKILL.md`
- Decisions: `../gy-decide/SKILL.md`
- Questions: `../gy-question/SKILL.md`
- How records connect: `../gy-ledger/SKILL.md`
- Command syntax: `gy cheat`
