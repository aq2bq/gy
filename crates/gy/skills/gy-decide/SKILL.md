---
name: gy-decide
description: "Use when you record a decision in gy: what it is for, where it holds, a mark, and what it closes."
---

# Recording a decision

## What it is for

A decision must say what it is for, or `decide` refuses it. Name it with `--serves <N|R|AC>`: the need, requirement, or criterion the judgement belongs to. `--closes <Q>` also ties a decision in, and so does `--relate` to a decision that already has one. A decision that closes a question and one that narrows a bound decision need no `--serves`.

What is recorded is a judgement about the thing being built. How the team runs is not one: how a review was judged, which tool or branch to use, and what happened in a session stay out of the ledger (d-749e).

A decision written before this rule may serve nothing. It is still readable: `gy show <D>` reports `what it serves (serves)` and `handover` counts it. Tie it with `gy link <D> serves <N|R|AC>`.

## Where it holds

`decide` refuses a decision without `--scope-note`. Say in one sentence where the decision holds and where it does not, so a later reader can tell. It cannot be empty.

## Closing a question

Pass `--closes <Q>` when the decision closes a question. The way it closed and the edge stay on the question. A closed question cannot be closed again.

## Lineage and the mark

To narrow or replace an older decision, use `--relate narrows|supersedes <old D> --mark <text>`. The mark is not a note: it is the exact text, inside the older decision's body or scope note, that stops applying. gy refuses a mark it cannot find there. The old body is never changed; the mark is shown beside it as the trace of the relation. Other relations (`widens`, `completes`) are made with `link`, without a mark.

## When unsure

The output of `decide` says what is still missing and what you could type next.
