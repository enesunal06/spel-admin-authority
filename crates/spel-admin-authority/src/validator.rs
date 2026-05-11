use crate::AdminKey;

/// Authority classification returned by a validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum AuthorityKind {
    /// A runtime-authorized account that represents a normal signer.
    Signer,
    /// A PDA accepted by a SPEL/LEE validator.
    ///
    /// This does not invent PDA signing semantics. Programs should only return
    /// this kind after official SPEL/LEE PDA validation proves the account is a
    /// valid, initialized PDA under the program's authority policy.
    ProgramDerivedAddress,
}

/// Validates whether a candidate key can become admin.
///
/// The crate does not implement manual on-curve checks. Signer validation should
/// use runtime authorization metadata, and PDA acceptance should use official
/// SPEL/LEE PDA validation before returning `Some(...)`.
pub trait AdminValidator {
    /// Returns the candidate kind when valid.
    fn classify(&self, key: &AdminKey) -> Option<AuthorityKind>;

    /// Returns whether the candidate can be stored as admin.
    fn is_valid(&self, key: &AdminKey) -> bool {
        self.classify(key).is_some()
    }
}

/// Permissive validator for examples and tests only.
///
/// This accepts any non-zero key. Do not use it as production policy.
#[derive(Clone, Copy, Debug, Default)]
pub struct NonZeroAdminValidator;

impl AdminValidator for NonZeroAdminValidator {
    fn classify(&self, key: &AdminKey) -> Option<AuthorityKind> {
        if key.is_zero() {
            None
        } else {
            Some(AuthorityKind::Signer)
        }
    }
}

/// Deterministic allow-list validator for tests.
#[derive(Clone, Copy, Debug)]
pub struct StaticAdminValidator<'a> {
    signers: &'a [AdminKey],
    pdas: &'a [AdminKey],
}

impl<'a> StaticAdminValidator<'a> {
    /// Creates a validator from accepted signer keys and PDA keys.
    pub const fn new(signers: &'a [AdminKey], pdas: &'a [AdminKey]) -> Self {
        Self { signers, pdas }
    }
}

impl AdminValidator for StaticAdminValidator<'_> {
    fn classify(&self, key: &AdminKey) -> Option<AuthorityKind> {
        if key.is_zero() {
            return None;
        }

        if self.signers.contains(key) {
            return Some(AuthorityKind::Signer);
        }

        if self.pdas.contains(key) {
            return Some(AuthorityKind::ProgramDerivedAddress);
        }

        None
    }
}
