<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/logo.svg" alt="gy" width="200">

# gy — good,yes

[English](README.md) | 日本語

gy は、要求が確定するまでの状態を記録します。ある仕事が何に基づいているかを、ニーズ・論点・決定・要求・受け入れ条件のノードと辺のグラフとして持ちます。使い方は一つで、設定するものはほとんどありません。エージェントが作業しながら書き、`publish` が記録をファイルに書き出し、それをコミットして後から読み返します。canonical はリポジトリの外に置き、gy はネットワークへ出ません。

スクショは gy 自身の記録です。英語の見本の記録は `scripts/demo-ledger.sh` で作れます。

いま

<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/serve-now-ja.png" alt="いま" width="100%">

フラクタルなグラフ

<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/serve-graph-ja.png" alt="フラクタルなグラフ" width="100%">

一件

<img src="https://raw.githubusercontent.com/aq2bq/gy/main/docs/images/serve-node-ja.png" alt="一件" width="100%">

## gy とは何か

エージェントに任せた仕事は、順調なあいだは読まなくて済みます。読まなくて済むので、読まなくなります。どこまで進むかは、文脈に入る量、対象の規模、モデルの性能に依存していて、その依存は順調なうちは見えません。

見えるようになるのは終盤です。要求を完了と宣言する場面。以前の作業が依拠した決定が、気付かないうちに置き換わっている場面。残った作業をどこかへ移す場面。そして、責任を負う一人が複数のプロジェクトを同時に抱え、そのすべてを読めない場面です。四つとも、順調なあいだは起きません。起きたときには、何に基づいて進めてきたかを覚えている人がいません。

gy はその終盤のためにあります。計画も日程も持ちません。各作業が何に基づいているか、完了と呼ぶ前に何が成り立っていなければならないかを記録し、記録どうしが矛盾している箇所を報告します。後から走らせる `lint` はありません。不正な書き込みはその場で拒まれ、注意が要る状態は `handover` が件数で出します。

## 5 種のノードと 12 の辺

ノードは 5 種あります。うち 4 種は、仕事の一巡をなぞる環を作ります。決定がニーズを生み、ニーズが要求として起票され、作業が論点を生み、論点が次の決定として閉じます。残る 1 種の受け入れ条件は、環の外から作業を測ります。

| ノード | ID | 意味 |
| --- | --- | --- |
| ニーズ | `n-…` | やるべき仕事。受け入れ条件を対象にする |
| 論点 | `q-…` | 未決のこと。決定者と 2 つ以上の選択肢を持つ |
| 決定 | `d-…` | 成立範囲を持つ決定 |
| 要求 | `r-…` | 承認を求めて立てたニーズ。外への参照を 1 つ持つ |
| 受け入れ条件 | `ac-…` | 進捗を数える相手 |

辺は、始まる側のノードだけに保存します。逆向きは導出するので、同じ事実を 2 度書きません。辺は 12 の関係のいずれかで、関係ごとに許されるノードの組は決まっています。

| 向き | 関係 | 逆向き |
| --- | --- | --- |
| 論点 → 決定 | `closes` | `closed-by` |
| 決定 → 決定 | `narrows`・`widens`・`supersedes`・`completes` | `narrowed-by`・`widened-by`・`superseded-by`・`completed-by` |
| ニーズ・要求 → 受け入れ条件 | `targets` | `targeted-by` |
| ニーズ → 決定 | `spawned-by` | `spawns` |
| ニーズ → 要求 | `filed-as` | `files` |
| ニーズ → ニーズ | `depends-on` | `depended-on-by` |
| 要求 → 決定 | `relies-on` | `relied-on-by` |
| 要求 → 論点 | `raised` | `raised-by` |
| ニーズ → 論点・要求 | `waits-on` | `awaited-by` |

決定は成立範囲とともに保存します。後から読む人が、その決定がどこで成り立ち、どこでは成り立たないかを判断できるようにするためです。`narrows` と `supersedes` では、古い決定のうち効力を失う箇所も指定します。指定した文字列は辺を書く時点でその決定の本文と照合します。

