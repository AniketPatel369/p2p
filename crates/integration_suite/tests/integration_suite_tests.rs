use integration_suite::{
    e2e_route_for_lan_and_relay, lifecycle_security_and_telemetry_validation,
    plaintext_and_encrypted_paths_coexist, required_mode_rejects_plaintext_frame,
    wire_discovery_to_ui_and_transfer,
};
use nat_traversal::Route;

#[test]
fn cross_module_wiring_discovery_to_ui_to_transfer_works() {
    let complete = wire_discovery_to_ui_and_transfer().expect("wiring should succeed");
    assert!(complete);
}

#[test]
fn e2e_scenarios_cover_lan_direct_and_relay_fallback() {
    let (lan_route, relay_route) = e2e_route_for_lan_and_relay();
    assert_eq!(lan_route, Route::Direct);
    assert_eq!(relay_route, Route::Relay);
}

#[test]
fn lifecycle_validation_checks_security_and_telemetry_redaction() {
    let (events_count, redacted, has_negotiated_event) =
        lifecycle_security_and_telemetry_validation().expect("lifecycle");
    assert_eq!(events_count, 3);
    assert!(redacted);
    assert!(has_negotiated_event);
}

#[test]
fn plaintext_and_encrypted_paths_both_work_for_migration() {
    let (plaintext_ok, encrypted_ok) = plaintext_and_encrypted_paths_coexist().expect("compat");
    assert!(plaintext_ok);
    assert!(encrypted_ok);
}

#[test]
fn required_mode_policy_rejects_plaintext_frame() {
    let status = required_mode_rejects_plaintext_frame().expect("reject plaintext");
    assert_eq!(status, "rejected");
}
