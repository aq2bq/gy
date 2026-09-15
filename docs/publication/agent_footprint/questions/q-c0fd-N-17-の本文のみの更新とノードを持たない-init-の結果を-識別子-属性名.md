# q-c0fd (Q-29) N-17 の本文のみの更新とノードを持たない init の結果を、識別子・属性名だけの契約でどう表すか

- 種類: question
- scope: agent_footprint
- created: 2026-09-13
- 状態: closed
- 別名: Q-29

## 関係

- closes d-922b (D-36) 変更系の結果はノード要約 id / type / scope / changed_attributes / body_changed とし、init は解決した root と scope を返す

## 本文

## 根拠と適用範囲
N-17 / D-34 の変更結果の形。main.rs の node set は本文と attrs を別々に保存する。本文はフロントマター属性ではなく、属性名 body も利用者が作れるため、変更属性名へ body を混ぜると両者を区別できない。init はノードを生成せず、現在の結果は root と scope。D-34 / Q-27 と未解決 question を確認したが、この境界の具体形は未記録。

## 案と推奨
案Aを推奨。ノードの要約は id / type / scope / changed_attributes / body_changed とする。changed_attributes は保存前後で変化した属性名（作成時は全属性名）、body_changed は本文の差分を示す真偽値。同値の再設定は空配列・false。本文・属性値を再送せず、本文のみの成功も何が変わったかで確認できる。init は実際に解決した root と scope を返す。N-17 の列挙に body_changed と root という例外が増える。

案Bは列挙項目を厳守する。本文のみの変更成功は識別子と空の changed_attributes だけ、init は scope のみ。形は最小だが、本文の変更と同値再設定を結果から区別できず、init の配置先は別途確認する。

どちらでも、値や本文は show / find で取得し、CLI / MCP に同じ形を使う。req compress の evidence なしは write=false の参照操作であり、既存の全文プレビューを維持する想定。evidence ありの実変更は全文を返さない。

## 閉じ方

- 決定で閉じた（2026-09-13T12:12:52.455036+00:00）

## 自由属性

- 無し

