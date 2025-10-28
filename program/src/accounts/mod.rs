use pinocchio::program_error::ProgramError;
use pinocchio_log::log;

pub mod vault;

/// Only 1 account type here, I really like to use hex for these scanrios
#[repr(u8)]
pub enum VaultProgramDiscriminator {
    Vault = 0x01,
}

impl VaultProgramDiscriminator {
    pub fn from_u8(value: u8) -> Result<Self, ProgramError> {
        match value {
            0x01 => Ok(VaultProgramDiscriminator::Vault),
            _ => {
                log!("Invalid account discriminator: {}", value);
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}

// I like to have compile-time errors. A discriminator should never be 0, as
// all account data is 0 when it is created. So it could look initalized without being
// initialized - this is also why I use PodOption<u8> for discriminators, so I know when
// something has been intentionally set
const _: () = assert!(VaultProgramDiscriminator::Vault as u8 != 0);
