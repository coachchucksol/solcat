'use server';

import { Connection, PublicKey } from '@solana/web3.js';
import { getAccount, getAssociatedTokenAddressSync } from '@solana/spl-token';
import { deserializeVault, vaultAddress, vaultToJSON, VaultJSON } from '../controllers/solcat';

// This runs on the server, so the RPC endpoint stays private
const getConnection = () => {
  const endpoint = process.env.RPC_ENDPOINT || 'https://api.devnet.solana.com';
  return new Connection(endpoint, 'confirmed');
};

export async function getBalance(address: string) {
  try {
    const connection = getConnection();
    const pubkey = new PublicKey(address);
    const balance = await connection.getBalance(pubkey);
    console.log(balance);

    return {
      success: true as const,
      data: balance,
    };
  } catch (error) {
    console.error('Error getting balance:', error);
    return {
      success: false as const,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

export async function getTokenBalance(owner: string, mint: string) {
  try {
    const connection = getConnection();
    const ownerPubkey = new PublicKey(owner);
    const mintPubkey = new PublicKey(mint);

    // Get the associated token account address
    const tokenAccount = getAssociatedTokenAddressSync(mintPubkey, ownerPubkey);

    try {
      // Try to get the token account
      const accountInfo = await getAccount(connection, tokenAccount);

      return {
        success: true as const,
        data: {
          balance: accountInfo.amount.toString(),
          tokenAccount: tokenAccount.toString(),
        },
      };
    } catch (accountError) {
      // Token account doesn't exist
      return {
        success: true as const,
        data: {
          balance: '0',
          tokenAccount: tokenAccount.toString(),
        },
      };
    }
  } catch (error) {
    console.error('Error getting token balance:', error);
    return {
      success: false as const,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

export async function getVault(admin: string, mint: string) {
  try {
    const connection = getConnection();
    const adminPubkey = new PublicKey(admin);
    const mintPubkey = new PublicKey(mint);

    const [vaultPubkey] = vaultAddress(adminPubkey, mintPubkey);

    const accountInfo = await connection.getAccountInfo(vaultPubkey);

    if (!accountInfo) {
      return {
        success: true as const,
        data: null,
      };
    }

    const vault = deserializeVault(Buffer.from(accountInfo.data));
    const vaultJSON = vaultToJSON(vault);

    return {
      success: true as const,
      data: {
        address: vaultPubkey.toString(),
        ...vaultJSON,
      },
    };
  } catch (error) {
    console.error('Error getting vault:', error);
    return {
      success: false as const,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

export async function getRecentBlockhash() {
  try {
    const connection = getConnection();
    const { blockhash } = await connection.getLatestBlockhash('confirmed');

    return {
      success: true as const,
      blockhash,
    };
  } catch (error) {
    console.error('Error getting blockhash:', error);
    return {
      success: false as const,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}

export async function submitTransaction(base64Transaction: string) {
  try {
    const connection = getConnection();
    const transactionBuffer = Buffer.from(base64Transaction, 'base64');

    const signature = await connection.sendRawTransaction(transactionBuffer, {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
    });

    // Wait for confirmation
    await connection.confirmTransaction(signature, 'confirmed');

    return {
      success: true as const,
      signature,
    };
  } catch (error) {
    console.error('Error submitting transaction:', error);
    return {
      success: false as const,
      error: error instanceof Error ? error.message : 'Unknown error',
    };
  }
}
