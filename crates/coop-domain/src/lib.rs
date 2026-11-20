//! Coop **domain** layer — pure model, no I/O.
//!
//! - [`rbac`] — Role / Scope / RbacLevel / Principal
//! - [`a2a`] — Task, Message, negotiation & supplier params

pub mod a2a;
pub mod rbac;

pub use a2a::*;
pub use rbac::*;
