use anyhow::Result;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account_idempotent,
};

pub mod accounts {
    pub mod vault {
        pub use solcat_diamond_hands_program::accounts::vault::Vault;
    }
}

pub mod instructions {
    pub mod lock_vault {
        pub use solcat_diamond_hands_program::instructions::lock_vault::LockVaultIxData;
    }

    pub mod empty_vault {
        pub use solcat_diamond_hands_program::instructions::empty_vault::EmptyVaultIxData;
    }
}

pub mod utils {
    pub use solcat_diamond_hands_program::utils::*;
}

// ----------------------- PROGRAM ID -----------------------
pub fn id() -> Pubkey {
    solcat_diamond_hands_program::id().into()
}

// ----------------------- VAULT -----------------------
pub fn vault_address(admin: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    let seeds = [
        accounts::vault::Vault::SEED,
        &admin.to_bytes(),
        &mint.to_bytes(),
    ];
    Pubkey::find_program_address(&seeds, &id())
}

pub fn deserialize_vault(data: &[u8]) -> Result<&accounts::vault::Vault> {
    let vault_account = unsafe {
        solcat_diamond_hands_program::utils::load_account::<accounts::vault::Vault>(data)
            .map_err(|_| anyhow::anyhow!("failed to deserialize vault"))?
    };
    Ok(vault_account)
}

pub fn lock_vault_ix(
    admin: &Pubkey,
    mint: &Pubkey,
    slots_to_lock: u64,
    tokens_to_lock: Option<u64>,
) -> [Instruction; 2] {
    let program_id = id();
    let token_program = spl_token_interface::id();
    let system_program = solana_system_interface::program::id();

    let (vault, vault_bump) = vault_address(admin, mint);

    let admin_token = get_associated_token_address(admin, mint);
    let vault_token = get_associated_token_address(&vault, mint);

    // [vault, admin, mint, admin_token, vault_token, token_program, system_program]
    let accounts = vec![
        AccountMeta::new(vault, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(admin_token, false),
        AccountMeta::new(vault_token, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix_data =
        instructions::lock_vault::LockVaultIxData::new(vault_bump, slots_to_lock, tokens_to_lock);
    let ix_data_bytes = unsafe { ix_data.to_bytes() };

    let lock_vault_ix = Instruction {
        program_id,
        accounts,
        data: ix_data_bytes.to_vec(),
    };

    let vault_ata_ix =
        create_associated_token_account_idempotent(admin, &vault, mint, &token_program);

    [vault_ata_ix, lock_vault_ix]
}

pub fn empty_vault_ix(admin: &Pubkey, mint: &Pubkey) -> [Instruction; 2] {
    let program_id = id();
    let token_program = spl_token_interface::id();
    let system_program = solana_system_interface::program::id();

    let (vault, _) = vault_address(admin, mint);

    let admin_token = get_associated_token_address(admin, mint);
    let vault_token = get_associated_token_address(&vault, mint);

    // [vault, admin, mint, admin_token, vault_token, token_program, system_program]
    let accounts = vec![
        AccountMeta::new(vault, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new(admin_token, false),
        AccountMeta::new(vault_token, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix_data = instructions::empty_vault::EmptyVaultIxData::new();
    let ix_data_bytes = unsafe { ix_data.to_bytes() };

    let empty_vault_ix = Instruction {
        program_id,
        accounts,
        data: ix_data_bytes.to_vec(),
    };

    // Note, we don't strictly need this call, however, its possible for a user to "clean"
    // up old token accounts with no tokens in them, so this makes sure the admin has the
    // correct token account before we transfer the tokens back. From a UX perspective,
    // imagine you have a user that goes to unlock their vault and in the time they had their
    // tokens locked up the closed their token account - then the TX would fail and they'd
    // freak out and send me angry messages saying, I rugged them. So we put this in just in case!
    // Note, `idempotent` means it will only try to create the account if it does not exsist, so its
    // safe in both cases.
    let admin_ata_ix =
        create_associated_token_account_idempotent(admin, admin, mint, &token_program);

    [admin_ata_ix, empty_vault_ix]
}
