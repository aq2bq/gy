---
name: gy-question
description: Use when checking existing records before creating unresolved questions in gy, or closing questions by fact, decision, or non-decision.
---

# gy Questions

Record choices that need a decision, then close them with a traceable reason.

## Check existing answers

`gy question add` searches all scopes before creating a question. Read matching sections and applicability conditions to determine whether the question is already answered. Use `--force` after verifying why a separate question is needed.

## Create a question

Read `gy question add --help` for current syntax. Supply:

- `--options`: at least two distinct, viable choices.
- `--decider`: whose agreement is required.
- For a bundle, `--bundle` and `--bundle-rationale`: why the same intervention closes its questions.

Investigate matters uniquely determined by facts; inventing a second option does not create a decision to make. A shared cause alone does not justify a bundle. `gy q` is reserved for quick human notes; agents use `gy question add`.

## Choose a closure method

| Result | Arguments to `gy question close <ID>` |
| --- | --- |
| Facts resolved the question without a choice | `--by fact --note "<reason>"` |
| A choice was made | `--by decision --decision <decision-ID>` |
| The decision was not to decide | `--by non-decision --decision <decision-ID>` |

For decision and non-decision closures, first record the decision with `gy decide`.

## Update waiting references

Closure lists nodes that still reference the question as unresolved. Read those nodes, verify whether their pending matters are resolved, and update their records. Run `gy lint` to check the resulting references.
