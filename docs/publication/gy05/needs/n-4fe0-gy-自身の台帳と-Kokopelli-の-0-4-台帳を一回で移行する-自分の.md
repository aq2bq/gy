# n-4fe0 (N-41) gy 自身の台帳と Kokopelli の 0.4 台帳を一回で移行する（自分の台帳で先に試し、旧 ID を別名に）

state: closed
scope: gy05
created: 2026-09-14
acceptance: 2026-09-15 15:09 Kokopelli の台帳（74fa00f、356 ノード）を本番で移行。凍結 fd37152、移行後 09a7f8d。next 6/6、進行中 5/5 で 0.4 と一致。正本 ~/.local/share/gy/a4774e3c
2026-09-15 15:12 進行管理担当の指摘（GitHub の正式名は YUKASHIKADO/Kokopelli）で、正本を消して --ref-base を正式名にして再移行。書き込みは移行の 1 件だけだったので損失なし。ID は振り直し。gy は ref を不透明に扱うので大文字小文字の照合は足さない（D-73）
2026-09-15 15:27 欠陥: 既知の名前以外の自由属性 8 種 13 件（additional-decisions、handoff_notes、partially-decided-by、completed_note、depends_on_sources、document_issue、measures、closure_clarification）が落ち、報告にも出なかった。値は publication/0.4 に残る。修正は n-9198、復旧は lead が edit --set で行う予定
  depends-on n-0551 (N-65) 移行 (a1): クレート gy-migrate の骨格（引数、ガード、0.4 を gy-core で読んで中間の型 Legacy へ、--dry-run、報告の器）
  depends-on n-00dc (N-66) 移行 (b1): 辺と waits-on（Relation に WaitsOn、next の準備判定を辺読みに、gy5 link の canonical 名に waits-on）
  depends-on n-3806 (N-67) 移行 (a2): ノードの写し（種類ごとの対応表、別名、ref、1 トランザクション）とテスト（N-65 から分割）
  depends-on n-e3c1 (N-68) 移行 (b2): legacy.rs の分割、要求の 11 状態 → 4 状態と approval / completion、決定側の closes を読まない修正
  depends-on n-3a5d (N-69) 移行 (b3): 記録の凍結（publication と legacy_records）、報告（stdout と migration-report.md、source の git コミット）、gy.toml の生成と検査（N-68 から分割）
  targets ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0
  targets ac-e8b7 (AC-54) すべての変更が物理設計の基準（1 ファイル 300 行、1 関数 40 行、glob import 0、属性の文字列キーは 1 か所、テストは操作ごとに 300 行以下、1 ニーズの差分 600 行以下）と層の依存の向きを満たし、検収前に機械で測った値が残っている
  targets ac-c2bd (AC-55) 正本が形式の版を持ち、古い版を開くと写しを残して 1 トランザクションで移行され、非互換の変更ごとに移行か対応表が同梱され、Kokopelli の写しで試した値が残っている
  depended-on-by n-147b (N-70) 0.5.0 の準備: gy5 を gy に改名し、gy-core と 0.4 の CLI と gy-migrate を消し、docs/ledger を新しい正本へ切り替え、版とCHANGELOG を整える（公開はマスターの指示）
## 範囲

移行の対象は 2 つ。先に gy 自身の台帳（docs/ledger、0.4 形式、スコープ 8 つ）を新しい gy へ移してドッグフーディングし、次に Kokopelli の台帳の写しで試す。旧 ID は別名として保持し、show / list で引ける。要求は ref に Issue の URL。成立範囲が空の決定は scope_unrecorded。G-1 は残作業付きのニーズにし Q-5 待ちを waits-on へ（D-83）。スナップショットと記録は publication として凍結し参照だけ残す。

## 09-15 分割（lead）

設計は .local/brief-n41-draft.md。N-65（骨格とノード）→ N-66（辺・状態・凍結・報告）→ このニーズ（gy 自身の台帳と Kokopelli の写し .local/kokopelli-ledger-copy（347 ノード、8fefd31）での試行と直し、AC-47 / AC-55 の測定）。

閉じた理由: 事実（migrated: complete）