## 要求の 4 状態と確定

要求の状態は 4 つです。`filed`（起票済み）、`approved`（確定）、`done`（完了）、`cancelled`（中止）。確定・改訂・完了・中止はそれぞれ 1 つのコマンドで、設計を誰から聞いたか、根拠、理由を記録します。それ以外の遷移は拒まれます。

| 遷移元 | 遷移先 | コマンド |
| --- | --- | --- |
| filed | approved | `req approve` |
| approved | filed | `req revise` |
| approved | done | `req done` |
| filed・approved | cancelled | `req cancel` |

確定（approved）は、記録の外の作業の関門です。gy は記録の外で作られたものを見られませんが、その結果を書き留めることは拒みます。`criterion satisfy` は、approved か done の要求がその受け入れ条件を `targets` で指しているときだけ通り、拒むときは次に打つコマンド（`req add …` か `req approve …`）を言います。要求が approved のあいだは、その題・本文・`targets`・`relies-on` と、それが指す受け入れ条件の題・本文は編集できません。変えるときは先に `req revise` で filed に戻します。誰が承認するかは利用者の方針で、gy は裁きません。同梱の `gy-loop` スキルは、要求を毎回自分で読むか、最初の 1 件だけ読むか、エージェントに任せるかを、エージェントに一度だけ尋ねさせます。

確定より後のことについて gy が持つのは、この 4 つの記録だけです。実装をどこで進めるか、どう設計するか、いつ監査するかは gy の外にあり、要求の ref が指す先に置きます。

## canonical の置き場所

canonical はリポジトリの外、`$XDG_DATA_HOME/gy/<リポジトリのルートのハッシュ>/` にある追記専用のイベントログです。リポジトリに置くのは `gy.toml` だけです。書き込みは 1 回で 1 トランザクションとしてログに追記し、連番・時刻・書き手・理由・出典を残します。その場で書き換えることはありません。

`undo --reason <文>` は直前のトランザクションを逆にたどる新しいトランザクションを追記します。間違いと訂正の両方が履歴に残ります。戻せるのは直前の 1 件だけで、続けて打つと直前の undo 自身を戻します（redo）。ログが canonical で、隣のスナップショットファイルは開くのを速くするだけで、消しても構いません。

## 26 の操作

最初の書き込みの前に (1):

| 操作 | 結果 |
| --- | --- |
| `init <scope>` | ここでリポジトリを始める。gy.toml にスコープを 1 つ書いてから、読むべきスキルと最初に起票するノードを名指しする。既に gy.toml があれば報告だけして触らない |

読み (9):

| 操作 | 結果 |
| --- | --- |
| `show <ID\|ref>... [--full]` | ID・別名・ref でノードを表示し、足りないものも出す |
| `list [--type] [--status] [--targets] [--grep] [--actor] [--since]` | ノードの一覧。`--actor` か `--since` を与えると書き込み単位。種別と状態の綴りは大小を問わず、`--since` は seq か日付（`YYYY-MM-DD`、あなたの場所のその日の 0 時から） |
| `next` | 前提の片付いたニーズ |
| `handover` | 進行中の要求と、再開に要る件数 |
| `publish [--scope] [--since] [--out]` | 指定した時点と範囲の記録をディレクトリに書く: 1 ノード 1 ファイルとスコープの索引 |
| `share <URL>` | 共有を始める（experimental）: remote を検査し、`gy.toml` に `remote` を書き、今の記録を上げ、守りと招待の文を出す |
| `join` | 参加する（experimental）: 要るものを確かめて直し方を並べ、複製を取り、誰として書くかと次の一手を言う。何度打っても同じ |
| `sync` | remote と同期する（experimental）。複製が無ければ取り、未 push の書きを 1 行 1 commit で push し、remote が先なら取り込んで載せ直す。異常時は原因と手段を出す |
| `serve` | 記録をブラウザで読む。127.0.0.1 で（GET だけ、書く経路は無い）、止めるまで。端末から起動したときはブラウザを開く |

書き (16):

