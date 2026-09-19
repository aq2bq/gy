---
name: gy-decide
description: Use when you record a decision in gy: how to write where it holds, a mark, and what it closes.
---

# Recording a decision

## Where it holds

`decide` refuses a decision without `--scope-note`. Say in one sentence where the decision holds and where it does not, so a later reader can tell. It cannot be empty.

## Closing a question

Pass `--closes <Q>` when the decision closes a question. The way it closed and the edge stay on the question. A closed question cannot be closed again.

## Lineage and the mark

To narrow or replace an older decision, use `--relate narrows|supersedes <old D> --mark <text>`. The mark is not a note: it is the exact text, inside the older decision's body or scope note, that stops applying. gy refuses a mark it cannot find there. The old body is never changed; the mark is shown beside it as the trace of the relation. Other relations (`widens`, `completes`) are made with `link`, without a mark.

## When unsure

The output of `decide` says what is still missing and what you could type next.
