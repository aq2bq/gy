//! The node: typed fields, the five kind-specific payloads, and free attributes.
use std::collections::BTreeMap;

use super::{
    Alias, Approval, Cancellation, Closed, Closure, Completion, DecisionScope, Edge, Link, NodeId,
    NodeKind, Ref, Relation, RequirementState, Revision,
};
use crate::store::{Error, Result};
use serde::{Deserialize, Serialize};

/// Free attributes carried verbatim. This is the model's only string-keyed
/// storage; every other node field is typed.
pub type FreeAttributes = BTreeMap<String, String>;

/// A node's raw attribute table: bags of string values keyed by section. Free
/// attributes live in the `attributes` bag. N-38 maps these to and from the
/// stored form.
pub type Attributes = BTreeMap<String, BTreeMap<String, String>>;

/// The single place the model reads a node attribute by a literal string key.
pub fn free_attribute<'a>(attributes: &'a Attributes, name: &str) -> Option<&'a String> {
    attributes.get("attributes").and_then(|bag| bag.get(name))
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Need {
    pub closed: Option<Closed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Question {
    pub closure: Option<Closure>,
    pub decider: Option<String>,
    pub options: Vec<String>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub scope: DecisionScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Requirement {
    pub state: RequirementState,
    pub reference: Option<Ref>,
    #[serde(default)]
    pub approval: Option<Approval>,
    #[serde(default)]
    pub revisions: Vec<Revision>,
    #[serde(default)]
    pub completion: Option<Completion>,
    #[serde(default)]
    pub cancellation: Option<Cancellation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Criterion {
    pub satisfied: bool,
    pub evidence: Option<String>,
    pub satisfied_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeData {
    Need(Need),
    Question(Question),
    Decision(Decision),
    Requirement(Requirement),
    Criterion(Criterion),
}
impl NodeData {
    pub fn kind(&self) -> NodeKind {
        match self {
            Self::Need(_) => NodeKind::Need,
            Self::Question(_) => NodeKind::Question,
            Self::Decision(_) => NodeKind::Decision,
            Self::Requirement(_) => NodeKind::Requirement,
            Self::Criterion(_) => NodeKind::Criterion,
        }
    }
}

/// A node. Its fields are typed; `free_attribute` is the single boundary that
/// reads a free attribute by string key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    id: NodeId,
    scope: String,
    created: String,
    title: String,
    body: String,
    aliases: Vec<Alias>,
    free: FreeAttributes,
    links: Vec<Edge>,
    data: NodeData,
}
impl Node {
    fn build(id: NodeId, scope: &str, created: &str, title: &str, data: NodeData) -> Result<Self> {
        if scope.trim().is_empty() {
            return Err(Error::invalid("a node needs a scope"));
        }
        if !valid_date(created) {
            return Err(Error::invalid("created must be YYYY-MM-DD"));
        }
        if title.trim().is_empty() {
            return Err(Error::invalid("a node needs a nonempty title"));
        }
        Ok(Self {
            id,
            scope: scope.to_string(),
            created: created.to_string(),
            title: title.to_string(),
            body: String::new(),
            aliases: Vec::new(),
            free: FreeAttributes::new(),
            links: Vec::new(),
            data,
        })
    }
    pub fn need(id: NodeId, scope: &str, created: &str, title: &str) -> Result<Self> {
        Self::build(id, scope, created, title, NodeData::Need(Need::default()))
    }
    pub fn question(id: NodeId, scope: &str, created: &str, title: &str) -> Result<Self> {
        Self::build(
            id,
            scope,
            created,
            title,
            NodeData::Question(Question::default()),
        )
    }
    pub fn decision(
        id: NodeId,
        scope: &str,
        created: &str,
        title: &str,
        decision_scope: DecisionScope,
    ) -> Result<Self> {
        Self::build(
            id,
            scope,
            created,
            title,
            NodeData::Decision(Decision {
                scope: decision_scope,
            }),
        )
    }
    pub fn requirement(
        id: NodeId,
        scope: &str,
        created: &str,
        title: &str,
        state: RequirementState,
    ) -> Result<Self> {
        Self::build(
            id,
            scope,
            created,
            title,
            NodeData::Requirement(Requirement {
                state,
                reference: None,
                approval: None,
                revisions: Vec::new(),
                completion: None,
                cancellation: None,
            }),
        )
    }
    pub fn criterion(id: NodeId, scope: &str, created: &str, title: &str) -> Result<Self> {
        Self::build(
            id,
            scope,
            created,
            title,
            NodeData::Criterion(Criterion::default()),
        )
    }
    pub fn id(&self) -> &NodeId {
        &self.id
    }
    pub fn scope(&self) -> &str {
        &self.scope
    }
    pub fn created(&self) -> &str {
        &self.created
    }
    pub fn kind(&self) -> NodeKind {
        self.data.kind()
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn body(&self) -> &str {
        &self.body
    }
    pub fn data(&self) -> &NodeData {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }
    pub fn aliases(&self) -> &[Alias] {
        &self.aliases
    }
    pub fn links(&self) -> &[Edge] {
        &self.links
    }
    /// Save a relationship. Only the from side stores the edge; the reverse is
    /// derived when needed.
    pub fn link(&mut self, link: Link) {
        self.links.push(link.forward().clone());
    }
    /// Drop the edge at `index`, returning it if it existed.
    pub fn unlink(&mut self, index: usize) -> Option<Edge> {
        (index < self.links.len()).then(|| self.links.remove(index))
    }
    pub fn set_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
    }
    /// Retitle a node. An empty title is rejected (invariant).
    pub fn set_title(&mut self, title: impl Into<String>) -> Result<()> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(Error::invalid("a node needs a nonempty title"));
        }
        self.title = title;
        Ok(())
    }
    pub fn add_alias(&mut self, alias: Alias) {
        self.aliases.push(alias);
    }
    pub fn free(&self, name: &str) -> Option<&String> {
        self.free.get(name)
    }
    pub fn set_free(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.free.insert(name.into(), value.into());
    }
    /// Append to a free attribute, joined by a newline; create it if absent.
    pub fn append_free(&mut self, name: &str, value: &str) {
        let joined = match self.free.get(name) {
            Some(current) => format!("{current}\n{value}"),
            None => value.to_string(),
        };
        self.free.insert(name.to_string(), joined);
    }
    pub fn state(&self) -> Option<RequirementState> {
        match &self.data {
            NodeData::Requirement(requirement) => Some(requirement.state),
            _ => None,
        }
    }
    /// Move a requirement. An invalid transition is rejected (invariant).
    pub fn advance(&mut self, to: RequirementState) -> Result<()> {
        match &mut self.data {
            NodeData::Requirement(requirement) => {
                requirement.state = requirement.state.advance(to)?;
                Ok(())
            }
            _ => Err(Error::invalid("only a requirement has a state")),
        }
    }
}

/// A `created` date in `YYYY-MM-DD` form.
fn valid_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    text.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

/// Needs that carry a Targets edge to a criterion, derived rather than stored.
pub fn bearer_count(needs: &[Node], criterion: &NodeId) -> usize {
    needs
        .iter()
        .filter(|node| {
            node.links()
                .iter()
                .any(|edge| edge.label == Relation::Targets && &edge.to == criterion)
        })
        .count()
}
