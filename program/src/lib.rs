#![allow(unexpected_cfgs)]

// All in all, a pretty standard entrypoint
pub mod accounts;
pub mod errors;
pub mod instructions;
pub mod pod;
pub mod utils;

pinocchio_pubkey::declare_id!("CATvuZTNuyeBkoo5Tpeqtxcn51NDLNMExWPZ5vzQxkEg");

use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    pubkey::Pubkey, ProgramResult,
};
use pinocchio_log::log;

// Add crate:: prefix to access parent modules
use crate::instructions::{
    empty_vault::process_empty_vault, lock_vault::process_lock_vault, VaultProgramInstructions,
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);

//Do not allocate memory.
no_allocator!();

// Use the no_std panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let discriminator = VaultProgramInstructions::try_from(&instruction_data[0])?;
    match discriminator {
        VaultProgramInstructions::LockVault => {
            log!("Locking Vault");
            process_lock_vault(program_id, accounts, instruction_data)
        }
        VaultProgramInstructions::EmptyVault => {
            log!("Emptying Vault");
            process_empty_vault(program_id, accounts, instruction_data)
        }
    }
}
