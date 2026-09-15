# ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる

satisfied: true (2026-09-15 N-38 / N-49 / N-55。events.jsonl の各行に seq・at・actor（GY_ACTOR 必須）・why・source。why は <操作名> <ID>、source は操作の evidence / URL（全操作、grep で確認）。undo は逆変更の行を追記（tests/undo.rs、undo_op.rs）) at 2026-09-15T03:51:51.817234+00:00
scope: gy05
created: 2026-09-14
  targeted-by n-0dae (N-38) 正本を git の外に置き、ハッシュ ID、GY_ACTOR 必須の履歴、undo、ロックを持つ
  targeted-by n-9198 gy-migrate が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直す: すべての残った属性を自由属性（配列は改行区切り）として写し、報告に一覧を出す。edit --append の意味（文字列に 1 行足す）を文書に明記
  targeted-by n-bd8d (N-49) undo（逆変更の行の追記）と snapshot.json（commit ごとの原子的な書き直し、無くてもログの再生で復元）



