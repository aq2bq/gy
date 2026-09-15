# ac-c21e (AC-43) セッション再開の 1 コマンド（handover）で、進行中と確定・未完了の要求が ref 付きで復元でき、warn は件数だけ

satisfied: true (2026-09-15 N-60 / N-41。gy5 handover の 1 コマンドで、進行中の要求（Filed / Approved）を ID・ref・題名・next_evidence・responsible で出し、warnings は件数だけ。Kokopelli の写しで進行中 4 件を ref 付きで復元（.local/report-n41.md）) at 2026-09-15T03:51:52.221215+00:00
scope: gy05
created: 2026-09-14
  targeted-by n-0471 (N-36) bearer_count と need の状態を導出にし、handover を error と進行中だけにする
  targeted-by n-0476 (N-60) 読み (2b): handover（error と進行中の要求を ref 付きで、warn は件数だけ）を views 層に作る（N-59 から分割）
  targeted-by n-35cf show / list / publish のニーズの状態を next と同じ導出（open / closed / done）にする。list --status done を受ける
  targeted-by n-58b4 (N-58) 読み (1b): list（--type / --status / --targets / --grep / --actor / --since。actor と since は書き込み単位）を views 層に作る（N-57 から分割）
  targeted-by n-66ee (N-34) 要求の状態を 4 つにし、確定後は改訂・完了・中止の記録だけを持ち、ID を振り直して ref を持たせる
  targeted-by n-7cc1 (N-57) 読み (1a): views の骨格と show（複数 ID、ref でも引ける、種類ごとの逐語、--full、--json）
  targeted-by n-8996 (N-59) 読み (2a): HistoryEntry の seq、need の状態の導出、next を views 層に作る



