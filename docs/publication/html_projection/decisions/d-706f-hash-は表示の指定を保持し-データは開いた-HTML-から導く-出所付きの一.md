# d-706f (D-47) hash は表示の指定を保持し、データは開いた HTML から導く。出所付きの一覧は種別化した出所（overview=next など）で表し、ID 集合は保存しない

## 関係
- narrows d-626e (D-38) 場所は URL が持つ: 表示状態は hash に載り、ブラウザの戻るが効き、ID を貼れば開く（mark: 表示状態は hash に載り）

decision_scope: HTML projection の hash（D-38 の具体化）。Overview のカードや状態行から至る一覧は、出所の種別（next、lint、waiting-on、requirement state など）とその引数を hash に載せ、実際の集合は表示時にそのファイルの埋め込みデータ（handover / next / lint の投影）から導く。同じ hash を再生成後の HTML で開けば、その時点の集合になる。個々のノードの選択は従来どおり ID で載せる。
scope: html_projection
created: 2026-09-13
  narrows d-626e (D-38) 場所は URL が持つ: 表示状態は hash に載り、ブラウザの戻るが効き、ID を貼れば開く（mark: 表示状態は hash に載り）
  closed-by q-6d40 (Q-31) Overviewから選ぶnext集合をURLで意味として保存するかID集合として保存するか

## Context

## Decision

## Consequences


