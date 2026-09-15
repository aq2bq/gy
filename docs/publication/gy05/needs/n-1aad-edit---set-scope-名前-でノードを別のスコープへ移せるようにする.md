# n-1aad edit --set scope=<名前> でノードを別のスコープへ移せるようにする（名前は gy.toml にあるものだけ。ID と辺は変わらない）

state: open
scope: gy05
created: 2026-09-15
  targets ac-7652 (AC-53) 後から検査する lint が無く、不正な状態は書き込み時に拒まれ、注意が要る状態は handover が作業として出す
  spawned-by d-7c64 publish の出力は 1 ファイルでなく、スコープごとのディレクトリに 1 ノード 1 ファイルと索引 1 ファイルを書き出す形にし、出力先のスコープのディレクトリの中だけを作り直す

