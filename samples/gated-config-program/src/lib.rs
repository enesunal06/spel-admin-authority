#![forbid(unsafe_code)]

//! Compiled host-side sample for the admin authority helpers.
//!
//! This crate does not pretend to be the full SPEL macro expansion. It exists
//! to validate the reusable library path in normal Rust tests. The real SPEL
//! source shape is included under `samples/spel-gated-config-program`.

use spel_admin_authority::{
    initialize_admin, renounce_admin, transfer_admin, update_admin_config, AdminAuthority,
    AdminConfig, AdminKey, AdminValidator, AuthoritySigner, Result,
};

/// Program-owned state used by this host-side sample.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProgramState {
    /// Admin authority state account.
    pub admin_authority: AdminAuthority,
    /// Config PDA-like account protected by admin authority.
    pub config: AdminConfig,
}

impl ProgramState {
    /// Initializes the sample program with one admin authority.
    pub fn initialize(admin: AdminKey, validator: &impl AdminValidator) -> Result<Self> {
        let mut state = Self::default();
        initialize_admin(&mut state.admin_authority, admin, validator)?;
        Ok(state)
    }
}

/// Updates config after the caller supplies runtime signer metadata.
pub fn update_config(
    state: &mut ProgramState,
    authority: AuthoritySigner,
    value: u64,
) -> Result<()> {
    update_admin_config(&state.admin_authority, &mut state.config, authority, value)
}

/// Transfers admin authority to a validated new signer or PDA key.
pub fn transfer_program_admin(
    state: &mut ProgramState,
    current_admin: AuthoritySigner,
    new_admin: AdminKey,
    validator: &impl AdminValidator,
) -> Result<()> {
    transfer_admin(
        &mut state.admin_authority,
        current_admin,
        new_admin,
        validator,
    )
}

/// Irreversibly renounces admin authority.
pub fn renounce_program_admin(
    state: &mut ProgramState,
    current_admin: AuthoritySigner,
) -> Result<()> {
    renounce_admin(&mut state.admin_authority, current_admin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spel_admin_authority::{AdminAuthorityError, StaticAdminValidator};

    const ADMIN: AdminKey = AdminKey::new([11; 32]);
    const NEXT_ADMIN: AdminKey = AdminKey::new([12; 32]);
    const USER: AdminKey = AdminKey::new([13; 32]);
    const PDA_ADMIN: AdminKey = AdminKey::new([14; 32]);

    fn validator() -> StaticAdminValidator<'static> {
        StaticAdminValidator::new(&[ADMIN, NEXT_ADMIN], &[PDA_ADMIN])
    }

    #[test]
    fn end_to_end_admin_lifecycle_gates_config_updates() {
        let mut state = ProgramState::initialize(ADMIN, &validator()).unwrap();

        update_config(&mut state, AuthoritySigner::authorized(ADMIN), 100).unwrap();
        assert_eq!(state.config.value(), 100);

        let user_result = update_config(&mut state, AuthoritySigner::authorized(USER), 200);
        assert_eq!(user_result, Err(AdminAuthorityError::Unauthorized));
        assert_eq!(state.config.value(), 100);

        transfer_program_admin(
            &mut state,
            AuthoritySigner::authorized(ADMIN),
            NEXT_ADMIN,
            &validator(),
        )
        .unwrap();

        let old_admin_result = update_config(&mut state, AuthoritySigner::authorized(ADMIN), 300);
        assert_eq!(old_admin_result, Err(AdminAuthorityError::Unauthorized));

        update_config(&mut state, AuthoritySigner::authorized(NEXT_ADMIN), 400).unwrap();
        assert_eq!(state.config.value(), 400);

        renounce_program_admin(&mut state, AuthoritySigner::authorized(NEXT_ADMIN)).unwrap();

        let revoked_result =
            update_config(&mut state, AuthoritySigner::authorized(NEXT_ADMIN), 500);
        assert_eq!(
            revoked_result,
            Err(AdminAuthorityError::AdminAlreadyRenounced)
        );
        assert_eq!(state.config.value(), 400);
    }

    #[test]
    fn transfer_accepts_pda_only_when_validator_allows_it() {
        let mut state = ProgramState::initialize(ADMIN, &validator()).unwrap();

        transfer_program_admin(
            &mut state,
            AuthoritySigner::authorized(ADMIN),
            PDA_ADMIN,
            &validator(),
        )
        .unwrap();

        assert_eq!(state.admin_authority.admin(), Some(PDA_ADMIN));
    }
}
