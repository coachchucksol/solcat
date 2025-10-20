use core::fmt;

use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{clock::Clock, Sysvar},
};
use pinocchio_log::log;

use crate::{
    accounts::VaultProgramDiscriminator,
    errors::DiamondHandsError,
    instructions::lock_vault::LockVaultIxData,
    pod::{PodOption, PodU64},
    utils::{
        load_account, load_account_mut_unchecked, load_signer, DataLen, Discriminator, Initialized,
    },
};

/// The Counter account structure
#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub struct Vault {
    /// Used to identify the account as a Vault account - Not needed in this context as there is only one type of account
    /// but it is a good standard practice
    discriminator: PodOption<u8>,
    /// u8 "Bump" that is used to "bump" the vault PDA on curve. I can be derived on-chain, but that takes up CU. So we tend
    /// to derive it off-chain ( using find_program_address ) and then save it on-chain to rederive the PDA ( using create_program_address )
    bump: u8,
    /// The owner and admin of the vault - the signer that locks and empties their vault
    admin: Pubkey,
    /// The token mint of the token that is to be locked
    mint: Pubkey,
    /// How many decimals the token has
    mint_decimals: u8,
    /// Vault token account - This could be an associated token account, we're going to keep it simple and just use
    vault_token: Pubkey,
    /// The slot at which the vault was created and the lockup starts
    start_slot: PodU64,
    /// The minimum amount of slots that vault has to be locked for
    slots_locked: PodU64,
    /// General good practice to have some reserved bytes for new features - but not necessary for the current implementation
    reserved: [u8; 32],
}

impl DataLen for Vault {
    const LEN: usize = core::mem::size_of::<Vault>();
}

/// Initialized for us is just that the account's first byte is the correct discriminator
/// when space is allocated on-chain it is always zerod out
impl Initialized for Vault {
    fn is_initialized(&self) -> bool {
        if let Some(discriminator) = self.discriminator() {
            *discriminator == Self::DISCRIMINATOR
        } else {
            false
        }
    }
}

impl Discriminator for Vault {
    const DISCRIMINATOR: u8 = VaultProgramDiscriminator::Vault as u8;
}

/// Some rust vodoo magic to format vault seeds - at its core, seeds are just arbitrary bytes
/// The Vault PDA is: ADMIN || MINT || Bump
#[macro_export]
macro_rules! vault_seed_with_bump {
    ($admin:expr, $mint:expr, $bump_slice:expr) => {
        [
            $crate::accounts::vault::Vault::SEED,
            $admin.as_ref(),
            $mint.as_ref(),
            $bump_slice,
        ]
    };
}

impl Vault {
    // ----------------------- ACCOUNT CHECKS ---------------------------

    /// A little discussion on PDA Selection. In this case, we are using the SEED || ADMIN || MINT || Bump.
    /// This configuration allows us to create one Vault per Admin and Mint pair.
    /// A benifit of this is it acts kinda like an associated token vault, so we always can know if a vault exists for a given admin and mint pair.
    /// This decision is arbitrary, you could use a different seed or even a different PDA selection strategy.
    /// For example if you wanted multiple vaults per admin and mint pair, you could add u8 "COUNT" to the seed.
    pub const SEED: &[u8] = b"VAULT";

    /// We use `create_program_address` to derive the vault PDA given a vault. So offchain we use `offchain_find_program_address`
    /// to find the PDA and bump, onchain we use `create_program_address` to derive the PDA for checking.
    pub fn create_program_address(
        program_id: &Pubkey,
        admin: &Pubkey,
        mint: &Pubkey,
        bump: u8,
    ) -> Result<Pubkey, ProgramError> {
        let bump_bytes = [bump];
        let seed_with_bump = vault_seed_with_bump!(admin, mint, &bump_bytes);
        let pda = pubkey::create_program_address(&seed_with_bump, program_id)?;

        Ok(pda)
    }

    /// This function is not strictly necessary, but I like a good sanity check.
    pub fn check_seeds(
        admin: &Pubkey,
        mint: &Pubkey,
        bump: u8,
        seeds: &[Seed],
    ) -> Result<(), ProgramError> {
        let bump_bytes = [bump];
        let seed_with_bump = vault_seed_with_bump!(admin, mint, &bump_bytes);

        if seeds.len() != seed_with_bump.len() {
            return Err(ProgramError::InvalidAccountData);
        }

        for (seed_index, seed) in seeds.iter().enumerate() {
            for (byte_index, byte) in seed.as_ref().iter().enumerate() {
                let seed_byte = seed_with_bump[seed_index][byte_index];
                if byte.ne(&seed_byte) {
                    return Err(ProgramError::InvalidAccountData);
                }
            }
        }

        Ok(())
    }

