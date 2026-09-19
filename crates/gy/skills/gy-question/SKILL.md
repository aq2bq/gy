---
name: gy-question
description: "Use when you open or close a question in gy: choosing the decider, the options, and the way it closes."
---

# Questions

## Opening

`question add` needs a decider (`--decider`) and at least two distinct options (repeat the flag: `--options A --options B`). If nobody's agreement would close it, it is not a question. A question a need waits on is about the need; a question a requirement raised is about the requirement (`gy-loop`, "Ask"). The body is the ground for the options, so write it with `--body-file` at creation.

A question carries what it is for as an edge. If a need waits for it, `link <need> waits-on <q>` (`raised` if a requirement produced it). Until it is linked, `missing` says so and `handover` counts it among the open questions nobody waits on. A need is not required: question → decision → a need `spawned-by` it also works.

## Three ways to close

`question close --by` takes one of:

- `fact`: a fact settled it. Put the ground in `--evidence`.
- `decision`: a decision settled it. Point at it with `--decision <D>`; make the decision first with `decide`.
- `non-decision`: closed without pointing at a decision.

Once closed, the way it closed cannot change.

## When unsure

`question add` prints `id: <ID>` first and shows what could follow (`question close`, `decide`, `link … waits-on`).
