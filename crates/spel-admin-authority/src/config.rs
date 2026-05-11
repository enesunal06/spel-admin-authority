use crate::{assert_admin, AdminAuthority, AdminKey, AuthoritySigner, Result};

/// Tiny config account used to demonstrate gated writes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct AdminConfig {
    value: u64,
    last_updated_by: Option<AdminKey>,
}

impl AdminConfig {
    /// Creates empty config.
    pub const fn new() -> Self {
        Self {
            value: 0,
            last_updated_by: None,
        }
    }

    /// Returns the stored value.
    pub const fn value(&self) -> u64 {
        self.value
    }

    /// Returns the last updating admin.
    pub const fn last_updated_by(&self) -> Option<AdminKey> {
        self.last_updated_by
    }

    /// Updates config after an admin check.
    pub fn update(
        &mut self,
        admin_state: &AdminAuthority,
        authority: AuthoritySigner,
        value: u64,
    ) -> Result<()> {
        update_admin_config(admin_state, self, authority, value)
    }
}

/// Alias kept for earlier examples.
pub type GatedConfig = AdminConfig;

/// Updates sample config after checking admin authority.
pub fn update_admin_config(
    admin_state: &AdminAuthority,
    config: &mut AdminConfig,
    authority: AuthoritySigner,
    value: u64,
) -> Result<()> {
    assert_admin(admin_state, authority)?;
    config.value = value;
    config.last_updated_by = Some(authority.key());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        update_admin_config, AdminAuthority, AdminAuthorityError, AdminConfig, AdminKey,
        AuthoritySigner, StaticAdminValidator,
    };

    const ADMIN: AdminKey = AdminKey::new([1; 32]);
    const USER: AdminKey = AdminKey::new([3; 32]);

    fn validator() -> StaticAdminValidator<'static> {
        StaticAdminValidator::new(&[ADMIN], &[])
    }

    #[test]
    fn update_admin_config_is_gated() {
        let state = AdminAuthority::initialized(ADMIN, &validator()).unwrap();
        let mut config = AdminConfig::new();

        update_admin_config(&state, &mut config, AuthoritySigner::authorized(ADMIN), 42).unwrap();

        assert_eq!(config.value(), 42);
        assert_eq!(config.last_updated_by(), Some(ADMIN));
        assert_eq!(
            update_admin_config(&state, &mut config, AuthoritySigner::authorized(USER), 7),
            Err(AdminAuthorityError::Unauthorized)
        );
        assert_eq!(config.value(), 42);
    }

    #[test]
    fn renounced_state_rejects_config_update() {
        let state = AdminAuthority::renounced();
        let mut config = AdminConfig::new();

        assert_eq!(
            update_admin_config(&state, &mut config, AuthoritySigner::authorized(ADMIN), 9),
            Err(AdminAuthorityError::AdminAlreadyRenounced)
        );
    }
}
