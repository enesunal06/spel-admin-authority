use crate::{AdminAuthorityError, AdminKey, AdminValidator, AuthoritySigner, Result};

/// Stores the active admin authority.
///
/// Renounce is intentionally irreversible. Once `renounced` is true, no future
/// initialize, transfer, renounce, or privileged check can succeed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct AdminAuthority {
    admin: Option<AdminKey>,
    renounced: bool,
}

impl AdminAuthority {
    /// Creates uninitialized state.
    pub const fn empty() -> Self {
        Self {
            admin: None,
            renounced: false,
        }
    }

    /// Creates already-renounced state.
    pub const fn renounced() -> Self {
        Self {
            admin: None,
            renounced: true,
        }
    }

    /// Creates initialized state.
    pub fn initialized(admin: AdminKey, validator: &impl AdminValidator) -> Result<Self> {
        let mut state = Self::empty();
        initialize_admin(&mut state, admin, validator)?;
        Ok(state)
    }

    /// Builds state from raw parts.
    pub const fn from_parts(admin: Option<AdminKey>, renounced: bool) -> Self {
        Self { admin, renounced }
    }

    /// Returns the active admin, if set.
    pub const fn admin(&self) -> Option<AdminKey> {
        self.admin
    }

    /// Returns whether an admin is set.
    pub const fn has_admin(&self) -> bool {
        self.admin.is_some()
    }

    /// Returns whether admin was irreversibly renounced.
    pub const fn is_renounced(&self) -> bool {
        self.renounced
    }

    /// Initializes admin exactly once.
    pub fn initialize(&mut self, admin: AdminKey, validator: &impl AdminValidator) -> Result<()> {
        initialize_admin(self, admin, validator)
    }

    /// Checks that the supplied runtime account is the active admin.
    pub fn assert_admin(&self, authority: AuthoritySigner) -> Result<()> {
        assert_admin(self, authority)
    }

    /// Transfers admin to a validated new key.
    pub fn transfer(
        &mut self,
        current_admin: AuthoritySigner,
        new_admin: AdminKey,
        validator: &impl AdminValidator,
    ) -> Result<()> {
        transfer_admin(self, current_admin, new_admin, validator)
    }

    /// Irreversibly renounces admin.
    pub fn renounce(&mut self, current_admin: AuthoritySigner) -> Result<()> {
        renounce_admin(self, current_admin)
    }
}

/// Initializes admin exactly once.
pub fn initialize_admin(
    state: &mut AdminAuthority,
    admin: AdminKey,
    validator: &impl AdminValidator,
) -> Result<()> {
    if state.renounced {
        return Err(AdminAuthorityError::AdminAlreadyRenounced);
    }

    if state.admin.is_some() {
        return Err(AdminAuthorityError::AdminAlreadyInitialized);
    }

    ensure_valid_new_admin(admin, validator)?;
    state.admin = Some(admin);
    Ok(())
}

/// Checks that the supplied runtime account is the active admin.
pub fn assert_admin(state: &AdminAuthority, authority: AuthoritySigner) -> Result<()> {
    if state.renounced {
        return Err(AdminAuthorityError::AdminAlreadyRenounced);
    }

    let Some(admin) = state.admin else {
        return Err(AdminAuthorityError::AdminNotInitialized);
    };

    if authority.key() != admin {
        return Err(AdminAuthorityError::Unauthorized);
    }

    if !authority.is_authorized() {
        return Err(AdminAuthorityError::MissingRequiredAuthorization);
    }

    Ok(())
}

/// Transfers admin to a validated new key.
pub fn transfer_admin(
    state: &mut AdminAuthority,
    current_admin: AuthoritySigner,
    new_admin: AdminKey,
    validator: &impl AdminValidator,
) -> Result<()> {
    assert_admin(state, current_admin)?;
    ensure_valid_new_admin(new_admin, validator)?;
    state.admin = Some(new_admin);
    Ok(())
}

/// Irreversibly renounces admin.
pub fn renounce_admin(state: &mut AdminAuthority, current_admin: AuthoritySigner) -> Result<()> {
    assert_admin(state, current_admin)?;
    state.admin = None;
    state.renounced = true;
    Ok(())
}

