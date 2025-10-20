#[cfg(test)]
mod vault_tests {
    use anyhow::Result;
    use solana_keypair::Keypair;
    use solana_program::pubkey::Pubkey;
    use solana_program_test::tokio;
    use solana_signer::Signer;
    use solcat_diamond_hands_sdk::{
        accounts::vault::Vault, empty_vault_ix, id, lock_vault_ix, utils::Discriminator,
        vault_address,
    };
    use spl_associated_token_account_interface::address::get_associated_token_address;

    use crate::fixtures::fixture::TestBuilder;

    pub async fn create_token_and_mint(
        fixture: &mut TestBuilder,
        tokens_to_mint: Option<u64>,
    ) -> Result<(Pubkey, Pubkey)> {
        let mint_keypair = Keypair::new();
        let mint = mint_keypair.pubkey();
        let payer = fixture.context.payer.insecure_clone();
        let tokens_to_mint = tokens_to_mint.unwrap_or(1_000_000);

        fixture.create_mint(&mint_keypair).await?;
        fixture
            .mint_spl_to(
                &mint,
                &payer.pubkey(),
                tokens_to_mint,
                &spl_token_interface::id(),
            )
            .await?;

        let payer_ata = get_associated_token_address(&payer.pubkey(), &mint);

        Ok((mint, payer_ata))
    }

    pub async fn lock_vault(
        fixture: &mut TestBuilder,
        mint: &Pubkey,
        slots_to_lock: u64,
        tokens_to_lock: Option<u64>,
    ) -> Result<(Pubkey, Pubkey)> {
        let admin = fixture.context.payer.insecure_clone();

        let ixs = lock_vault_ix(&admin.pubkey(), mint, slots_to_lock, tokens_to_lock);
        fixture.send_transaction(&ixs, None, &[&admin]).await?;

        let (vault, _) = vault_address(&admin.pubkey(), mint);
        let vault_ata = get_associated_token_address(&vault, mint);

        Ok((vault, vault_ata))
    }

    pub async fn empty_vault(fixture: &mut TestBuilder, mint: &Pubkey) -> Result<()> {
        let admin = fixture.context.payer.insecure_clone();

        let ix = empty_vault_ix(&admin.pubkey(), mint);

        fixture.send_transaction(&[ix], None, &[&admin]).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_program_ok() -> Result<()> {
        let fixture = TestBuilder::new().await;
        let program_id: Pubkey = id();

        let account = fixture.context.banks_client.get_account(program_id).await?;

        assert!(account.is_some());
        assert!(!account.unwrap().data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_mint() -> Result<()> {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;

        let (_, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint)).await?;

        let admin_ata_account = fixture.get_token_account(&admin_ata).await?;
        assert_eq!(admin_ata_account.amount, tokens_to_mint);

        Ok(())
    }

    #[tokio::test]
    async fn test_lock_vault() -> Result<()> {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;
        let slots_to_lock = 10;

        let (mint, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint)).await?;
        let (vault, vault_ata) = lock_vault(&mut fixture, &mint, slots_to_lock, None).await?;

        let admin_ata_account = fixture.get_token_account(&admin_ata).await?;
        let vault_ata_account = fixture.get_token_account(&vault_ata).await?;
        let vault_account = fixture.get_vault_account(&vault).await?;

        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);

        assert!(vault_account.discriminator().is_some());
        assert_eq!(
            *vault_account.discriminator().unwrap(),
            Vault::DISCRIMINATOR
        );
        assert_eq!(
            *vault_account.admin(),
            fixture.context.payer.pubkey().to_bytes()
        );
        assert_eq!(*vault_account.mint(), mint.to_bytes());
        assert_eq!(vault_account.slots_locked(), slots_to_lock);
        assert_ne!(vault_account.start_slot(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_lock_and_empty_vault() -> Result<()> {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;
        let slots_to_lock = 10;

        let (mint, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint)).await?;
        let (vault, vault_ata) = lock_vault(&mut fixture, &mint, slots_to_lock, None).await?;

        let admin_ata_account = fixture.get_token_account(&admin_ata).await?;
        let vault_ata_account = fixture.get_token_account(&vault_ata).await?;
        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);

        fixture.warp_slot_incremental(10).await?;

        empty_vault(&mut fixture, &mint).await?;

        let get_vault_result = fixture.get_vault_account(&vault).await;
        assert!(get_vault_result.is_err());

        let get_vault_ata_result = fixture.get_token_account(&vault_ata).await;
        assert!(get_vault_ata_result.is_err());

        let admin_ata_account = fixture.get_token_account(&admin_ata).await?;
        assert_eq!(admin_ata_account.amount, tokens_to_mint);

        Ok(())
    }

    #[tokio::test]
    async fn test_lock_and_empty_vault_error_still_locked() -> Result<()> {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;
        let slots_to_lock = 100;

        let (mint, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint)).await?;
        let (_, vault_ata) = lock_vault(&mut fixture, &mint, slots_to_lock, None).await?;

        let admin_ata_account = fixture.get_token_account(&admin_ata).await?;
        let vault_ata_account = fixture.get_token_account(&vault_ata).await?;
        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);

        fixture.warp_slot_incremental(10).await?;

        let empty_vault_result = empty_vault(&mut fixture, &mint).await;
        assert!(empty_vault_result.is_err());

        let admin_ata_account = fixture.get_token_account(&admin_ata).await?;
        let vault_ata_account = fixture.get_token_account(&vault_ata).await?;
        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);

        Ok(())
    }
}
