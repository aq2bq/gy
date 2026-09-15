//! The node: typed fields, the five kind-specific payloads, and free attributes.
use std::collections::BTreeMap;

use super::{Alias, Closure, DecisionScope, Link, NodeId, NodeKind, Ref, RequirementState};
use crate::store::{Error, Result};

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Need {
    pub targets: Vec<NodeId>,
    pub spawned_by: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Question {
    pub closure: Option<Closure>,
    pub decider: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub scope: DecisionScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Requirement {
    pub state: RequirementState,
    pub reference: Option<Ref>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Criterion {
    pub satisfied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    id: NodeId,
    title: String,
    body: String,
    aliases: Vec<Alias>,
    free: FreeAttributes,
    links: Vec<Link>,
    data: NodeData,
}
impl Node {
    fn build(id: NodeId, title: impl Into<String>, data: NodeData) -> Result<Self> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(Error::invalid("a node needs a nonempty title"));
        }
        Ok(Self {
            id,
            title,
            body: String::new(),
            aliases: Vec::new(),
            free: FreeAttributes::new(),
            links: Vec::new(),
            data,
        })
    }
    pub fn need(id: NodeId, title: impl Into<String>) -> Result<Self> {
        Self::build(id, title, NodeData::Need(Need::default()))
    }
    pub fn question(id: NodeId, title: impl Into<String>) -> Result<Self> {
        Self::build(id, title, NodeData::Question(Question::default()))
    }
    pub fn decision(id: NodeId, title: impl Into<String>, scope: DecisionScope) -> Result<Self> {
        Self::build(id, title, NodeData::Decision(Decision { scope }))
    }
    pub fn requirement(
        id: NodeId,
        title: impl Into<String>,
        state: RequirementState,
    ) -> Result<Self> {
        Self::build(
            id,
            title,
            NodeData::Requirement(Requirement {
                state,
                reference: None,
            }),
        )
    }
    pub fn criterion(id: NodeId, title: impl Into<String>) -> Result<Self> {
        Self::build(id, title, NodeData::Criterion(Criterion::default()))
    }
    pub fn id(&self) -> &NodeId {
        &self.id
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
    pub fn links(&self) -> &[Link] {
        &self.links
    }
    pub fn link(&mut self, link: Link) {
        self.links.push(link);
    }
    pub fn set_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
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

/// Needs that target a criterion, derived rather than stored (N-36).
pub fn bearer_count(needs: &[Node], criterion: &NodeId) -> usize {
    needs
        .iter()
        .filter(
            |node| matches!(&node.data, NodeData::Need(need) if need.targets.contains(criterion)),
        )
        .count()
}
