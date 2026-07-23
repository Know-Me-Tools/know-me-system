//! Verifies the migrated `hello-knowme` example manifest parses with the
//! host's `parse_manifest` (plugin-wit-convergence, task 5.2).

use std::path::Path;

#[test]
fn hello_knowme_manifest_parses_and_references_hook_world() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("plugins/examples/hello-knowme/manifest.json");
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let manifest = knowme_plugin_host::parse_manifest(&json).expect("manifest parses");
    assert_eq!(manifest.id, "tools.know-me.examples.hello");
    assert_eq!(
        manifest.permissions,
        vec![knowme_plugin_host::PluginPermission::Storage]
    );
    // The 1.0.0 world reference rides along as an additive field; the v0.1
    // manifest schema ignores unknown fields by design.
    let raw: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(raw["world"], "knowme:plugin/hook@1.0.0");
}
