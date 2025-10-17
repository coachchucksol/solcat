use pinocchio::program_error::ProgramError;
use pinocchio_log::log;

pub mod vault;

#[repr(u8)]
pub enum VaultProgramDiscriminator {
    Vault = 1,
}

impl VaultProgramDiscriminator {
    pub fn from_u8(value: u8) -> Result<Self, ProgramError> {
        match value {
            1 => Ok(VaultProgramDiscriminator::Vault),
            _ => {
                log!("Invalid account discriminator: {}", value);
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}