| 操作 | 結果 |
| --- | --- |
| `need add "<題>" --targets <AC>... [--spawned-by <D>] [--body-file <path>]` | 受け入れ条件を対象にニーズを立てる |
| `need close <ID> --by fact\|external --evidence <文>` | 要求を経由せずニーズを閉じる |
| `question add "<題>" --decider <名> --options <文>... [--body-file <path>]` | 決定者と 2 つ以上の選択肢を持つ論点を立てる |
| `question close <ID> --by fact\|decision\|non-decision --evidence <文> [--decision <D>]` | 論点を閉じ、決定で閉じる場合は決定も残す |
| `criterion add "<題>" [--body-file <path>]` | 受け入れ条件を追加する |
| `criterion satisfy <AC> --evidence <文> [--revoke]` | 受け入れ条件が成り立つ根拠を記録する。`--revoke` で取り消す |
| `req add "<題>" --need <N>... [--relies-on <D>]... [--targets <AC>]... [--ref <ref>] [--body-file <path>]` | ニーズ・決定・受け入れ条件に対して要求を立てる |
| `req approve <ID\|ref> --design <文> --heard-by <名> --evidence <文>` | 要求の設計を確定する |
| `req revise <ID> --reason <文> --source <文>` | 確定した要求を起票済みへ戻す |
| `req done <ID> --evidence <文>` | 確定した要求が完了したことを記録する |
| `req cancel <ID> --reason <文> --source <文>` | 完了しなかった要求を中止する |
| `decide "<題>" --scope-note <文> [--body-file <path>] [--closes <Q>]... [--relate <関係> <D> --mark <文>] [--source <文>]` | 決定を作り、論点を閉じ、系譜の辺を 1 つ記録する |
| `link <from> <関係> <to> [--mark <文>] [--remove]` | 辺を 1 つ追加または削除する |
| `edit <ID> --reason <文> [--title] [--body-file] [--set k=v] [--append k=v]` | 題名・本文・自由属性を変える。自由属性は文字列で、`--set` は上書き、`--set k=` は消去、`--append` は改行区切りで 1 行足す。`--set scope=<名前>` で gy.toml にあるスコープへ移せ、`--set decision_scope=<文>` で未記録の成立範囲を 1 回だけ記録できる |
| `scope rename <旧> <新>` | あるスコープの全ノードを新しい名へ移し、gy.toml をコメントと順序を保ったまま書き換える |
| `undo --reason <文>` | 直前のトランザクションを打ち消す |

書き込みはどれも、変えたもの、そのノードにまだ無いもの、次に打てるコマンドの形を出力します。次の一手は別の手順書を見なくても分かります。ノードを作る書き込み（`need add` / `question add` / `criterion add` / `decide` / `req add`）は、先頭に `id: <ID>` を出します。`--json` を付けると同じ内容をデータで返します。

## 日付と時刻

ノードの `created`、受け入れ条件の `satisfied_at`、要求の記録の日付は UTC の瞬間として保存され、`show` と `list` はあなたの場所（`TZ`）の `YYYY-MM-DD HH:MM` で出します。`--json` と `publish` は保存の値（`2026-09-18T07:28:56Z`）のままです。チームで時点を指すときは日付ではなく seq か ID を使ってください。

## セッションの再開

新しいセッションは 3 つのコマンドで始めます。`handover` は進行中の要求を ref 付きで示し、開いている論点の数、着手できるニーズの数、エラーと警告の件数を出します。`next` は前提の片付いたニーズを列挙し、エージェントがそのうち 1 つを依頼者（エージェントに仕事を頼む人）へ差し出します。`show` は 1 つのノードを読む。

```sh
gy handover
gy next
gy show n-3f9a
```

## gy.toml

設定するのはスコープ名と、必要なら `publish` の出力先、チームで使うなら記録専用の git remote だけです。それ以外のキーがあると、読み込みの時点で拒まれます。表の外のキーは最初の `[scopes.*]` より前に書きます。

```toml
# 任意。--out を付けないときの publish の出力先。
output = "docs/publication"
# 任意（experimental）。記録専用の git repo。チームで使う節を参照。
remote = "https://github.com/you/yourproject-ledger.git"

[scopes.myproject]
```

