use installer_update::{
    evaluate_update, rollback_marker, validate_manifest, InstallPolicy, PackageManifest,
    UpdateChannel,
};

fn base_manifest() -> PackageManifest {
    PackageManifest {
        version: "1.2.0".to_string(),
        channel: UpdateChannel::Stable,
        platform: "linux-x86_64".to_string(),
        package_url: "https://example.com/p2p-1.2.0.tar.gz".to_string(),
        sha256: "a".repeat(64),
        rollback_from: Some("1.1.0".to_string()),
    }
}

#[test]
fn manifest_validation_accepts_valid_manifest() {
    let m = base_manifest();
    validate_manifest(&m, &InstallPolicy::default()).expect("manifest valid");
}

#[test]
fn manifest_validation_rejects_non_https_when_required() {
    let mut m = base_manifest();
    m.package_url = "http://example.com/p2p.tar.gz".to_string();
    let err = validate_manifest(&m, &InstallPolicy::default()).expect_err("non https denied");
    assert!(err.to_string().contains("policy violation"));
}

#[test]
fn update_decision_blocks_downgrade_by_default() {
    let mut m = base_manifest();
    m.version = "1.0.0".to_string();

    let decision = evaluate_update("1.1.0", UpdateChannel::Stable, &m, &InstallPolicy::default())
        .expect("decision");

    assert!(!decision.allowed);
    assert_eq!(decision.reason, "downgrade blocked by policy");
}

#[test]
fn update_decision_allows_newer_version() {
    let m = base_manifest();
    let decision = evaluate_update("1.1.0", UpdateChannel::Stable, &m, &InstallPolicy::default())
        .expect("decision");
    assert!(decision.allowed);
    assert_eq!(decision.reason, "update accepted");
}

#[test]
fn rollback_marker_is_generated() {
    let marker = rollback_marker("1.1.0", "1.2.0");
    assert_eq!(marker, "rollback:1.1.0<-1.2.0");
}
