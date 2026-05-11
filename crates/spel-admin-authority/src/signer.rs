use crate::AdminKey;

/// Runtime authorization metadata for an authority account.
///
/// SPEL programs should build this from the account ID and
/// `AccountWithMetadata::is_authorized` after using `#[account(signer)]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct AuthoritySigner {
    key: AdminKey,
    is_authorized: bool,
}

impl AuthoritySigner {
    /// Creates metadata from a key and runtime authorization bit.
    pub const fn new(key: AdminKey, is_authorized: bool) -> Self {
        Self { key, is_authorized }
    }

    /// Creates metadata for an authorized signer.
    pub const fn authorized(key: AdminKey) -> Self {
        Self::new(key, true)
    }

    /// Creates metadata for an unauthorized signer.
    pub const fn unauthorized(key: AdminKey) -> Self {
        Self::new(key, false)
    }

    /// Alias for [`Self::authorized`].
    pub const fn signed(key: AdminKey) -> Self {
        Self::authorized(key)
    }

    /// Alias for [`Self::unauthorized`].
    pub const fn unsigned(key: AdminKey) -> Self {
        Self::unauthorized(key)
    }

    /// Returns the key.
    pub const fn key(self) -> AdminKey {
        self.key
    }

    /// Returns whether the runtime marked this account as authorized.
    pub const fn is_authorized(self) -> bool {
        self.is_authorized
    }

    /// Alias for [`Self::is_authorized`].
    pub const fn is_signer(self) -> bool {
        self.is_authorized
    }
}
