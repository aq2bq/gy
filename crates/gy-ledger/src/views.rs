//! Derived views: show, list, next, handover, and publish (N-39, N-40). This
//! skeleton fixes the layer; the projections arrive later.
use crate::model::NodeKind;

pub enum View {
    Show,
    List,
    Next,
    Handover,
    Publish,
}

/// The node kinds a view reads. A placeholder boundary for N-39 / N-40.
pub fn reads(_view: View) -> &'static [NodeKind] {
    &NodeKind::ALL
}
