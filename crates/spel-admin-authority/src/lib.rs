#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Admin authority primitives for SPEL/LEE programs.
//!
//! The core is intentionally framework-agnostic. It stores one active admin,
//! checks runtime authorization metadata, supports transfer, and treats renounce
//! as irreversible.
//!
//! SPEL integration should use existing account annotations such as
//! `#[account(signer)]` and call [`assert_admin`] explicitly inside privileged
//! handlers. A real `#[admin_gated]` annotation is left to official SPEL macro
//! support rather than a fragile standalone macro.

mod config;
mod error;
mod key;
mod signer;
mod state;
mod validator;

pub use config::{update_admin_config, AdminConfig, GatedConfig};
pub use error::{AdminAuthorityError, Result};
pub use key::{AdminKey, SignerKey};
pub use signer::AuthoritySigner;
pub use state::{assert_admin, initialize_admin, renounce_admin, transfer_admin, AdminAuthority};
pub use validator::{AdminValidator, AuthorityKind, NonZeroAdminValidator, StaticAdminValidator};
