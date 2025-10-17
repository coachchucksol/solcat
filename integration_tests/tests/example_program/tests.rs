#[cfg(test)]
mod tests {
    use anyhow::Result;
    use solana_commitment_config::CommitmentLevel;
    use solana_keypair::Keypair;
    use solana_program::pubkey::Pubkey;
    use solana_program_test::tokio;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
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
        fixture: &TestBuilder,
        mint: &Pubkey,
        slots_to_lock: u64,
        tokens_to_lock: Option<u64>,
    ) -> Result<(Pubkey, Pubkey)> {
        let admin = fixture.context.payer.insecure_clone();

        let ixs = lock_vault_ix(&admin.pubkey(), mint, slots_to_lock, tokens_to_lock);

        let blockhash = fixture.context.banks_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&fixture.context.payer.pubkey()),
            &[&fixture.context.payer],
            blockhash,
        );

        fixture
            .context
            .banks_client
            .process_transaction_with_preflight_and_commitment(tx, CommitmentLevel::Processed)
            .await?;

        let (vault, _) = vault_address(&admin.pubkey(), mint);
        let vault_ata = get_associated_token_address(&vault, &mint);

        Ok((vault, vault_ata))
    }

    pub async fn empty_vault(fixture: &TestBuilder, mint: &Pubkey) -> Result<()> {
        let admin = fixture.context.payer.insecure_clone();

        let ix = empty_vault_ix(&admin.pubkey(), mint);

        let blockhash = fixture.context.banks_client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&fixture.context.payer.pubkey()),
            &[&fixture.context.payer],
            blockhash,
        );

        fixture
            .context
            .banks_client
            .process_transaction_with_preflight_and_commitment(tx, CommitmentLevel::Processed)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_program_ok() {
        let fixture = TestBuilder::new().await;
        let program_id: Pubkey = id();

        let account = fixture
            .context
            .banks_client
            .get_account(program_id)
            .await
            .expect("Could not get program");

        assert!(account.is_some());
        assert!(account.unwrap().data.len() > 0);
    }

    #[tokio::test]
    async fn test_create_mint() {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;

        let (_, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint))
            .await
            .expect("Could not create mint");

        let admin_ata_account = fixture
            .get_token_account(&admin_ata)
            .await
            .expect("Could not get token account");
        assert_eq!(admin_ata_account.amount, tokens_to_mint);
    }

    #[tokio::test]
    async fn test_lock_vault() {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;
        let slots_to_lock = 10;

        let (mint, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint))
            .await
            .expect("Could not create mint");
        let (vault, vault_ata) = lock_vault(&mut fixture, &mint, slots_to_lock, None)
            .await
            .expect("Could not create vault");

        let admin_ata_account = fixture
            .get_token_account(&admin_ata)
            .await
            .expect("Could not get token account");
        let vault_ata_account = fixture
            .get_token_account(&vault_ata)
            .await
            .expect("Could not get token account");
        let vault_account = fixture
            .get_vault_account(&vault)
            .await
            .expect("Could not grab vault");

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
        assert_eq!(vault_account.tokens_locked(), tokens_to_mint);
        assert_ne!(vault_account.start_slot(), 0);
    }

    #[tokio::test]
    async fn test_lock_and_empty_vault() {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;
        let slots_to_lock = 10;

        let (mint, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint))
            .await
            .expect("Could not create mint");
        let (vault, vault_ata) = lock_vault(&mut fixture, &mint, slots_to_lock, None)
            .await
            .expect("Could not create vault");

        let admin_ata_account = fixture
            .get_token_account(&admin_ata)
            .await
            .expect("Could not get token account");
        let vault_ata_account = fixture
            .get_token_account(&vault_ata)
            .await
            .expect("Could not get token account");
        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);

        fixture
            .warp_slot_incremental(10)
            .await
            .expect("Failed to warp slot");

        empty_vault(&fixture, &mint)
            .await
            .expect("Failed to empty vault");

        let get_vault_result = fixture.get_vault_account(&vault).await;
        assert!(get_vault_result.is_err());

        let get_vault_ata_result = fixture.get_token_account(&vault_ata).await;
        assert!(get_vault_ata_result.is_err());

        let admin_ata_account = fixture
            .get_token_account(&admin_ata)
            .await
            .expect("Could not get token account");
        assert_eq!(admin_ata_account.amount, tokens_to_mint);
    }

    #[tokio::test]
    async fn test_lock_and_empty_vault_error_still_locked() {
        let mut fixture = TestBuilder::new().await;
        let tokens_to_mint = 1000;
        let slots_to_lock = 100;

        let (mint, admin_ata) = create_token_and_mint(&mut fixture, Some(tokens_to_mint))
            .await
            .expect("Could not create mint");
        let (_, vault_ata) = lock_vault(&mut fixture, &mint, slots_to_lock, None)
            .await
            .expect("Could not create vault");

        let admin_ata_account = fixture
            .get_token_account(&admin_ata)
            .await
            .expect("Could not get token account");
        let vault_ata_account = fixture
            .get_token_account(&vault_ata)
            .await
            .expect("Could not get token account");
        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);

        fixture
            .warp_slot_incremental(10)
            .await
            .expect("Failed to warp slot");

        let empty_vault_result = empty_vault(&fixture, &mint).await;
        assert!(empty_vault_result.is_err());

        let admin_ata_account = fixture
            .get_token_account(&admin_ata)
            .await
            .expect("Could not get token account");
        let vault_ata_account = fixture
            .get_token_account(&vault_ata)
            .await
            .expect("Could not get token account");
        assert_eq!(admin_ata_account.amount, 0);
        assert_eq!(vault_ata_account.amount, tokens_to_mint);
    }
}
