use crate::{
    accounts::vault::Vault,
    pod::{PodOption, PodU64},
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

/// Note that the this struct is also 1-byte aligned using Pods
/// It also has `repr(C, packed)` to make sure no extra bytes are added in
/// padding. This allows us to `C-style` derefrence the account
///
/// Note: I come from a C background, so this is more natural to me
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LockVaultIxData {
    pub discriminator: u8,
    pub vault_bump: u8,
    /// Amount of slots to lock - a Solana epoch has `432_000` slots per epoch
    /// and at the time of writing its about about 2 days per epoch.
    pub slots_to_lock: PodU64,
    /// If this is provided, it will only lock up that amount of tokens, if its `None`
    /// all tokens will be locked
    pub tokens_to_lock: PodOption<PodU64>,
}

impl LockVaultIxData {
    /// We have to do a little casting to setup the IxData, but it ensures everything is gucci
    pub fn new(vault_bump: u8, slots_to_lock: u64, tokens_to_lock: Option<u64>) -> Self {
        let tokens_to_lock = match tokens_to_lock {
            Some(tokens_to_lock) => PodOption::some(PodU64::from(tokens_to_lock)),
            None => PodOption::none(),
        };

        Self {
            discriminator: Self::DISCRIMINATOR,
            vault_bump,
            slots_to_lock: PodU64::from(slots_to_lock),
            tokens_to_lock,
        }
    }

    /// # Safety
    /// C style cast into bytes
    pub unsafe fn to_bytes(&self) -> &[u8] {
        unsafe { crate::utils::to_bytes::<Self>(self) }
    }
}

impl DataLen for LockVaultIxData {
    const LEN: usize = core::mem::size_of::<LockVaultIxData>();
}

impl Discriminator for LockVaultIxData {
    const DISCRIMINATOR: u8 = VaultProgramInstructions::LockVault as u8;
}

/// This function will check accounts and lock up some `Some(ix_data.tokens_to_lock)` or all
/// `None(ix_data.tokens_to_lock)` tokens for `ix_data.slots_to_lock`
pub fn process_lock_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [vault, admin, mint, admin_token, vault_token, token_program, system_program] = accounts
    else {
        log!("Not enough keys, need 7, got {}", accounts.len());
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let ix_data = unsafe { load_ix_data::<LockVaultIxData>(data)? };

    // ----------------------- CHECKS -----------------------
    load_token_program(token_program)?;
    load_system_program(system_program)?;
    // We make sure the vault is owned by the system account, as in, not this program yet.
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
    // Note, if we only need one or a couple variable(s) from an account, I like to just output it
    // from a code block, this `drops` the refrence to the data. Drill and extract.
    let mint_decimals = {
        let mint = Mint::from_account_info(mint)?;
        mint.decimals()
    };

    // Load and validate the admin token account
    // Generally, I like to surround any account `load` in code blocks
    // this will drop the refrence to the data after the closing bracket
    // which means you dont have to call `drop`
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

    // Grab how many tokens are in the token account and some additional checks
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

    // If we did not specify how many tokens to lock, we lock all of them
    let tokens_to_lock = match ix_data.tokens_to_lock.as_ref() {
        Some(tokens_to_lock) => tokens_to_lock.get(),
        None => all_tokens,
    };

    // Obviously we can't transfer out more than we have. This would fail in the
    // transfer step, but I like to be explicit with my checks and output a decent
    // error message
    if tokens_to_lock > all_tokens {
        log!(
            "Tokens to lock exceed the available tokens {} > {}",
            tokens_to_lock,
            all_tokens
        );
        return Err(ProgramError::InvalidArgument);
    }

    // ----------------------- Create Vault -----------------------

    // First step is to create the vault - seeds are a little magical in rust land,
    // so I tend to copy and paste what works
    let rent = Rent::get()?;

    let bump_bytes = [ix_data.vault_bump];
    let seed_with_bump = vault_seed_with_bump!(admin.key(), mint.key(), &bump_bytes);
    let signing_seeds = [
        Seed::from(seed_with_bump[0]),
        Seed::from(seed_with_bump[1]),
        Seed::from(seed_with_bump[2]),
        Seed::from(seed_with_bump[3]),
    ];

    // Sanity check that the seeds are okay
    Vault::check_seeds(admin.key(), mint.key(), ix_data.vault_bump, &signing_seeds)?;

    let signer = Signer::from(&signing_seeds);

    CreateAccount {
        from: admin,
        to: vault,
        space: Vault::LEN as u64,
        owner: program_id,
        lamports: rent.minimum_balance(Vault::LEN),
    }
    .invoke_signed(std::slice::from_ref(&signer))?;

    unsafe {
        Vault::initialize(
            vault,
            admin.key(),
            mint.key(),
            ix_data,
            vault_token.key(),
            mint_decimals,
        )?;
    }

    // ----------------------- Transfer Tokens -----------------------
    // Now we transfer the token to the vault - note, we did not
    // create the Token Account here, so we actually need to call
    // `create_associated_token_account` in the same transaction before
    // this instruction
    pinocchio_token::instructions::Transfer {
        from: admin_token,
        to: vault_token,
        authority: admin,
        amount: tokens_to_lock,
    }
    .invoke()?;

    // ----------------------- Info -----------------------
    // Love a good completed message at the end, its more comfortable when
    // you see the transaction in the solana explorer
    log!(
        "Vault locked with {} tokens ( {} ), for {} slots",
        tokens_to_lock,
        mint.key(),
        ix_data.slots_to_lock.get()
    );

    Ok(())
}
