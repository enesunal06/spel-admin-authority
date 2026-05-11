#![no_main]

use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::account::Data;
use spel_admin_authority::{
    assert_admin, initialize_admin, renounce_admin, transfer_admin, AdminAuthority,
    AdminAuthorityError, AdminConfig, AdminKey, AuthoritySigner, NonZeroAdminValidator,
};
use spel_framework::prelude::*;

#[cfg(not(test))]
risc0_zkvm::guest::entry!(main);

#[account_type]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct ProgramConfig {
    pub admin_authority: AdminAuthority,
    pub config: AdminConfig,
}

impl ProgramConfig {
    fn decode(account: &AccountWithMetadata) -> Result<Self, SpelError> {
        Self::try_from_slice(account.account.data.as_ref())
            .map_err(|err| SpelError::custom(6100, format!("failed to decode config: {err}")))
    }

    fn encode_into(
        self,
        mut account: AccountWithMetadata,
    ) -> Result<AccountWithMetadata, SpelError> {
        let bytes = borsh::to_vec(&self)
            .map_err(|err| SpelError::custom(6101, format!("failed to encode config: {err}")))?;
        account.account.data = Data::try_from(bytes)
            .map_err(|_| SpelError::custom(6102, "encoded config exceeds account data limit"))?;
        Ok(account)
    }
}

fn key_from_account(account: &AccountWithMetadata) -> AdminKey {
    AdminKey::new(*account.account_id.value())
}

fn authority_from_account(account: &AccountWithMetadata) -> AuthoritySigner {
    AuthoritySigner::new(key_from_account(account), account.is_authorized)
}

fn to_spel_error(err: AdminAuthorityError) -> SpelError {
    SpelError::Unauthorized {
        message: format!("admin authority check failed: {err:?}"),
    }
}

#[lez_program]
mod admin_gated_config {
    #[allow(unused_imports)]
    use super::*;

    #[instruction]
    pub fn initialize(
        #[account(init, pda = literal("admin_config"))] config_account: AccountWithMetadata,
        #[account(signer)] admin: AccountWithMetadata,
    ) -> SpelResult {
        let mut program_config = ProgramConfig {
            admin_authority: AdminAuthority::empty(),
            config: AdminConfig::new(),
        };
        let validator = NonZeroAdminValidator;
        initialize_admin(
            &mut program_config.admin_authority,
            key_from_account(&admin),
            &validator,
        )
        .map_err(to_spel_error)?;

        let config_account = program_config.encode_into(config_account)?;

        Ok(SpelOutput::execute(vec![config_account, admin], vec![]))
    }

    #[instruction]
    pub fn update_config(
        #[account(mut, pda = literal("admin_config"))] config_account: AccountWithMetadata,
        #[account(signer)] admin: AccountWithMetadata,
        value: u64,
    ) -> SpelResult {
        let mut program_config = ProgramConfig::decode(&config_account)?;
        let authority = authority_from_account(&admin);
        assert_admin(&program_config.admin_authority, authority).map_err(to_spel_error)?;
        program_config
            .config
            .update(&program_config.admin_authority, authority, value)
            .map_err(to_spel_error)?;

        let config_account = program_config.encode_into(config_account)?;

        Ok(SpelOutput::execute(vec![config_account, admin], vec![]))
    }

    #[instruction]
    pub fn transfer(
        #[account(mut, pda = literal("admin_config"))] config_account: AccountWithMetadata,
        #[account(signer)] current_admin: AccountWithMetadata,
        new_admin: [u8; 32],
    ) -> SpelResult {
        let mut program_config = ProgramConfig::decode(&config_account)?;
        let validator = NonZeroAdminValidator;
        transfer_admin(
            &mut program_config.admin_authority,
            authority_from_account(&current_admin),
            AdminKey::new(new_admin),
            &validator,
        )
        .map_err(to_spel_error)?;

        let config_account = program_config.encode_into(config_account)?;

        Ok(SpelOutput::execute(
            vec![config_account, current_admin],
            vec![],
        ))
    }

    #[instruction]
    pub fn renounce(
        #[account(mut, pda = literal("admin_config"))] config_account: AccountWithMetadata,
        #[account(signer)] current_admin: AccountWithMetadata,
    ) -> SpelResult {
        let mut program_config = ProgramConfig::decode(&config_account)?;
        renounce_admin(
            &mut program_config.admin_authority,
            authority_from_account(&current_admin),
        )
        .map_err(to_spel_error)?;

        let config_account = program_config.encode_into(config_account)?;

        Ok(SpelOutput::execute(
            vec![config_account, current_admin],
            vec![],
        ))
    }
}
