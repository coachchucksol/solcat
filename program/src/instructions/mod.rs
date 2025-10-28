pub mod empty_vault;
pub mod lock_vault;

use crate::errors::DiamondHandsError;

#[repr(u8)]
pub enum VaultProgramInstructions {
    LockVault = 0x01,
    EmptyVault = 0x02,
}

impl TryFrom<&u8> for VaultProgramInstructions {
    type Error = DiamondHandsError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0x01 => Ok(VaultProgramInstructions::LockVault),
            0x02 => Ok(VaultProgramInstructions::EmptyVault),
            _ => Err(DiamondHandsError::InvalidInstruction),
        }
    }
}

// Discriminators should never be 0, so I have compile time asserts
// to make sure they are never 0
const _: () = assert!(VaultProgramInstructions::LockVault as u8 != 0);
const _: () = assert!(VaultProgramInstructions::EmptyVault as u8 != 0);