fn ensure_valid_new_admin(admin: AdminKey, validator: &impl AdminValidator) -> Result<()> {
    if validator.is_valid(&admin) {
        Ok(())
    } else {
        Err(AdminAuthorityError::InvalidNewAdmin)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_admin, initialize_admin, renounce_admin, transfer_admin, AdminAuthority,
        AdminAuthorityError, AdminKey, AuthoritySigner, StaticAdminValidator,
    };

    const ADMIN: AdminKey = AdminKey::new([1; 32]);
    const NEXT_ADMIN: AdminKey = AdminKey::new([2; 32]);
    const USER: AdminKey = AdminKey::new([3; 32]);
    const PDA_ADMIN: AdminKey = AdminKey::new([4; 32]);

    fn validator() -> StaticAdminValidator<'static> {
        StaticAdminValidator::new(&[ADMIN, NEXT_ADMIN], &[PDA_ADMIN])
    }

    #[test]
    fn initialize_admin_sets_one_admin() {
        let mut state = AdminAuthority::empty();

        initialize_admin(&mut state, ADMIN, &validator()).unwrap();

        assert_eq!(state.admin(), Some(ADMIN));
        assert_eq!(
            initialize_admin(&mut state, NEXT_ADMIN, &validator()),
            Err(AdminAuthorityError::AdminAlreadyInitialized)
        );
    }

    #[test]
    fn initialize_admin_rejects_invalid_new_admin() {
        let mut state = AdminAuthority::empty();

        assert_eq!(
            initialize_admin(&mut state, AdminKey::ZERO, &validator()),
            Err(AdminAuthorityError::InvalidNewAdmin)
        );
        assert_eq!(
            initialize_admin(&mut state, USER, &validator()),
            Err(AdminAuthorityError::InvalidNewAdmin)
        );
    }

    #[test]
    fn validator_can_accept_pda_conservatively() {
        let state = AdminAuthority::initialized(PDA_ADMIN, &validator()).unwrap();

        assert_eq!(state.admin(), Some(PDA_ADMIN));
    }

    #[test]
    fn assert_admin_requires_matching_authorized_admin() {
        let state = AdminAuthority::initialized(ADMIN, &validator()).unwrap();

        assert_admin(&state, AuthoritySigner::authorized(ADMIN)).unwrap();
        assert_eq!(
            assert_admin(&state, AuthoritySigner::authorized(USER)),
            Err(AdminAuthorityError::Unauthorized)
        );
        assert_eq!(
            assert_admin(&state, AuthoritySigner::unauthorized(ADMIN)),
            Err(AdminAuthorityError::MissingRequiredAuthorization)
        );
    }

    #[test]
    fn admin_can_transfer_to_valid_new_admin() {
        let mut state = AdminAuthority::initialized(ADMIN, &validator()).unwrap();

        transfer_admin(
            &mut state,
            AuthoritySigner::authorized(ADMIN),
            NEXT_ADMIN,
            &validator(),
        )
        .unwrap();

        assert_eq!(state.admin(), Some(NEXT_ADMIN));
        assert_eq!(
            assert_admin(&state, AuthoritySigner::authorized(ADMIN)),
            Err(AdminAuthorityError::Unauthorized)
        );
        assert_admin(&state, AuthoritySigner::authorized(NEXT_ADMIN)).unwrap();
    }

    #[test]
    fn non_admin_cannot_transfer_or_renounce() {
        let mut state = AdminAuthority::initialized(ADMIN, &validator()).unwrap();

        assert_eq!(
            transfer_admin(
                &mut state,
                AuthoritySigner::authorized(USER),
                NEXT_ADMIN,
                &validator()
            ),
            Err(AdminAuthorityError::Unauthorized)
        );
        assert_eq!(
            renounce_admin(&mut state, AuthoritySigner::authorized(USER)),
            Err(AdminAuthorityError::Unauthorized)
        );
        assert_eq!(state.admin(), Some(ADMIN));
    }

    #[test]
    fn transfer_rejects_invalid_new_admin() {
        let mut state = AdminAuthority::initialized(ADMIN, &validator()).unwrap();

        assert_eq!(
            transfer_admin(
                &mut state,
                AuthoritySigner::authorized(ADMIN),
                USER,
                &validator()
            ),
            Err(AdminAuthorityError::InvalidNewAdmin)
        );
        assert_eq!(state.admin(), Some(ADMIN));
    }

    #[test]
    fn renounce_admin_is_irreversible() {
        let mut state = AdminAuthority::initialized(ADMIN, &validator()).unwrap();

        renounce_admin(&mut state, AuthoritySigner::authorized(ADMIN)).unwrap();

        assert!(state.is_renounced());
        assert_eq!(state.admin(), None);
        assert_eq!(
            assert_admin(&state, AuthoritySigner::authorized(ADMIN)),
            Err(AdminAuthorityError::AdminAlreadyRenounced)
        );
        assert_eq!(
            initialize_admin(&mut state, NEXT_ADMIN, &validator()),
            Err(AdminAuthorityError::AdminAlreadyRenounced)
        );
        assert_eq!(
            transfer_admin(
                &mut state,
                AuthoritySigner::authorized(ADMIN),
                NEXT_ADMIN,
                &validator()
            ),
            Err(AdminAuthorityError::AdminAlreadyRenounced)
        );
    }

    #[test]
    fn uninitialized_state_rejects_privileged_checks() {
        let state = AdminAuthority::empty();

        assert_eq!(
            assert_admin(&state, AuthoritySigner::authorized(ADMIN)),
            Err(AdminAuthorityError::AdminNotInitialized)
        );
    }
}
