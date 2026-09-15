# q-6d40 (Q-31) Overviewから選ぶnext集合をURLで意味として保存するかID集合として保存するか

state: closed
decider: マスター
options: 案A（推奨）: overview=next のような型付きの出所をhashに保存し、HTML内のD.nextから都度対象IDを導出する。再生成後も現在のnextを読む意味を保持する。対応するフィルター表示と解除を用意する。, 案B: クリック時の対象ID集合をhashに保存する。再生成後も当時選んだ集合を可能な範囲で復元する。URLは長くなり、現在のnextと一致しない場合がある。
scope: html_projection
created: 2026-09-13
  closes d-706f (D-47) hash は表示の指定を保持し、データは開いた HTML から導く。出所付きの一覧は種別化した出所（overview=next など）で表し、ID 集合は保存しない


閉じ方: 決定（2026-09-13T22:48:35.700634+00:00）

