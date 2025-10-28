use pinocchio::{account_info::AccountInfo, program_error::ProgramError};
use pinocchio_log::log;

use crate::errors::DiamondHandsError;

// All on-chain accounts implement these: `Discriminator`, `DataLen`, and `Initialized`
// while not strictly nesscary, it helps out with consistancy. In my current on-chain
// products, I combine these traits into one.
pub trait Discriminator {
    const DISCRIMINATOR: u8;
}

pub trait DataLen {
    const LEN: usize;
}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
/// Note, this is why we have to use 1-byte aligned structs, it allows us to
/// safely map a struct onto a section of memory - it is `safe` as long as the struct
/// is 1-byte alinged.
#[inline(always)]
pub unsafe fn load_account<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_account_unchecked::<T>(bytes).and_then(|account| {
        if account.is_initialized() {
            Ok(account)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
#[inline(always)]
pub unsafe fn load_account_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
#[inline(always)]
pub unsafe fn load_account_mut<T: DataLen + Initialized>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    load_account_mut_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
#[inline(always)]
pub unsafe fn load_account_mut_unchecked<T: DataLen>(
    bytes: &mut [u8],
) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
/// We follow the same standards for the IX data
#[inline(always)]
pub unsafe fn load_ix_data<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(DiamondHandsError::InvalidInstructionData.into());
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
#[inline(always)]
pub unsafe fn to_bytes<T: DataLen>(data: &T) -> &[u8] {
    core::slice::from_raw_parts(data as *const T as *const u8, T::LEN)
}

/// # Safety
/// Caller must ensure everything is 1 byte aligned
#[inline(always)]
pub unsafe fn to_mut_bytes<T: DataLen>(data: &mut T) -> &mut [u8] {
    core::slice::from_raw_parts_mut(data as *mut T as *mut u8, T::LEN)
}

// --------------------- OTHER HELPERS ---------------------
// The following helpers just make it look a little nicer to check certain aspects of accounts
// or make sure the right program is asserted

pub fn load_signer(info: &AccountInfo, expect_writable: bool) -> Result<(), ProgramError> {
    if !info.is_signer() {
        log!("Account is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if expect_writable && !info.is_writable() {
        log!("Signer is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

pub fn load_token_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key().ne(&pinocchio_token::id()) {
        log!("Account is not the token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn load_system_program(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.key().ne(&pinocchio_system::id()) {
        log!("Account is not the system program");
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

pub fn load_system_account(info: &AccountInfo, is_writable: bool) -> Result<(), ProgramError> {
    let owner = info.owner();
    if owner.ne(&pinocchio_system::id()) {
        log!("Account is not owned by the system program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.data_is_empty() {
        log!("Account data is not empty");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if is_writable && !info.is_writable() {
        log!("Account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
