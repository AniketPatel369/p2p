use nat_traversal::{
    decide_route, gather_candidates, should_attempt_hole_punch, NatType, Route,
};
use std::net::SocketAddr;

fn addr(s: &str) -> SocketAddr {
    s.parse().expect("valid socket addr")
}

#[test]
fn chooses_direct_when_both_have_reflexive_candidates() {
    let a = gather_candidates(
        addr("192.168.1.10:5000"),
        Some(addr("203.0.113.10:5000")),
        Some(addr("198.51.100.1:7000")),
    );
    let b = gather_candidates(
        addr("10.0.0.2:5001"),
        Some(addr("203.0.113.20:5001")),
        Some(addr("198.51.100.2:7000")),
    );

    let plan = decide_route(NatType::RestrictedCone, NatType::FullCone, &a, &b);
    assert_eq!(plan.route, Route::Direct);
}

#[test]
fn chooses_relay_when_symmetric_nat_detected_and_relay_available() {
    let a = gather_candidates(
        addr("192.168.1.10:5000"),
        Some(addr("203.0.113.10:5000")),
        Some(addr("198.51.100.1:7000")),
    );
    let b = gather_candidates(addr("10.0.0.2:5001"), Some(addr("203.0.113.20:5001")), None);

    let plan = decide_route(NatType::Symmetric, NatType::RestrictedCone, &a, &b);
    assert_eq!(plan.route, Route::Relay);
}

#[test]
fn falls_back_to_direct_when_no_relay_available() {
    let a = gather_candidates(addr("192.168.1.10:5000"), None, None);
    let b = gather_candidates(addr("10.0.0.2:5001"), None, None);

    let plan = decide_route(NatType::Symmetric, NatType::Symmetric, &a, &b);
    assert_eq!(plan.route, Route::Direct);
    assert!(plan.reason.contains("relay unavailable"));
}

#[test]
fn hole_punch_not_attempted_for_symmetric_nat() {
    assert!(!should_attempt_hole_punch(
        NatType::Symmetric,
        NatType::RestrictedCone
    ));
    assert!(should_attempt_hole_punch(
        NatType::RestrictedCone,
        NatType::FullCone
    ));
}

#[test]
fn relay_used_when_reflexive_missing_but_relay_present() {
    let a = gather_candidates(
        addr("192.168.1.10:5000"),
        None,
        Some(addr("198.51.100.1:7000")),
    );
    let b = gather_candidates(addr("10.0.0.2:5001"), None, None);

    let plan = decide_route(NatType::Unknown, NatType::Unknown, &a, &b);
    assert_eq!(plan.route, Route::Relay);
}