読みはすべてのスコープを対象にします。書き込みでスコープが要るのは、ファイルが 2 つ以上のスコープを挙げているときだけです。`--scope <名>` で選びます。最初の書き込みが記録を作り、読みが作ることはありません。

## チームで使う（experimental）

記録は、それ専用の git repo 1 つを canonical にして共有します。構成員は人で、各自が自分のマシンと記録の複製を持ちます。始める人は `gy share` を、招かれた人は `gy join` を本人が打ち、エージェントはその人の複製に今までどおり書くだけです。同じマシンの複数エージェントは元々ローカル記録を共有していて remote は要りません。手続きは 2 つで、どちらも gy が次の一手を言います。

**共有を始める（単独で使っていた人）**。GitHub に空の private repo を 1 つ作り、プロジェクトの checkout で打ちます。

```sh
gy share https://github.com/you/yourproject-ledger.git
```

remote を検査し（空か記録専用か、push できるか）、`gy.toml` に `remote` を書き、今の記録をそのまま上げ、branch protection の付け方（linear history の必須、force push の禁止。gy は設定を変えません）と、メンバーへの招待の文を出します。`gy.toml` はプロジェクトと一緒にコミットします。

この一歩はエージェントではなく、あなたの作業です。`gy share` は記録の全体をその repo へ上げます。エージェントの実行環境はこれを外部への送信と見なして拒むことがあり、エージェントは自分にその許可を与えられません。repo を作るのと branch を守るのがあなたの作業であるのと同じく、`gy share` もあなたが一度だけ打つか、エージェントの設定で `gy share` と `gy sync` をあなたが許可してください。その後は、エージェントは今までどおり書き、push は背景で行われます。

**参加する（招待された人）**。記録の repo の write の権限をもらい、プロジェクトを clone して打ちます。

```sh
gy join
```

要るもの（git、repo を読める資格、名乗る名前 — `git config user.name` と `user.email`、または `GIT_AUTHOR_NAME` と `GIT_AUTHOR_EMAIL`）を一度に確かめ、足りなければ直し方を並べて止まります。揃っていれば複製を取り、誰として書くことになるか（`user.name / GY_ACTOR`）と次の一手（`gy handover`）を言います。何度打っても同じ状態に落ち着きます。`gy join` を打たずに `gy handover` から入っても最初のコマンドが複製を取り、同じ「joined」の 1 行を出します。

**それからの使い方は今までと同じです。**

- 書きは手元に即座に載り、push は背景で行われます（書きの直後と、`gy serve` の起動中は 10 秒ごと）。`gy sync` で明示にも同期できます。remote に届かない間も読み書きは通り、戻れば溜まった分がまとめて push されます。
- remote が先に進んでいたら、自分の未 push の書きはその後ろに載せ直されます。同じノードを相手が先に変えていた書きだけが拒まれ、次の gy コマンドの標準エラーと `handover` で本人にだけ知らされます。やり直すかどうかはあなたが決めます。
- 書き手は `user.name / GY_ACTOR` で記録され、`list` と画面にそう出ます。同じエージェント名でも人間が違えば別の書き手です。名前は git がコミットに署名するのと同じもので、git と同じ順（`GIT_AUTHOR_NAME` が先、無ければ `git config user.name`）で決まります。1 回の書きを記録と git の履歴が別々の人の名前で指すことはありません。
- remote は gy だけが書く場所です。1 書き = 1 commit で、履歴が gy 以外に変えられていれば同期が拒んで戻し方を示します。記録以外の内容がある repo は拒みます。
- `gy.toml` の `remote` の行を消せば手元だけの記録に戻ります（次のコマンドが 1 回だけ知らせます）。戻した後にまた繋ぐと、両方が進んでいれば拒まれます。合流は持ちません。

記録には判断に至るやり取りが入ります。remote に置くということは、その記録が GitHub にある、ということです。

## 書き手

