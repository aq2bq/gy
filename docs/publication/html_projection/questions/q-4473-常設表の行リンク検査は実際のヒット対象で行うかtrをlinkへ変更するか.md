# q-4473 (Q-32) 常設表の行リンク検査は実際のヒット対象で行うかtrをlinkへ変更するか

state: closed
decider: マスター
options: 案A（推奨）: trのネイティブなrow役割とcellを保つ。trにもpointerを指定し、各列中央のelementFromPointが行全体を覆うa[href]（link役割、pointer）を返し、実マウスで対象詳細を開くことをE2Eに追加する。表のアクセシビリティを維持し、AC-32は実際の操作対象で判定する。, 案B: trにrole=linkとpointerを付け、行リンクとして検査する。DOMのtableは残るが、Chromiumのアクセシビリティツリーではデータ行24とcell144が消えるため、支援技術からの表の読み取りを失う。
scope: html_projection
created: 2026-09-13
  closes d-6380 (D-48) 一覧の行は表の row 役割を保ち、行全体を覆う native link を実ヒット対象とする。押せる見た目の検査は要素の計算スタイルではなく実ヒット対象で行う


閉じ方: 決定（2026-09-13T23:05:10.342455+00:00）

