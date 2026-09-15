//! The scope index README: heading, reading, lists, history, and diagnostics
//! (d-7c64).
use super::super::derive::need_state;
use super::{FileEntry, diagnostics, history, path, plural, reference};
use crate::model::{Node, NodeData, NodeKind};
use crate::ops::repository::{Repository, Result, Store};
use std::fmt::Write;

pub(super) fn index<S: Store>(
    repository: &Repository<S>,
    all: &[Node],
    scope: &str,
    since: Option<u64>,
    seq: u64,
    writer: &str,
    location: &str,
) -> Result<FileEntry> {
    let mut out = header(scope, seq, since, writer, location);
    out.push_str("\n## 読み方\n\n");
    out.push_str(reading());
    out.push_str("\n## 一覧\n\n");
    out.push_str(&lists(all, scope));
    out.push_str("\n## 履歴\n\n");
    out.push_str(&history::history(repository, all, scope, since)?);
    out.push_str("\n## 診断\n\n");
    out.push_str(&diagnostics::diagnostics(repository, Some(scope))?);
    Ok(FileEntry {
        path: "README.md".to_string(),
        text: out,
    })
}

fn header(scope: &str, seq: u64, since: Option<u64>, writer: &str, location: &str) -> String {
    let generated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let since = since.map_or("(先頭から)".to_string(), |since| since.to_string());
    format!(
        "# gy の公開物 — {scope}\n\n\
         - 生成: {generated}\n\
         - seq: {seq}\n\
         - scope: {scope}\n\
         - since: {since}\n\
         - 書き手: {writer}\n\
         - 正本: {location}\n"
    )
}

/// The per-kind list, each line a link to the node's file and its state.
fn lists(all: &[Node], scope: &str) -> String {
    let mut out = String::new();
    for kind in NodeKind::ALL {
        let _ = writeln!(out, "### {}", plural(kind));
        let mut nodes: Vec<&Node> = all
            .iter()
            .filter(|node| node.kind() == kind && node.scope() == scope)
            .collect();
        nodes.sort_by_key(|node| (node.created().to_string(), node.id().to_string()));
        if nodes.is_empty() {
            out.push_str("- 無し\n");
            continue;
        }
        for node in nodes {
            match state(node, all) {
                Some(state) => {
                    let _ = writeln!(out, "- [{}]({}) — {state}", reference(node), path(node));
                }
                None => {
                    let _ = writeln!(out, "- [{}]({})", reference(node), path(node));
                }
            }
        }
    }
    out
}

/// A node's state as a list shows it; a decision has none.
fn state(node: &Node, all: &[Node]) -> Option<String> {
    match node.data() {
        NodeData::Need(_) => Some(need_state(node, all).name().to_string()),
        NodeData::Question(data) => Some(
            if data.closure.is_some() {
                "closed"
            } else {
                "open"
            }
            .to_string(),
        ),
        NodeData::Requirement(data) => Some(data.state.name().to_string()),
        NodeData::Criterion(data) => Some(
            if data.satisfied {
                "satisfied"
            } else {
                "unsatisfied"
            }
            .to_string(),
        ),
        NodeData::Decision(_) => None,
    }
}

/// What a reader needs to read the rest without the ledger.
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
     - waits-on: ニーズ → 論点・要求。\n\
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
