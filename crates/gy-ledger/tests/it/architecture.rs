//! The layer direction the ledger keeps (D-76, d-3bbc): store → model → ops →
//! views. A file in one layer may only use the layers to its left, so store
//! stays reached from nowhere and views never reach store. Replaces the check
//! that scripts/measure.sh item 7 used to make.
use archunit::{assert_passes, project_layers};

#[test]
fn dependencies_follow_store_model_ops_views() {
    let rule = project_layers()
        .layer("store")
        .defined_by("crates/gy-ledger/src/store/**")
        .layer("model")
        .defined_by("crates/gy-ledger/src/model/**")
        .layer("ops")
        .defined_by("crates/gy-ledger/src/ops/**")
        .layer("views")
        .defined_by("crates/gy-ledger/src/views/**")
        .where_layer("store")
        .may_only_depend_on_layers(&[])
        .where_layer("model")
        .may_only_depend_on_layers(&["store"])
        .where_layer("ops")
        .may_only_depend_on_layers(&["store", "model"])
        .where_layer("views")
        .may_only_depend_on_layers(&["model", "ops"]);

    assert_passes!(rule);
}
