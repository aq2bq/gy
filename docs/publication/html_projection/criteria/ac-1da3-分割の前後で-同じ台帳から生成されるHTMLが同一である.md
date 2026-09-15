# ac-1da3 (AC-6) 分割の前後で、同じ台帳から生成されるHTMLが同一である

- 種類: criterion
- scope: html_projection
- created: 2026-09-13
- 状態: satisfied
- 別名: AC-6

## 関係

- 無し

## 本文

## 充足

- satisfied（2026-09-13: baseline 21c899560cc481ccd086a8cbdf3595f10d4bde7b の template と html.rs の concat! 順で再結合した19断片をバイト比較し一致。両方61026 bytes、SHA256 28b12d2df2ed37f34f13a30aa230a7b64714538a4d951487abad83f24e4b8b5b。gy自身の台帳コピー47ノードを変更後CLIで生成し、その同一escaped payloadを旧templateへ埋め込んだHTMLと比較して一致。両方99576 bytes、SHA256 598b4ba854a97ea40f1b2476ee54e5377e2a4f2c89224c2599f2dda8e4ccdd99。再現スクリプトと測定記録は .local/html-split/verify.py と identity.json。314ノード実台帳のブラウザ操作は未確認。） 2026-09-13T06:22:32.274961+00:00

## 自由属性

- 無し

