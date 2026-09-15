//! publish: the record at a point and range (D-61, D-71, d-edb0). It writes
//! every node verbatim, the write history, and the diagnostics into one
//! Markdown file that stands on its own. It is a development artifact to
//! review later, not a reading for the master.
mod diagnostics;
mod history;
mod nodes;

use crate::model::Node;
use crate::ops::repository::{Repository, Result, Store};

/// The whole publication for one scope and range. The writer and the canonical
/// location come from the caller, so the view reads no environment.
pub fn publish<S: Store>(
    repository: &Repository<S>,
    scope: Option<&str>,
    since: Option<u64>,
    writer: &str,
    location: &str,
) -> Result<String> {
    let seq = log_seq(repository);
    let mut out = header(seq, scope, since, writer, location);
    out.push_str("\n## 読み方\n\n");
    out.push_str(reading());
    out.push_str("\n## 記録\n\n");
    out.push_str(&nodes::nodes(repository, scope, since)?);
    out.push_str("\n## 履歴\n\n");
    out.push_str(&history::history(repository, since)?);
    out.push_str("\n## 診断\n\n");
    out.push_str(&diagnostics::diagnostics(repository, scope)?);
    Ok(out)
}

/// The last write sequence, which is the publication's point in time.
pub fn log_seq<S: Store>(repository: &Repository<S>) -> u64 {
    repository
        .store()
        .history()
        .last()
        .map_or(0, |entry| entry.seq)
}

fn header(
    seq: u64,
    scope: Option<&str>,
    since: Option<u64>,
    writer: &str,
    location: &str,
) -> String {
    let generated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let scope = scope.unwrap_or("(すべて)");
    let since = since.map_or("(先頭から)".to_string(), |since| since.to_string());
    format!(
        "# gy の公開物\n\n\
         - 生成: {generated}\n\
         - seq: {seq}\n\
         - scope: {scope}\n\
         - since: {since}\n\
         - 書き手: {writer}\n\
         - 正本: {location}\n"
    )
}

/// What a reader needs to read the rest of the document without the ledger.
fn reading() -> &'static str {
    "ノードの種類（5 種）:\n\
     - need（n-…）: これから行う作業。受け入れ条件を対象にする。\n\
     - question（q-…）: まだ決めていないこと。決定者と 2 つ以上の選択肢を持つ。\n\
     - decision（d-…）: 条件付きの決定。成立範囲（decision_scope）を持つ。\n\
     - requirement（r-…）: ニーズを承認に回したもの。外への参照（ref）を持つ。\n\
     - criterion（ac-…）: 受け入れ条件。作業の進みを数える相手。\n\n\
     要求の状態（4 つ）:\n\
     - filed: 起票済み。設計はまだ承認されていない。\n\
     - approved: 確定。マスターの設計承認がある。\n\
     - done: 完了。出荷の根拠が記録されている。\n\
     - cancelled: 中止。完了以外の理由で閉じた。\n\n\
     関係（12）と向き:\n\
     - closes: 論点 → 決定。\n\
     - narrows / widens / supersedes / completes: 決定 → 決定。\n\
     - targets: ニーズ / 要求 → 受け入れ条件。\n\
     - spawned-by: ニーズ → 決定。\n\
     - filed-as: ニーズ → 要求。\n\
     - depends-on: ニーズ → ニーズ。\n\
     - relies-on: 要求 → 決定。\n\
     - raised: 要求 → 論点。\n\
     - waits-on: ニーズ → 論点・要求（論点が閉じる、または要求が完了・中止になるまで待つ）。\n\
     辺は始点のノードだけに保存し、逆向きは導出する。この文書は両向きを出す。\n\n\
     論点の閉じ方:\n\
     - fact: 事実で閉じた。 - decision: 決定で閉じた。 - non-decision: 決定を伴わずに閉じた。\n\n\
     ID と別名:\n\
     - ID は種類の接頭辞と短いハッシュ（例 n-3f9a）。旧 ID があれば別名として併記する（例 D-78）。\n\
     - 参照はすべて「ID（別名） 題名」で書く。要求は ref（外への参照）も持つ。\n\n\
     履歴の項目:\n\
     - seq: 書き込みの連番（時点）。 - 日時・書き手: いつ・誰が（GY_ACTOR）。\n\
     - ノード: 対象の ID と題名。 - 何を: created / updated / deleted。\n\
     - なぜ: 操作の名前と対象。 - 出典: 操作に渡した根拠や URL。\n"
}

/// A node as another line refers to it: new ID, old alias when there is one,
/// and title (d-edb0).
pub(super) fn reference(node: &Node) -> String {
    match node.aliases().first() {
        Some(alias) => format!("{} ({}) {}", node.id(), alias.0, node.title()),
        None => format!("{} {}", node.id(), node.title()),
    }
}

pub(super) fn in_scope(node: &Node, scope: Option<&str>) -> bool {
    scope.is_none_or(|scope| node.scope() == scope)
}
