# SPEL Admin Authority Library

A small access-control library for LEE programs built with SPEL. It gives a program one admin authority, lets that admin transfer control, and lets the admin permanently renounce control.

## Why This Exists

Privileged instructions show up in most programs sooner or later: changing config, rotating authority, pausing features, or handing control to another account. Without a shared pattern, each team ends up writing its own version. This crate keeps the basic admin flow boring, explicit, and testable.

What is implemented:

- initialize one admin authority;
- require runtime authorization before privileged work;
- transfer authority to a validated new admin;
- irreversibly renounce authority;
- demonstrate a gated config PDA update pattern.

What is not claimed: unresolved framework-level behavior such as a one-line SPEL annotation, explicit on-curve validation, or full deployed-PDA admin semantics.

## Workspace

- `crates/spel-admin-authority`: no-std core library with optional Borsh derives.
- `samples/gated-config-program`: compiled Rust harness that imports the library and tests the admin lifecycle.
- `samples/spel-gated-config-program`: reference SPEL source using `#[lez_program]`, `#[instruction]`, `#[account(...)]`, and explicit `assert_admin(...)` calls. It is outside the default workspace because it needs pinned SPEL/LEE Git dependencies and a RISC Zero guest toolchain.

## Dependency

For a path dependency:

```toml
[dependencies]
spel-admin-authority = { path = "../spel-admin-authority/crates/spel-admin-authority" }
```

Enable Borsh derives when storing authority state in SPEL account data:

```toml
[dependencies]
spel-admin-authority = {
  path = "../spel-admin-authority/crates/spel-admin-authority",
  features = ["borsh"]
}
```

The SPEL reference sample pins the framework dependencies it was written against:

```toml
spel-framework = { git = "https://github.com/logos-co/spel.git", rev = "ea2f998ed13c5ad66ef1cafe51de859bef27f872", package = "spel-framework" }
nssa_core = { git = "https://github.com/logos-blockchain/logos-execution-zone.git", tag = "v0.2.0-rc3" }
risc0-zkvm = { version = "=3.0.5", features = ["std"] }
```

Replace those pins with maintainer-approved tags, commits, or local path dependencies when integrating into a real `logos-scaffold` project.

## Core API

```rust
use spel_admin_authority::{
    assert_admin, initialize_admin, renounce_admin, transfer_admin,
    AdminAuthority, AdminKey, AuthoritySigner, NonZeroAdminValidator,
};

let admin = AdminKey::new([1; 32]);
let next_admin = AdminKey::new([2; 32]);
let validator = NonZeroAdminValidator;
let mut state = AdminAuthority::empty();

initialize_admin(&mut state, admin, &validator)?;
assert_admin(&state, AuthoritySigner::authorized(admin))?;
transfer_admin(
    &mut state,
    AuthoritySigner::authorized(admin),
    next_admin,
    &validator,
)?;
renounce_admin(&mut state, AuthoritySigner::authorized(next_admin))?;
```

Renounce is irreversible. After `renounce_admin(...)`, `admin = None`, `renounced = true`, and no further admin operation can succeed.

## SPEL Integration

Use SPEL's existing account annotations to require runtime authorization:

```rust
#[instruction]
pub fn update_config(
    #[account(mut, pda = literal("admin_config"))]
    config_account: AccountWithMetadata,
    #[account(signer)]
    admin: AccountWithMetadata,
    value: u64,
) -> SpelResult {
    let mut program_config = ProgramConfig::decode(&config_account)?;
    let authority = AuthoritySigner::new(
        AdminKey::new(*admin.account_id.value()),
        admin.is_authorized,
    );

    assert_admin(&program_config.admin_authority, authority)?;
    program_config.config.update(
        &program_config.admin_authority,
        authority,
        value,
    )?;

    // serialize config_account and return SpelOutput
}
```

See [`samples/spel-gated-config-program/methods/guest/src/bin/admin_gated_config.rs`](samples/spel-gated-config-program/methods/guest/src/bin/admin_gated_config.rs) for the full reference source.

## Valid New Admins

The core crate exposes `AdminValidator` instead of guessing at runtime cryptography:

```rust
pub trait AdminValidator {
    fn classify(&self, key: &AdminKey) -> Option<AuthorityKind>;
}
```

For now:

- normal signer validity should rely on SPEL runtime authorization metadata, especially `#[account(signer)]` and `AccountWithMetadata::is_authorized`;
- explicit on-curve validation is not implemented until SPEL/LEE exposes an official API;
- PDA admin authority should only be accepted after the program validates the account through canonical SPEL/LEE PDA mechanisms such as `#[account(pda = ...)]` or `compute_pda`;
- `NonZeroAdminValidator` is only a convenience for examples and tests. It accepts any non-zero key and is not production authorization policy.

`StaticAdminValidator` is also test-oriented; use it to model accepted signers or PDAs in unit tests, not to replace runtime account validation.

## Transaction Size Overhead

Each gated instruction needs the admin account in its account list. If the admin account is not already present, the approximate added transaction size is:

- 32 bytes for the account key;
- 1 byte for the instruction account index, ignoring compact-length boundary changes;
- 64 bytes when a newly introduced normal signer signature is required.

Worst case for a new normal admin signer is about 97 bytes. If the admin account and signature are already present for another instruction, the incremental overhead may be only the extra instruction account index.

## Limitations

Single annotation or config flag:

The RFP asks for minimal boilerplate, ideally one annotation. This implementation uses explicit `assert_admin(...)` helper calls because a robust `#[admin_gated]` belongs in the SPEL macro crate or an official SPEL extension point.

On-curve validation:

The library does not perform manual on-curve checks. It treats SPEL/LEE runtime authorization metadata as the source of truth until an official API exists.

Deployed PDA admin authority:

The core can store a PDA key if a validator accepts it, but it does not invent PDA signing semantics. Full deployed-PDA admin behavior requires maintainer clarification on canonical SPEL/LEE PDA authorization.

Localnet e2e:

`logos-scaffold` is currently Unix-oriented. The default CI runs unit, sample, format, and clippy checks. A localnet smoke test should be run manually in a supported Unix/Linux scaffold environment once maintainers confirm the dependency pins and setup flow.

## Tests

Run the default validation suite:

```sh
cargo test --workspace --all-targets
```

Run with optional Borsh derives:

```sh
cargo test --workspace --all-targets --all-features
```

Run clippy:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The tests cover initialization, single-admin semantics, unauthorized access, missing authorization, transfer, irreversible renounce, PDA acceptance through a validator, and gated config updates.
