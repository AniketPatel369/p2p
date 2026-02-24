use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub version: String,
    pub channel: UpdateChannel,
    pub platform: String,
    pub package_url: String,
    pub sha256: String,
    pub rollback_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPolicy {
    pub allow_channel_upgrade: bool,
    pub allow_downgrade: bool,
    pub require_https: bool,
    pub allowed_platforms: HashSet<String>,
}

impl Default for InstallPolicy {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert("windows-x86_64".to_string());
        allowed.insert("macos-aarch64".to_string());
        allowed.insert("linux-x86_64".to_string());

        Self {
            allow_channel_upgrade: true,
            allow_downgrade: false,
            require_https: true,
            allowed_platforms: allowed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateDecision {
    pub allowed: bool,
    pub reason: &'static str,
}

pub fn validate_manifest(manifest: &PackageManifest, policy: &InstallPolicy) -> Result<(), InstallerError> {
    if manifest.version.trim().is_empty() {
        return Err(InstallerError::InvalidManifest("version is empty"));
    }
    if manifest.sha256.len() != 64 || !manifest.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(InstallerError::InvalidManifest("sha256 must be 64 hex chars"));
    }
    if policy.require_https && !manifest.package_url.starts_with("https://") {
        return Err(InstallerError::PolicyViolation("non-https package url denied"));
    }
    if !policy.allowed_platforms.contains(&manifest.platform) {
        return Err(InstallerError::PolicyViolation("platform not allowed"));
    }
    Ok(())
}

pub fn evaluate_update(
    current_version: &str,
    current_channel: UpdateChannel,
    candidate: &PackageManifest,
    policy: &InstallPolicy,
) -> Result<UpdateDecision, InstallerError> {
    validate_manifest(candidate, policy)?;

    let version_cmp = compare_semver(&candidate.version, current_version)?;
    if version_cmp < 0 && !policy.allow_downgrade {
        return Ok(UpdateDecision {
            allowed: false,
            reason: "downgrade blocked by policy",
        });
    }

    if !policy.allow_channel_upgrade && channel_rank(candidate.channel) > channel_rank(current_channel) {
        return Ok(UpdateDecision {
            allowed: false,
            reason: "channel upgrade blocked by policy",
        });
    }

    if version_cmp == 0 {
        return Ok(UpdateDecision {
            allowed: false,
            reason: "already on same version",
        });
    }

    Ok(UpdateDecision {
        allowed: true,
        reason: "update accepted",
    })
}

pub fn rollback_marker(previous_version: &str, failed_version: &str) -> String {
    format!("rollback:{}<-{}", previous_version, failed_version)
}

fn channel_rank(channel: UpdateChannel) -> u8 {
    match channel {
        UpdateChannel::Stable => 0,
        UpdateChannel::Beta => 1,
        UpdateChannel::Nightly => 2,
    }
}

fn compare_semver(a: &str, b: &str) -> Result<i8, InstallerError> {
    let pa = parse_semver(a)?;
    let pb = parse_semver(b)?;
    Ok(if pa > pb {
        1
    } else if pa < pb {
        -1
    } else {
        0
    })
}

fn parse_semver(v: &str) -> Result<(u64, u64, u64), InstallerError> {
    let mut parts = v.split('.');
    let major = parts
        .next()
        .ok_or(InstallerError::InvalidManifest("invalid semver"))?
        .parse::<u64>()
        .map_err(|_| InstallerError::InvalidManifest("invalid semver"))?;
    let minor = parts
        .next()
        .ok_or(InstallerError::InvalidManifest("invalid semver"))?
        .parse::<u64>()
        .map_err(|_| InstallerError::InvalidManifest("invalid semver"))?;
    let patch = parts
        .next()
        .ok_or(InstallerError::InvalidManifest("invalid semver"))?
        .parse::<u64>()
        .map_err(|_| InstallerError::InvalidManifest("invalid semver"))?;

    if parts.next().is_some() {
        return Err(InstallerError::InvalidManifest("invalid semver"));
    }

    Ok((major, minor, patch))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerError {
    InvalidManifest(&'static str),
    PolicyViolation(&'static str),
}

impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallerError::InvalidManifest(m) => write!(f, "invalid manifest: {m}"),
            InstallerError::PolicyViolation(m) => write!(f, "policy violation: {m}"),
        }
    }
}

impl std::error::Error for InstallerError {}
