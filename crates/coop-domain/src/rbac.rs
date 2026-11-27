//! Generic RBAC — clear roles + levels; scopes are independent permissions.
//!
//! Defined here in the **domain** layer (`coop-domain::rbac`):
//! - [`Role`] — who you are (generic, product-agnostic)
//! - [`RbacLevel`] — how strong the grant is
//! - [`Scope`] — what skill/permission (A2A skill ids)
//!
//! AuthZ rule: principal may invoke a skill iff their highest role
//! [`Role::max_level`] is ≥ that skill's [`Scope::required_level`].

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Permission / A2A skill. Add variants without changing [`Role`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Negotiate,
    #[serde(rename = "supplier.read")]
    SupplierRead,
    Investigate,
    /// Generic hub → agent message (any registered adapter).
    #[serde(rename = "peer.message")]
    PeerMessage,
}

impl Scope {
    pub fn skill_id(self) -> &'static str {
        match self {
            Scope::Negotiate => "negotiate",
            Scope::SupplierRead => "supplier.read",
            Scope::Investigate => "investigate",
            Scope::PeerMessage => "peer.message",
        }
    }

    pub fn required_level(self) -> RbacLevel {
        match self {
            Scope::SupplierRead => RbacLevel::Read,
            Scope::Negotiate | Scope::Investigate | Scope::PeerMessage => RbacLevel::Act,
        }
    }

    pub fn from_skill(skill: &str) -> Result<Self> {
        match skill {
            "negotiate" => Ok(Scope::Negotiate),
            "supplier.read" => Ok(Scope::SupplierRead),
            "investigate" => Ok(Scope::Investigate),
            "peer.message" => Ok(Scope::PeerMessage),
            other => bail!("unknown skill `{other}`"),
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.skill_id())
    }
}

/// Clearance ladder (generic): Read < Consult < Act < Admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RbacLevel {
    Read = 1,
    Consult = 2,
    Act = 3,
    Admin = 4,
}

/// Generic RBAC roles — not tied to negotiation/supplier product names.
///
/// | Role | max_level | Typical use |
/// |------|-----------|-------------|
/// | `Reader` | Read | read-only callers |
/// | `Contributor` | Consult | enrich / advise |
/// | `Operator` | Act | run skills that change workflow state |
/// | `Admin` | Admin | full authority |
/// | `Service` | Admin | gateway / machine identity |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Reader,
    Contributor,
    Operator,
    Admin,
    Service,
}

impl Role {
    /// Ceiling level this role confers on **any** scope.
    pub fn max_level(self) -> RbacLevel {
        match self {
            Role::Reader => RbacLevel::Read,
            Role::Contributor => RbacLevel::Consult,
            Role::Operator => RbacLevel::Act,
            Role::Admin | Role::Service => RbacLevel::Admin,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub agent_id: String,
    pub tenant_id: String,
    pub roles: Vec<Role>,
}

impl Principal {
    pub fn level_for(&self, _scope: Scope) -> Option<RbacLevel> {
        self.roles.iter().map(|r| r.max_level()).max()
    }

    pub fn can_invoke(&self, scope: Scope) -> bool {
        self.level_for(scope)
            .is_some_and(|lvl| lvl >= scope.required_level())
    }

    pub fn authorize(&self, skill: &str) -> Result<()> {
        let scope = Scope::from_skill(skill)?;
        if self.can_invoke(scope) {
            Ok(())
        } else {
            bail!(
                "principal {} (roles {:?}) lacks {scope} at RBAC >= {:?} for skill `{skill}`",
                self.agent_id,
                self.roles,
                scope.required_level()
            )
        }
    }
}
