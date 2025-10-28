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

/// No inputs needed, if you did want to add in some,
/// make sure they are 1-byte aligned, and you use `repr(C, packed)`
/// this ensures the program will not add any extra padding where you
/// don't want it.
#[repr(C, packed)]
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
        Self::default()
    }

    /// # Safety
    /// C style cast into bytes - to do this, the struct needs to be 1-byte aligned
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

/// This will check all nessecary accounts and make sure that the vault can be emptied
/// When it does, it will transfer all of the tokens back to the creator as well as
/// close the Vault account and its rent will go back to the creator as well!
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

    // Load and validate the admin token account - Note, I like to seperate
    // these types of checks where I load an account because the refrence to
    // the `data` is dropped after the closing bracket - otherwise you have to
    // call `drop`, which looks bad to me
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

    // Same here, if we only need one variable from an account, I like to just output it
    // from a code block and `drop` the refrence to the data. Drill and extract.
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

    // Vault Checks - it makes sure the admin matches and is a signer, the mint matches
    // and the vault_token matches what is in the account
    Vault::check(
        program_id,
        vault,
        true,
        Some(admin),
        Some(mint),
        Some(vault_token),
    )?;

    // This makes sure the vault is able to be unlocked
    Vault::check_unlock_okay(vault)?;

    // ----------------------- Get Signer Seeds -----------------------
    // Seeds were always kinda confusing to me in a rust format, so I just tend to copy and past what works
    let bump: u8 = unsafe {
        let data = vault.borrow_data_unchecked();
        let vault_account = load_account::<Vault>(data)?;
        vault_account.bump()
    };
    let bump_bytes = [bump];
    let seed_with_bump = vault_seed_with_bump!(admin.key(), mint.key(), &bump_bytes);
    let signing_seeds = [
        Seed::from(seed_with_bump[0]),
        Seed::from(seed_with_bump[1]),
        Seed::from(seed_with_bump[2]),
        Seed::from(seed_with_bump[3]),
    ];
    Vault::check_seeds(admin.key(), mint.key(), bump, &signing_seeds)?;
    let signer = Signer::from(&signing_seeds);

    // ----------------------- Transfer Tokens -----------------------

    // Transfer all of the tokens back to the admin
    pinocchio_token::instructions::Transfer {
        from: vault_token,
        to: admin_token,
        authority: vault,
        amount: tokens_to_empty,
    }
    .invoke_signed(std::slice::from_ref(&signer))?;

    // ----------------------- Close Vault Token Account -----------------------

    // You have to have a 0, token balance before you can close
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
        // I would always reccomend this as there could be `rehydration` attacks
        // where if you re-initalize this account in the same transaction it could have
        // lingering data - so, boyscouts rule here.
        vault.borrow_mut_data_unchecked().fill(0);
    }

    // ----------------------- Info -----------------------
    // I like to be more verbose in my logging, I don't really care about the CU
    // in one-off transactions that will not be used often. Its also reassuring to see this
    // on the solana explorer.
    log!(
        "Vault emptied {} tokens ( {} ) to {}",
        tokens_to_empty,
        mint.key(),
        admin.key()
    );

    Ok(())
}
