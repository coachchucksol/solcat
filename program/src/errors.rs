use pinocchio::program_error::ProgramError;

/// Nothing special here, just some more error types
#[derive(Clone, PartialEq)]
pub enum DiamondHandsError {
    InvalidInstruction,
    InvalidInstructionData,
    VaultLocked,
}

impl From<DiamondHandsError> for ProgramError {
    fn from(e: DiamondHandsError) -> Self {
        Self::Custom(e as u32)
    }
}
