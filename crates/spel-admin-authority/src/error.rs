/// Admin authority operation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminAuthorityError {
    /// Admin was already initialized.
    AdminAlreadyInitialized,
    /// Admin has not been initialized.
    AdminNotInitialized,
    /// Admin was renounced and cannot be restored.
    AdminAlreadyRenounced,
    /// The supplied account is not the current admin.
    Unauthorized,
    /// The current admin account was not runtime-authorized.
    MissingRequiredAuthorization,
    /// New admin failed validation.
    InvalidNewAdmin,
}

/// Result alias for admin authority operations.
pub type Result<T> = core::result::Result<T, AdminAuthorityError>;