    /// This function does all of the checks needed for a vault. Checks are probably the biggest part of on-chain programming.
    /// With that being said, clear concise and non-messy code makes it way less likely that you're going to miss a check.
    /// So I always like to make a "check" function per account
    ///
    /// Note: I like to be verbose with my `log!` statements to ensure that errors are logged and debugged easily.
    /// this does come at a CU cost. You can always take them out later.
    pub fn check(
        program_id: &Pubkey,
        account_info: &AccountInfo,
        expect_writable: bool,
        check_admin: Option<&AccountInfo>,
        check_mint: Option<&AccountInfo>,
        check_token: Option<&AccountInfo>,
    ) -> Result<(), ProgramError> {
        let account_owner = account_info.owner();
        if account_owner.ne(program_id) {
            log!(
                "Vault account has an invalid program owner {} != {}",
                program_id,
                account_owner
            );
            return Err(ProgramError::InvalidAccountOwner);
        }

        if expect_writable && !account_info.is_writable() {
            log!("Vault account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // Yes this is "unsafe" at its core, we are just mapping memory to a struct.
        // In C land, this is common, not so much in Rust. Take a look at the
        // `load_account` function in the `utils` folder for more details.
        let account = unsafe {
            let data = account_info.borrow_data_unchecked();
            let result = load_account::<Vault>(data);

            if let Err(error) = result {
                log!("Vault account could not be deseralized");
                return Err(error);
            }
            result?
        };

        let account_key: Pubkey =
            Self::create_program_address(program_id, &account.admin, &account.mint, account.bump)?;
        if account_info.key().ne(&account_key) {
            log!(
                "Vault PDA does not match {} != {}",
                &account_key,
                account_info.key()
            );
            return Err(ProgramError::InvalidAccountData);
        }

        if let Some(admin) = check_admin {
            load_signer(admin, true)?;
            if account.admin().ne(admin.key()) {
                log!(
                    "Vault admin does not match {} != {}",
                    account.admin(),
                    admin.key()
                );
                return Err(ProgramError::InvalidAccountData);
            }
        }

        if let Some(mint) = check_mint {
            if account.mint().ne(mint.key()) {
                log!(
                    "Vault mint does not match {} != {}",
                    account.mint(),
                    mint.key()
                );
                return Err(ProgramError::InvalidAccountData);
            }
        }

        if let Some(token) = check_token {
            if account.vault_token().ne(token.key()) {
                log!(
                    "Vault token account does not match {} != {}",
                    account.vault_token(),
                    token.key()
                );
                return Err(ProgramError::InvalidAccountData);
            }
        }

        Ok(())
    }

    /// This is a general function to check if a vault can be unlocked.
    /// Note that we ALWAYS use checked arithmatic, in this case `saturating_sub`
    /// byte overflows and underflows are deadly and quiet.
    /// # Safety
    /// Needs to load the account, which is "unsafe"
    pub unsafe fn check_unlock_okay(account_info: &AccountInfo) -> Result<(), ProgramError> {
        let data = account_info.borrow_mut_data_unchecked();
        let account = load_account_mut_unchecked::<Vault>(data)?;
        let clock = Clock::get()?;

        let slots_elapsed = clock.slot.saturating_sub(account.start_slot());
        if slots_elapsed < account.slots_locked() {
            let remaining_slots = account.slots_locked().saturating_sub(slots_elapsed);
            log!(
                "Vault will unlock in {} slots ({} epochs)",
                remaining_slots,
                remaining_slots / 432_000u64
            );
            return Err(DiamondHandsError::VaultLocked.into());
        }

        Ok(())
    }

    // ----------------------- INITIALIZE ------------------------
    /// Just initalizes the Vault account, nothing special here
    /// # Safety
    /// Needs to load the account, which is "unsafe"
    pub unsafe fn initialize(
        account_info: &AccountInfo,
        admin: &Pubkey,
        mint: &Pubkey,
        ix_data: &LockVaultIxData,
        vault_token: &Pubkey,
        mint_decimals: u8,
    ) -> Result<(), ProgramError> {
        let data = account_info.borrow_mut_data_unchecked();
        let account = load_account_mut_unchecked::<Vault>(data)?;

        if account.is_initialized() {
            log!("Vault account is already initialized");
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let clock = Clock::get()?;

        account.discriminator = PodOption::some(VaultProgramDiscriminator::Vault as u8);
        account.bump = ix_data.vault_bump;
        account.admin = *admin;
        account.mint = *mint;
        account.vault_token = *vault_token;
        account.mint_decimals = mint_decimals;
        account.start_slot = PodU64::from(clock.slot);
        account.slots_locked = PodU64::from(ix_data.slots_to_lock);

        Ok(())
    }

    // ----------------------- GETTERS ---------------------------
    /// I prefer getters and setters where applicable over public fields.
    pub fn discriminator(&self) -> Option<&u8> {
        self.discriminator.as_ref()
    }

    pub fn bump(&self) -> u8 {
        self.bump
    }

    pub fn admin(&self) -> &Pubkey {
        &self.admin
    }

    pub fn mint(&self) -> &Pubkey {
        &self.mint
    }

    pub fn vault_token(&self) -> &Pubkey {
        &self.vault_token
    }

    pub fn start_slot(&self) -> u64 {
        self.start_slot.into()
    }

    pub fn slots_locked(&self) -> u64 {
        self.slots_locked.into()
    }
}

impl fmt::Display for Vault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let discriminator_str = match self.discriminator() {
            Some(d) => format!("{}", d),
            None => "None".to_string(),
        };

        write!(
            f,
            "Vault Account:\n\
             ├─ Discriminator: {}\n\
             ├─ Bump: {}\n\
             ├─ Admin: {:?}\n\
             ├─ Mint: {:?}\n\
             ├─ Vault Token Account: {:?}\n\
             ├─ Start Slot: {}\n\
             └─ Slots Locked: {} ({:.3} epochs)",
            discriminator_str,
            self.bump,
            self.admin,
            self.mint,
            self.vault_token,
            self.start_slot(),
            self.slots_locked(),
            self.slots_locked() as f64 / 432_000.0,
        )
    }
}
