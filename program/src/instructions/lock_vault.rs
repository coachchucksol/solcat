use crate::{
    accounts::vault::Vault,
    utils::{
        load_ix_data, load_signer, load_system_account, load_system_program, load_token_program,
        DataLen, Discriminator,
    },
    vault_seed_with_bump,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::state::{Mint, TokenAccount};

use super::VaultProgramInstructions;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LockVaultIxData {
    pub discriminator: u8,
    pub vault_bump: u8,
    pub slots_to_lock: u64,
    pub tokens_to_lock: Option<u64>,
}

impl LockVaultIxData {
    pub fn new(vault_bump: u8, slots_to_lock: u64, tokens_to_lock: Option<u64>) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            vault_bump,
            slots_to_lock,
            tokens_to_lock,
        }
    }

    pub unsafe fn to_bytes(&self) -> &[u8] {
        unsafe { crate::utils::to_bytes::<Self>(&self) }
    }
}

impl DataLen for LockVaultIxData {
    const LEN: usize = core::mem::size_of::<LockVaultIxData>();
}

impl Discriminator for LockVaultIxData {
    const DISCRIMINATOR: u8 = VaultProgramInstructions::LockVault as u8;
}

pub fn process_lock_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [vault, admin, mint, admin_token, vault_token, token_program, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let ix_data = unsafe { load_ix_data::<LockVaultIxData>(data)? };

    // ----------------------- CHECKS -----------------------
    load_token_program(token_program)?;
    load_system_program(system_program)?;
    load_system_account(vault, true)?;
    load_signer(admin, true)?;

    // Check PDA is correct
    let pda: Pubkey =
        Vault::create_program_address(program_id, admin.key(), mint.key(), ix_data.vault_bump)?;
    if vault.key().ne(&pda) {
        log!(
            "Vault account has an invalid key {} != {}",
            vault.key(),
            &pda
        );
        return Err(ProgramError::InvalidAccountData);
    };

    // Load and validate the mint account
    let mint_decimals = {
        let mint = Mint::from_account_info(mint)?;
        mint.decimals()
    };

    // Load and validate the admin token account
    {
        let vault_token_account = TokenAccount::from_account_info(vault_token)?;
        if vault_token_account.owner().ne(vault.key()) {
            log!(
                "Admin is not the owner of the vault token account {} != {}",
                vault_token_account.owner(),
                vault.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_token_account.mint().ne(mint.key()) {
            log!(
                "Mint does not match the vault token account {} != {}",
                vault_token_account.mint(),
                mint.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let all_tokens = {
        let admin_token_account = TokenAccount::from_account_info(admin_token)?;
        if admin_token_account.owner().ne(admin.key()) {
            log!(
                "Admin is not the owner of the admin token account {} != {}",
                admin_token_account.owner(),
                admin.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }
        if admin_token_account.mint().ne(mint.key()) {
            log!(
                "Mint does not match the admin token account {} != {}",
                admin_token_account.mint(),
                mint.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }

        admin_token_account.amount()
    };

    let tokens_to_lock = ix_data.tokens_to_lock.unwrap_or_else(|| all_tokens);

    if tokens_to_lock > all_tokens {
        log!(
            "Tokens to lock exceed the available tokens {} > {}",
            tokens_to_lock,
            all_tokens
        );
        return Err(ProgramError::InvalidArgument);
    }

    // ----------------------- Create Vault -----------------------

    let rent = Rent::get()?;

    let bump_bytes = [ix_data.vault_bump];
    let seed_with_bump = vault_seed_with_bump!(admin.key(), mint.key(), &bump_bytes);
    let signing_seeds = [
        Seed::from(seed_with_bump[0]),
        Seed::from(seed_with_bump[1]),
        Seed::from(seed_with_bump[2]),
        Seed::from(seed_with_bump[3]),
    ];

    Vault::check_seeds(admin.key(), mint.key(), ix_data.vault_bump, &signing_seeds)?;

    let signer = Signer::from(&signing_seeds);

    CreateAccount {
        from: admin,
        to: vault,
        space: Vault::LEN as u64,
        owner: &program_id,
        lamports: rent.minimum_balance(Vault::LEN),
    }
    .invoke_signed(&[signer.clone()])?;

    unsafe {
        Vault::initialize(
            vault,
            admin.key(),
            mint.key(),
            &ix_data,
            mint_decimals,
            tokens_to_lock,
        )?;
    }

    // ----------------------- Transfer Tokens -----------------------
    pinocchio_token::instructions::Transfer {
        from: admin_token,
        to: vault_token,
        authority: admin,
        amount: tokens_to_lock,
    }
    .invoke()?;

    // ----------------------- Info -----------------------
    log!(
        "Vault locked with {} tokens ( {} ), for {} slots",
        tokens_to_lock,
        mint.key(),
        ix_data.slots_to_lock
    );

    Ok(())
}
