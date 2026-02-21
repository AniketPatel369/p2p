use lan_offline::{LanOfflineGuard, LanPolicy, PolicyDecision};
use std::net::SocketAddr;

#[test]
fn allows_private_and_denies_public_in_offline_mode() {
    let guard = LanOfflineGuard::new(LanPolicy::default());

    let private: SocketAddr = "192.168.1.10:9000".parse().expect("private");
    let public: SocketAddr = "8.8.8.8:53".parse().expect("public");

    assert_eq!(guard.evaluate_peer(private), PolicyDecision::Allow);
    assert_eq!(
        guard.evaluate_peer(public),
        PolicyDecision::Deny("public internet address denied in offline mode")
    );
}

#[test]
fn allows_link_local_ipv4_and_ipv6() {
    let guard = LanOfflineGuard::new(LanPolicy::default());

    let ll4: SocketAddr = "169.254.10.20:8080".parse().expect("ll4");
    let ll6: SocketAddr = "[fe80::1234]:8080".parse().expect("ll6");

    assert_eq!(guard.evaluate_peer(ll4), PolicyDecision::Allow);
    assert_eq!(guard.evaluate_peer(ll6), PolicyDecision::Allow);
}

#[test]
fn validate_peer_set_fails_on_public_member() {
    let guard = LanOfflineGuard::new(LanPolicy::default());
    let peers: Vec<SocketAddr> = vec![
        "10.0.0.4:7000".parse().expect("private"),
        "8.8.4.4:53".parse().expect("public"),
    ];

    let err = guard
        .validate_peer_set(peers.iter())
        .expect_err("public address should fail");
    assert!(err
        .to_string()
        .contains("public internet address denied in offline mode"));
}

#[test]
fn disabling_offline_mode_allows_public_addresses() {
    let mut guard = LanOfflineGuard::new(LanPolicy::default());
    guard.disable_offline_mode();

    let public: SocketAddr = "1.1.1.1:443".parse().expect("public");
    assert_eq!(guard.evaluate_peer(public), PolicyDecision::Allow);
}

#[test]
fn deny_private_when_policy_disables_it() {
    let policy = LanPolicy {
        allow_private: false,
        ..LanPolicy::default()
    };
    let guard = LanOfflineGuard::new(policy);

    let private: SocketAddr = "10.1.2.3:1234".parse().expect("private");
    assert_eq!(
        guard.evaluate_peer(private),
        PolicyDecision::Deny("private-range denied")
    );
}
