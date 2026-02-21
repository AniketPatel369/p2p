use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    OpenInternet,
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Direct,
    Relay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSet {
    pub local_candidate: SocketAddr,
    pub stun_reflexive_candidate: Option<SocketAddr>,
    pub relay_candidate: Option<SocketAddr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectivityPlan {
    pub route: Route,
    pub reason: &'static str,
}

pub fn gather_candidates(
    local_candidate: SocketAddr,
    stun_reflexive_candidate: Option<SocketAddr>,
    relay_candidate: Option<SocketAddr>,
) -> CandidateSet {
    CandidateSet {
        local_candidate,
        stun_reflexive_candidate,
        relay_candidate,
    }
}

/// Decide direct vs relay route from NAT signals and available candidates.
pub fn decide_route(
    local_nat: NatType,
    remote_nat: NatType,
    local: &CandidateSet,
    remote: &CandidateSet,
) -> ConnectivityPlan {
    let both_have_reflexive = local.stun_reflexive_candidate.is_some() && remote.stun_reflexive_candidate.is_some();
    let any_symmetric = matches!(local_nat, NatType::Symmetric) || matches!(remote_nat, NatType::Symmetric);

    if any_symmetric {
        if local.relay_candidate.is_some() || remote.relay_candidate.is_some() {
            return ConnectivityPlan {
                route: Route::Relay,
                reason: "symmetric NAT detected; using relay",
            };
        }

        return ConnectivityPlan {
            route: Route::Direct,
            reason: "symmetric NAT detected but relay unavailable; try direct best-effort",
        };
    }

    if both_have_reflexive {
        return ConnectivityPlan {
            route: Route::Direct,
            reason: "both peers have reflexive candidates",
        };
    }

    if local.relay_candidate.is_some() || remote.relay_candidate.is_some() {
        return ConnectivityPlan {
            route: Route::Relay,
            reason: "insufficient direct candidates; fallback to relay",
        };
    }

    ConnectivityPlan {
        route: Route::Direct,
        reason: "default direct route",
    }
}

pub fn should_attempt_hole_punch(local_nat: NatType, remote_nat: NatType) -> bool {
    !matches!(local_nat, NatType::Symmetric) && !matches!(remote_nat, NatType::Symmetric)
}
