//! Identity-based agent trust score (no behavioral reputation).

use serde::{Deserialize, Serialize};

use crate::rbac::{Principal, RbacLevel, Scope};

/// Discrete trust band derived from caller identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustScore {
    Untrusted = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

impl TrustScore {
    pub fn as_str(self) -> &'static str {
        match self {
            TrustScore::Untrusted => "untrusted",
            TrustScore::Low => "low",
            TrustScore::Medium => "medium",
            TrustScore::High => "high",
        }
    }
}

impl std::fmt::Display for TrustScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How the caller proved identity (strength ladder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityProof {
    /// Shared API key / demo Bearer (weakest).
    SharedSecret,
    OAuth2,
    Mtls,
}

impl Scope {
    /// Minimum trust band required for this skill.
    pub fn min_trust(self) -> TrustScore {
        match self {
            Scope::SupplierRead => TrustScore::Low,
            Scope::Negotiate | Scope::Investigate => TrustScore::Medium,
        }
    }
}

impl Principal {
    /// Trust from identity: role clearance + proof strength.
    pub fn trust_score(&self, proof: IdentityProof) -> TrustScore {
        let Some(level) = self.roles.iter().map(|r| r.max_level()).max() else {
            return TrustScore::Untrusted;
        };
        let from_role = match level {
            RbacLevel::Read => TrustScore::Low,
            RbacLevel::Consult | RbacLevel::Act => TrustScore::Medium,
            RbacLevel::Admin => TrustScore::High,
        };
        match proof {
            IdentityProof::SharedSecret => from_role,
            IdentityProof::OAuth2 => from_role.max(TrustScore::Medium),
            IdentityProof::Mtls => TrustScore::High,
        }
    }

    /// RBAC scope check + trust floor for the skill.
    pub fn authorize_with_trust(
        &self,
        skill: &str,
        proof: IdentityProof,
    ) -> anyhow::Result<TrustScore> {
        self.authorize(skill)?;
        let scope = Scope::from_skill(skill)?;
        let trust = self.trust_score(proof);
        if trust < scope.min_trust() {
            anyhow::bail!(
                "principal {} trust {trust} below {:?} required for skill `{skill}`",
                self.agent_id,
                scope.min_trust()
            );
        }
        Ok(trust)
    }
}
