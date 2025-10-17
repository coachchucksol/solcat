use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_account_decoder::UiAccountEncoding;
use solana_keypair::{read_keypair_file, Keypair, Pubkey};
use solana_signer::Signer;
use solana_transaction::Transaction;
use solcat_diamond_hands_sdk::{deserialize_vault, empty_vault_ix, id, lock_vault_ix};
use std::{path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
#[command(name = "solcat")]
#[command(about = "Diamond Hands Vault CLI", long_about = None)]
struct Cli {
    /// RPC address to connect to
    #[arg(
        short,
        long,
        env = "RPC",
        default_value = "https://api.mainnet-beta.solana.com"
    )]
    rpc: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// View all vaults for a given public key
    View {
        /// Wallet to query vaults for
        #[arg(short, long, env = "WALLET")]
        wallet: String,
    },

    /// Lock tokens in a vault
    Lock {
        /// Path to the Solana keypair file
        #[arg(short, long, env = "KEYPAIR")]
        keypair: PathBuf,

        /// Token mint address
        #[arg(short, long)]
        mint: String,

        /// Amount of tokens to lock (in base units). If not provided, locks all tokens
        #[arg(short, long)]
        tokens_to_lock: Option<u64>,

        /// Number of slots to lock the vault for
        #[arg(short, long)]
        slots_to_lock: u64,
    },

    /// Empty a vault and withdraw all tokens
    Empty {
        /// Path to the Solana keypair file
        #[arg(short, long, env = "KEYPAIR")]
        keypair: PathBuf,

        /// Token mint address
        #[arg(short, long)]
        mint: String,
    },
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    let rpc_client = RpcClient::new(cli.rpc.clone());

    // Match on the subcommand
    match &cli.command {
        Commands::View { wallet } => {
            let wallet_pubkey =
                Pubkey::from_str(wallet).map_err(|e| anyhow!("Could not read wallet: {}", e))?;

            println!("\n=== Viewing vaults for wallet: {} ===", wallet_pubkey);
            println!("RPC address: {}", cli.rpc);

            view_vaults(&rpc_client, &wallet_pubkey)
        }

        Commands::Lock {
            keypair,
            mint,
            tokens_to_lock,
            slots_to_lock,
        } => {
            let keypair =
                read_keypair_file(keypair).map_err(|e| anyhow!("Could not read keypair: {}", e))?;
            let mint_pubkey =
                Pubkey::from_str(mint).map_err(|e| anyhow!("Could not read mint: {}", e))?;

            println!("\n=== Locking vault ===");
            println!("RPC address: {}", cli.rpc);
            println!("Mint: {}", mint_pubkey);
            println!(
                "Tokens to lock: {}",
                tokens_to_lock.map_or("All".to_string(), |a| a.to_string())
            );
            println!("Slots to lock: {}", slots_to_lock);

            lock_vault(
                &rpc_client,
                &keypair,
                &mint_pubkey,
                *tokens_to_lock,
                *slots_to_lock,
            )
        }

        Commands::Empty { keypair, mint } => {
            let keypair =
                read_keypair_file(keypair).map_err(|e| anyhow!("Could not read keypair: {}", e))?;
            let mint_pubkey =
                Pubkey::from_str(mint).map_err(|e| anyhow!("Could not read mint: {}", e))?;

            println!("\n=== Emptying vault ===");
            println!("RPC address: {}", cli.rpc);
            println!("Mint: {}", mint_pubkey);

            empty_vault(&rpc_client, &keypair, &mint_pubkey)
        }
    }
}

pub fn view_vaults(rpc_client: &RpcClient, wallet: &Pubkey) -> Result<()> {
    let program_id = id();
    let config: RpcProgramAccountsConfig = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            3, // Offset to Admin pubkey
            wallet.to_bytes().to_vec(),
        ))]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: None,
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };

    let results = rpc_client
        .get_program_accounts_with_config(&program_id, config)
        .map_err(|e| anyhow!("Could not fetch accounts {}", e))?;

    if results.is_empty() {
        println!("No vaults found");
        return Ok(());
    }

    for (pubkey, account) in results {
        let vault_account = deserialize_vault(&account.data)
            .map_err(|e| anyhow!("Could not deserialize account {}", e))?;

        println!("\n{}", pubkey);
        println!("{}\n", vault_account);
    }

    Ok(())
}

pub fn lock_vault(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    mint: &Pubkey,
    tokens_to_lock: Option<u64>,
    slots_to_lock: u64,
) -> Result<()> {
    let ixs = lock_vault_ix(&keypair.pubkey(), mint, slots_to_lock, tokens_to_lock);

    let blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&ixs, Some(&keypair.pubkey()), &[&keypair], blockhash);

    rpc_client.send_and_confirm_transaction_with_spinner(&tx)?;

    Ok(())
}

pub fn empty_vault(rpc_client: &RpcClient, keypair: &Keypair, mint: &Pubkey) -> Result<()> {
    let ix = empty_vault_ix(&keypair.pubkey(), mint);

    let blockhash = rpc_client.get_latest_blockhash()?;
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&keypair.pubkey()), &[&keypair], blockhash);

    rpc_client.send_and_confirm_transaction_with_spinner(&tx)?;

    Ok(())
}
