# n-9198 gy-migrate が既知の名前以外の自由属性を落とし報告にも出さない欠陥を直す: すべての残った属性を自由属性（配列は改行区切り）として写し、報告に一覧を出す。edit --append の意味（文字列に 1 行足す）を文書に明記

state: closed
scope: gy05
created: 2026-09-15
  targets ac-d8f9 (AC-47) Kokopelli の 0.4 台帳が一回の移行で入る。全ノードはハッシュ ID に振り直され、旧 ID と ref で引ける。lint の error が 0
  targets ac-b0f3 (AC-46) すべての書き込みに、いつ・誰が（GY_ACTOR 必須）・なぜ・出典が残り、undo できる
閉じた理由: 事実（2026-09-15 lead が検収。measure は legacy/ の文字列キーのみ、テスト 0 failed。Kokopelli の写しで自由属性 27 名 255 値が写り、8 種 13 件を含む。next 6 件、辺 540、waits-on 17。コミット 6446450。ローカルにインストール済み）

