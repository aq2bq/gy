# ac-add0 (AC-36) 取り込み由来で成立範囲が空のままの決定だけが残る台帳で、gy lint と gy handover の終了コードが 0 になり、gy decide で作った決定の成立範囲の欠落は error のまま残る

- 種類: criterion
- scope: import_provenance
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-36

## 関係

- targeted-by n-4534 (N-31) 取り込み時に成立範囲を意図して空欄にした決定の L7 を、新規の欠落と区別して報告する

## 本文

## 充足

- satisfied（2026-09-14 写しの台帳で測定（target/debug/gy、D-56 の実装）。定型文「（移行時に明示されていない）」で成立範囲を空欄のまま 3 件、埋まった 1 件を gy import で取り込み: 4 件すべて imported=true、空欄 3 件が L14 warn、L7 は 0 件、gy lint と gy handover の終了コードは 0。その台帳で gy decide した D-5 の decision_scope を空にすると L7 error が 1 件出て lint / handover の終了コードは 1。D-1 から imported の印を外すと L7 error に戻り、gy node set D-1 --set imported=true で L14 warn に移る。gy 自身の台帳は新しい binary でも無指摘、終了コード 0。cargo fmt --check OK、clippy -D warnings OK、cargo test --workspace --locked 73 passed / 0 failed（configured_workflow 15、html_render 7、mutation_output 3、workflows 39、gy_core 9）。） 2026-09-14T02:52:57.544466+00:00

## 自由属性

- 無し

