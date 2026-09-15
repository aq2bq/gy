# ac-1b8a (AC-15) 母数の概念が無いゲートの合格を、数値を作らずに記録でき、母数のあるゲートでは母数の欠落が検出される

- 種類: criterion
- scope: workflow_records
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-15

## 関係

- targeted-by n-54b9 (N-12) 母数の概念が無いゲートを、数値を作らずに記録できるようにする

## 本文

## 充足

- satisfied（N-12: configured_workflow suite 13 passed, 0 failed. New test submits counted passed and passed-without-population together, rejects 4 missing-field cases (denominator, population, reason, evidence) with exit 2 and unchanged node bytes; full example normal/waived transitions and archive pass. cargo fmt --all -- --check and cargo clippy --workspace --all-targets --locked -- -D warnings exit 0; cargo test --workspace --locked: 63 passed, 0 failed. No core changes for N-12.） 2026-09-13T08:54:35.512588+00:00

## 自由属性

- 無し

