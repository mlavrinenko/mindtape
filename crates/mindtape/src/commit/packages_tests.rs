#![allow(clippy::unwrap_used)]

use super::PackageCache;
use typst::syntax::package::PackageSpec;

#[test]
fn a_pin_cache_keeps_only_its_latest_tree() {
    let spec = PackageSpec {
        namespace: "local".into(),
        name: "mindtape".into(),
        version: "0.3.0".parse().unwrap(),
    };
    let mut cache = PackageCache::default();
    cache.insert("mindtape", "tree-one", Some((spec.clone(), Vec::new())));
    assert!(cache.get("mindtape", "tree-one").is_some());
    cache.insert("mindtape", "tree-two", Some((spec, Vec::new())));
    assert!(cache.get("mindtape", "tree-one").is_none());
    assert!(cache.get("mindtape", "tree-two").is_some());
}
