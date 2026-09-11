---
name: gy-decide
description: Use when recording decisions with applicability conditions in gy and linking changed passages in existing decisions and question closures.
---

# gy Decisions

Record what was decided, where it applies, and which earlier records it changes.

## State the decision and its scope

```sh
gy decide "State the decision in one sentence" \
  --scope-note "Applicable paths, conditions, and exclusions" --scope <scope>
```

`--scope` selects the ledger scope. `--scope-note` defines where the decision applies. A person must review whether that description is specific; filling the field alone does not establish this.

## Close answered questions

Supply `--closes <question-ID>` for the question being answered. Read the open questions displayed during creation and close any others answered by the same decision.

## Link changes to earlier decisions

Use `gy link <new-decision-ID> <relationship> <old-decision-ID>`:

| Relationship | Effect on the older decision |
| --- | --- |
| `narrows` | Restricts applicability |
| `widens` | Expands applicability |
| `supersedes` | Replaces the decision |
| `completes` | Fills previously undecided parts |

For `narrows` and `supersedes`, supply `--mark` with the affected passage quoted from the older body. A file-level relationship alone cannot identify the invalidated sentence.

## Verify the affected passage

Display the older decision with `gy show <old-decision-ID>`. Confirm that the affected passage and its relationship to the new decision are clear.

gy stores the mark in frontmatter and adds annotations during show/render. If a body edit makes the passage impossible to locate, review the displayed diagnostic and update the anchor to identify the intended passage.