書き込みはどれも `GY_ACTOR` で書き手を名乗ります。未設定か空ならエラーになり、名前は理由と出典と並んで履歴に残ります。

```sh
export GY_ACTOR=leader
```

エージェントは各自の名前で自分の記録を書くので、履歴を見れば誰が何をなぜ変えたかが分かります。

## ID と別名と ref

ID は gy が振ります。種類の接頭辞と短いハッシュで、`n-3f9a` のような形です。ハッシュには少なくとも 1 文字の a〜f が含まれるので、旧 ID と見分けがつきます。中央の採番器を持たないので、2 人のエージェントが同時に書いても衝突せず、衝突すればハッシュが長くなるだけです。`show` は ID の完全一致、別名（0 埋めと大文字小文字は同一視するので `D-8` = `D-08`）、要求の外への参照を、完全一致か末尾一致で受け付けます。

既存の記録は旧 ID を別名として保つので、`show D-164` でも `show '#6027'` でも改名後のノードに届きます。要求の参照（`--ref`）は不透明な値で、gy は保存するだけで、その先を読みません。

## publish

`publish` は、指定した時点と範囲の記録をディレクトリに書き出します。`<out>/<scope>/<種類>/` に 1 ノード 1 ファイル、`<out>/<scope>/README.md` にスコープの索引を置きます。これは開発の成果物です。後からエージェントが読み、何をなぜ決めたかを振り返り、公開物どうしを差分で比べます。依頼者が読むものではなく、gy は人間向けの出力の形を足しません。

ノードのファイルには、別名と ref 付きの ID、題名、scope、created、状態、成立範囲、本文、mark 付きの両向きの辺（相手の ID・別名・題名）、閉じ方と根拠、要求の記録、自由属性を入れます。参照はすべて相手の題名を伴うので、各ファイルは単体で分かります。決定のファイルは系譜の関係を先頭に置きます。

索引には、生成日時・ログの seq・scope と since・書き手・canonical の場所、短い「読み方」の節、種類ごとの一覧（各ノードのリンクと状態）、書き込みの履歴、診断を入れます。

`--out` は出力先のディレクトリ、`gy.toml` の `output` は既定の出力先を指定し、どちらも無ければエラーです。対象スコープのディレクトリだけを消して作り直し、`--out` の他のファイルと他のスコープのディレクトリには触りません。

## インストール

```sh
cargo install gy --locked
```

チェックアウトから入れるなら `cargo install --path crates/gy --locked` を実行します。バイナリ名は `gy` です。

0.4 から来る場合は記録を一度だけ移します。手順と写せないものは [docs/migration-0.5.md](docs/migration-0.5.md) にあります。

## スキルの置き方

エージェント用のスキル（入口の `gy-loop` と早見表、`gy-ledger`、`gy-question`、`gy-decide`）は crate の `skills/` に同梱されていますが、`cargo install` は置いてくれません。エージェントがスキルを読む場所（例: `~/.agents/skills`）へ写します。

```sh
# チェックアウトから
cp -R crates/gy/skills/gy-* ~/.agents/skills/
# レジストリの写しから（版を合わせる）
cp -R ~/.cargo/registry/src/*/gy-1.0.0/skills/gy-* ~/.agents/skills/
```

スキルが変わった版は CHANGELOG の Updating にそう書いてあります。その版に上げたら写し直し、エージェントから gy のスキルが全部見えることを確かめてください。スキルごとのリンク（例: `~/.claude/skills/gy-ledger` → `~/.agents/skills/gy-ledger`）で読ませている場合は、増えたスキルのリンクを足す必要があります。記録の形式の版が上がったときは、gy が移行の直後に標準エラーへ 1 行で知らせ、CHANGELOG の Updating を指します。

## 開発

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

ワークスペースは `gy-ledger`（保存・モデル・操作）、`gy-serve`（読み取り専用の Web 表示）、`gy`（CLI）で構成します。CI は Linux で検証し、他のプラットフォームは未検証です。`gy serve` の画面には `e2e/`（Playwright、chromium）の E2E があり、[e2e/README.md](e2e/README.md) を読みます。`cargo test` には入りません。
