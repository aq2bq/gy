# ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0

- 種類: criterion
- scope: gy05
- created: 2026-09-14
- 状態: satisfied
- 別名: AC-47

## 関係

- targeted-by n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）
- targeted-by n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）
- targeted-by n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）
- targeted-by n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）
- targeted-by n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）
- targeted-by n-9198 gy-migrate が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直す: すべての残った属性を自由属性（配列は改行区切り）として写し、報告に一覧を出す。edit --append の意味（文字列に 1 行足す）を文書に明記
- targeted-by n-b6a7 waits-on の先に要求を許す: model の組、derive の準備判定、gy-migrate の waiting-on の写し、テスト
- targeted-by n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正

## 本文

## 測り方

Kokopelli の台帳の写しを gy-migrate 1 コマンドで移行し、全ノードがハッシュ ID、旧 ID・#番号・URL で resolve でき、handover の error が 0 で warnings が 0.4 の lint の warn 以下であることを見る。

## 充足

- satisfied（2026-09-15 N-41（.local/report-n41.md）。Kokopelli の写し（342 ノード、コミット 8fefd31）を gy-migrate 1 コマンドで移行: 全ノードがハッシュ ID、旧 ID（N-18 等）・#6027・URL で解決、0.4 の lint error 0 / warn 81 に対し gy5 handover の warnings 13。gy 自身の台帳（253 ノード）でも next 17/17 一致。再移行の差分は publication のパスのみ） 2026-09-15T03:51:50.882447+00:00

## 自由属性

- 無し

