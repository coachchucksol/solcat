use pinocchio::program_error::ProgramError;

#[derive(Clone, PartialEq)]
pub enum DiamondHandsError {
    InvalidInstruction,
    InvalidInstructionData,
    ArithmeticOverflow,
    ArithmeticUnderflow,
    VaultLocked,
}

impl From<DiamondHandsError> for ProgramError {
    fn from(e: DiamondHandsError) -> Self {
        Self::Custom(e as u32)
    }
}
