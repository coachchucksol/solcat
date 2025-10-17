pub mod empty_vault;
pub mod lock_vault;

use crate::errors::DiamondHandsError;

#[repr(u8)]
pub enum VaultProgramInstructions {
    LockVault = 1,
    EmptyVault = 2,
}

impl TryFrom<&u8> for VaultProgramInstructions {
    type Error = DiamondHandsError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            1 => Ok(VaultProgramInstructions::LockVault),
            2 => Ok(VaultProgramInstructions::EmptyVault),
            _ => Err(DiamondHandsError::InvalidInstruction),
        }
    }
}
