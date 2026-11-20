//! Coop **domain** layer — pure model, no I/O.
//!
//! - [`rbac`] — Role / Scope / RbacLevel / Principal
//! - [`trust`] — TrustScore from identity
//! - [`a2a`] — Task, Message, negotiation & supplier params

pub mod a2a;
pub mod rbac;
pub mod trust;

pub use a2a::*;
pub use rbac::*;
pub use trust::{IdentityProof, TrustScore};
