/// A 32-byte authority identifier.
///
/// In SPEL this maps to the raw bytes of an `AccountId`, but the core crate
/// keeps the type runtime-neutral.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub struct AdminKey([u8; 32]);

impl AdminKey {
    /// The zero key. It is not accepted as a new admin authority.
    pub const ZERO: Self = Self([0; 32]);

    /// Creates a key from raw bytes.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns raw bytes.
    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }

    /// Borrows raw bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns `true` when the key is all zeroes.
    pub fn is_zero(&self) -> bool {
        self.0 == [0; 32]
    }
}

/// Alias for callers that think in signer terms.
pub type SignerKey = AdminKey;
