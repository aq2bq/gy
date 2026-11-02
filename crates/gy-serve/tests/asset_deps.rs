//! Every embedded asset has one role. JavaScript dependency and mutation
//! checks run on the parsed syntax in `e2e/unit/architecture.test.mjs`.
use gy_serve::assets;
use std::collections::BTreeMap;

#[test]
fn every_asset_names_its_role() {
    let roles: BTreeMap<String, String> =
        serde_json::from_str(include_str!("asset-roles.json")).unwrap();
    let mut files: Vec<&str> = assets::FILES.iter().map(|(name, _)| *name).collect();
    files.sort_unstable();
    assert_eq!(roles.keys().map(String::as_str).collect::<Vec<_>>(), files);
    for (name, role) in roles {
        assert!(
            ["core", "region", "part", "asset", "network"].contains(&role.as_str()),
            "{name} has the unknown role {role}"
        );
    }
}
