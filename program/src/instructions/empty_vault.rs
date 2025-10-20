use crate::{
    accounts::vault::Vault,
    instructions::VaultProgramInstructions,
    utils::{
        load_account, load_ix_data, load_signer, load_system_program, load_token_program, DataLen,
        Discriminator,
    },
    vault_seed_with_bump,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_log::log;
use pinocchio_token::state::{Mint, TokenAccount};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyVaultIxData {
    pub discriminator: u8,
}

impl Default for EmptyVaultIxData {
    fn default() -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
        }
    }
}

impl EmptyVaultIxData {
    pub fn new() -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
        }
    }

    /// # Safety
    /// C style cast into bytes
    pub unsafe fn to_bytes(&self) -> &[u8] {
        unsafe { crate::utils::to_bytes::<Self>(self) }
    }
}

impl DataLen for EmptyVaultIxData {
    const LEN: usize = core::mem::size_of::<EmptyVaultIxData>();
}

impl Discriminator for EmptyVaultIxData {
    const DISCRIMINATOR: u8 = VaultProgramInstructions::EmptyVault as u8;
}

pub fn process_empty_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [vault, admin, mint, admin_token, vault_token, token_program, system_program] = accounts
    else {
        log!("Not enough keys, need 7, got {}", accounts.len());
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let _ = unsafe { load_ix_data::<EmptyVaultIxData>(data)? };

    // ----------------------- CHECKS -----------------------
    load_token_program(token_program)?;
    load_system_program(system_program)?;
    load_signer(admin, true)?;

    // Load and validate the mint account
    {
        let _ = Mint::from_account_info(mint)?;
    }

    // Load and validate the admin token account
    {
        let admin_token_account = TokenAccount::from_account_info(admin_token)?;
        if admin_token_account.mint().ne(mint.key()) {
            log!(
                "Admin token account does not match mint {} != {}",
                admin_token_account.mint(),
                mint.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        if admin_token_account.owner().ne(admin.key()) {
            log!(
                "Admin is not the owner of the admin token account {} != {}",
                admin_token_account.owner(),
                admin.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let tokens_to_empty = {
        let vault_token_account = TokenAccount::from_account_info(vault_token)?;
        if vault_token_account.owner().ne(vault.key()) {
            log!(
                "Vault is not the owner of the vault token account {} != {}",
                vault_token_account.owner(),
                vault.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_token_account.mint().ne(mint.key()) {
            log!(
                "Vault token account does not match mint {} != {}",
                vault_token_account.mint(),
                mint.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        vault_token_account.amount()
    };

    // Vault Checks
    Vault::check(
        program_id,
        vault,
        true,
        Some(admin),
        Some(mint),
        Some(vault_token),
    )?;
    unsafe {
        Vault::check_unlock_okay(vault)?;
    }

    let vault_account = unsafe {
        let data = vault.borrow_data_unchecked();
        load_account::<Vault>(data)?
    };

    // ----------------------- Get Signer Seeds -----------------------
    let bump_bytes = [vault_account.bump()];
    let seed_with_bump = vault_seed_with_bump!(admin.key(), mint.key(), &bump_bytes);
    let signing_seeds = [
        Seed::from(seed_with_bump[0]),
        Seed::from(seed_with_bump[1]),
        Seed::from(seed_with_bump[2]),
        Seed::from(seed_with_bump[3]),
    ];
    Vault::check_seeds(
        admin.key(),
        mint.key(),
        vault_account.bump(),
        &signing_seeds,
    )?;
    let signer = Signer::from(&signing_seeds);

    // ----------------------- Transfer Tokens -----------------------

    pinocchio_token::instructions::Transfer {
        from: vault_token,
        to: admin_token,
        authority: vault,
        amount: tokens_to_empty,
    }
    .invoke_signed(std::slice::from_ref(&signer))?;

    // ----------------------- Close Vault Token Account -----------------------

    pinocchio_token::instructions::CloseAccount {
        account: vault_token,
        destination: admin_token,
        authority: vault,
    }
    .invoke_signed(std::slice::from_ref(&signer))?;

    // ----------------------- Close Vault -----------------------
    unsafe {
        // Transfer all lamports from vault to admin
        *admin.borrow_mut_lamports_unchecked() = admin.lamports().saturating_add(vault.lamports());
        *vault.borrow_mut_lamports_unchecked() = 0;

        // Zero out the vault data to mark it as closed
        vault.borrow_mut_data_unchecked().fill(0);
    }

    // ----------------------- Info -----------------------

    log!(
        "Vault emptied {} tokens ( {} ) to {}",
        tokens_to_empty,
        mint.key(),
        admin.key()
    );

    Ok(())
}
