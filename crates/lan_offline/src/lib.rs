use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanPolicy {
    pub allow_loopback: bool,
    pub allow_link_local: bool,
    pub allow_private: bool,
    pub deny_public: bool,
}

impl Default for LanPolicy {
    fn default() -> Self {
        Self {
            allow_loopback: true,
            allow_link_local: true,
            allow_private: true,
            deny_public: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(&'static str),
}

#[derive(Debug, Clone)]
pub struct LanOfflineGuard {
    policy: LanPolicy,
    mode_enabled: bool,
}

impl LanOfflineGuard {
    pub fn new(policy: LanPolicy) -> Self {
        Self {
            policy,
            mode_enabled: true,
        }
    }

    pub fn enable_offline_mode(&mut self) {
        self.mode_enabled = true;
    }

    pub fn disable_offline_mode(&mut self) {
        self.mode_enabled = false;
    }

    pub fn is_offline_mode_enabled(&self) -> bool {
        self.mode_enabled
    }

    /// Validate whether a peer address can be used while in offline LAN mode.
    pub fn evaluate_peer(&self, addr: SocketAddr) -> PolicyDecision {
        if !self.mode_enabled {
            return PolicyDecision::Allow;
        }

        let ip = addr.ip();

        if ip.is_loopback() {
            return if self.policy.allow_loopback {
                PolicyDecision::Allow
            } else {
                PolicyDecision::Deny("loopback denied")
            };
        }

        if is_link_local(ip) {
            return if self.policy.allow_link_local {
                PolicyDecision::Allow
            } else {
                PolicyDecision::Deny("link-local denied")
            };
        }

        if is_private(ip) {
            return if self.policy.allow_private {
                PolicyDecision::Allow
            } else {
                PolicyDecision::Deny("private-range denied")
            };
        }

        if self.policy.deny_public {
            return PolicyDecision::Deny("public internet address denied in offline mode");
        }

        PolicyDecision::Allow
    }

    /// Returns true only when all peers satisfy offline-LAN policy.
    pub fn validate_peer_set<'a>(&self, peers: impl IntoIterator<Item = &'a SocketAddr>) -> Result<(), LanOfflineError> {
        for peer in peers {
            match self.evaluate_peer(*peer) {
                PolicyDecision::Allow => {}
                PolicyDecision::Deny(reason) => {
                    return Err(LanOfflineError::PeerDenied {
                        peer: *peer,
                        reason,
                    })
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanOfflineError {
    PeerDenied { peer: SocketAddr, reason: &'static str },
}

impl std::fmt::Display for LanOfflineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanOfflineError::PeerDenied { peer, reason } => {
                write!(f, "peer {peer} denied: {reason}")
            }
        }
    }
}

impl std::error::Error for LanOfflineError {}

fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 10
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192 && o[1] == 168)
        }
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            (seg[0] & 0xfe00) == 0xfc00 // fc00::/7 unique local addresses
        }
    }
}

fn is_link_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 169 && o[1] == 254
        }
        IpAddr::V6(v6) => {
            let seg0 = v6.segments()[0];
            (seg0 & 0xffc0) == 0xfe80 // fe80::/10
        }
    }
}
