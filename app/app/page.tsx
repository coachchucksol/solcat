'use client';

import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import Image from "next/image";
import dynamic from 'next/dynamic';
import { getBalance, getVault, getTokenBalance, submitTransaction, getRecentBlockhash } from './actions/solana';
import { VaultJSON, SOLCAT_MINT, lockVaultIx } from './controllers/solcat';
import { Transaction } from '@solana/web3.js';

// Dynamically import WalletMultiButton with no SSR
const WalletMultiButton = dynamic(
  async () => (await import('@solana/wallet-adapter-react-ui')).WalletMultiButton,
  { ssr: false }
);

interface VaultData extends VaultJSON {
  address: string;
}

export default function Home() {
  // ============================================================================
  // State
  // ============================================================================
  const { publicKey, signTransaction } = useWallet();
  const [balance, setBalance] = useState<number | null>(null);
  const [solcatBalance, setSolcatBalance] = useState<string | null>(null);
  const [vault, setVault] = useState<VaultData | null>(null);
  const [loadingVault, setLoadingVault] = useState(false);
  const [loadingTokenBalance, setLoadingTokenBalance] = useState(false);
  const [creatingVault, setCreatingVault] = useState(false);

  // ============================================================================
  // Effects
  // ============================================================================
  useEffect(() => {
    if (publicKey) {
      loadBalance();
      loadVault();
      loadSolcatBalance();
    } else {
      setBalance(null);
      setVault(null);
      setSolcatBalance(null);
    }
  }, [publicKey]);

  const loadBalance = async () => {
    if (!publicKey) return;

    const result = await getBalance(publicKey.toString());
    if (result.success) {
      setBalance(result.data);
    } else {
      console.error('Failed to load balance:', result.error);
      setBalance(null);
    }
  };

  const loadSolcatBalance = async () => {
    if (!publicKey) return;

    setLoadingTokenBalance(true);
    const result = await getTokenBalance(publicKey.toString(), SOLCAT_MINT.toString());

    if (result.success) {
      setSolcatBalance(result.data.balance);
    } else {
      console.error('Failed to load SOLCAT balance:', result.error);
      setSolcatBalance(null);
    }
    setLoadingTokenBalance(false);
  };

  const loadVault = async () => {
    if (!publicKey) return;

    setLoadingVault(true);
    const result = await getVault(publicKey.toString(), SOLCAT_MINT.toString());

    if (result.success) {
      setVault(result.data);
    } else {
      console.error('Failed to load vault:', result.error);
      setVault(null);
    }
    setLoadingVault(false);
  };

  const handleCreateVault = async () => {
    if (!publicKey || !signTransaction) {
      console.error('Wallet not connected');
      return;
    }

    try {
      setCreatingVault(true);

      // Create the lock vault instructions
      // Lock for 1 epoch (432000 slots) with all available tokens
      const slotsToLock = BigInt(432000);
      const tokensToLock = solcatBalance ? BigInt(solcatBalance) : null;

      const instructions = lockVaultIx(
        publicKey,
        SOLCAT_MINT,
        slotsToLock,
        tokensToLock
      );

      // Get recent blockhash from server action
      const blockhashResult = await getRecentBlockhash();
      if (!blockhashResult.success) {
        throw new Error('Failed to get blockhash');
      }

      // Create transaction
      const transaction = new Transaction();
      transaction.recentBlockhash = blockhashResult.blockhash;
      transaction.feePayer = publicKey;

      // Add all instructions
      instructions.forEach(ix => transaction.add(ix));

      // Sign the transaction
      const signedTransaction = await signTransaction(transaction);

      // Serialize and send to server
      const serializedTransaction = signedTransaction.serialize();
      const base64Transaction = Buffer.from(serializedTransaction).toString('base64');

      const result = await submitTransaction(base64Transaction);

      if (result.success) {
        console.log('Vault created successfully! Signature:', result.signature);
        // Reload vault data
        await loadVault();
        await loadSolcatBalance();
      } else {
        console.error('Failed to create vault:', result.error);
        alert(`Failed to create vault: ${result.error}`);
      }
    } catch (error) {
      console.error('Error creating vault:', error);
      alert(`Error creating vault: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setCreatingVault(false);
    }
  };

  // ============================================================================
  // Renders
  // ============================================================================

  const renderPicture = () => {
    return (
      <Image
        src="/cat.png"
        alt="Cat"
        width={16}
        height={16}
        className="w-48 h-48 sm:w-64 sm:h-64 md:w-80 md:h-80 rounded-2xl shadow-2xl"
        style={{ imageRendering: 'pixelated' }}
        priority
      />
    );
  };

  const renderConnectButton = () => {
    return (
      <WalletMultiButton className="!bg-black hover:!bg-gray-800 !transition-colors !font-medium !text-sm sm:!text-base" />
    );
  };

  const formatSolcatBalance = (balance: string, decimals: number = 6): string => {
    const balanceBigInt = BigInt(balance);
    const divisor = BigInt(10 ** decimals);
    return (balanceBigInt / divisor).toString();
  };

  const renderVaults = () => {
    if (!publicKey) return null;

    return (
      <div className="w-full max-w-2xl space-y-4">
        {/* Balance Section */}
        <div className="bg-white/10 p-4 rounded-lg backdrop-blur-sm">
          <div className="space-y-2">
            <h2 className="text-white text-xl font-bold">
              SOL Balance: {balance !== null ? `${(balance / 1e9).toFixed(4)} SOL` : 'Loading...'}
            </h2>
            <h2 className="text-white text-xl font-bold">
              SOLCAT Balance: {loadingTokenBalance ? 'Loading...' : solcatBalance !== null ? `${formatSolcatBalance(solcatBalance)} SOLCAT` : 'Loading...'}
            </h2>
          </div>
        </div>

        {/* Vault Section */}
        <div className="bg-white/10 p-4 rounded-lg backdrop-blur-sm">
          <h3 className="text-white text-lg font-bold mb-3">SOLCAT Vault</h3>

          {loadingVault ? (
            <p className="text-white/80">Loading vault...</p>
          ) : vault ? (
            <div className="space-y-2 text-sm">
              <div className="flex flex-col gap-1">
                <span className="text-white/60 text-xs">Address:</span>
                <span className="text-white font-mono text-xs break-all">{vault.address}</span>
              </div>

              <div className="grid grid-cols-2 gap-3 pt-2">
                <div>
                  <span className="text-white/60 text-xs block">Tokens Locked:</span>
                  <span className="text-white font-semibold">
                    {(BigInt(vault.tokensLocked) / BigInt(10 ** vault.mintDecimals)).toString()} SOLCAT
                  </span>
                </div>

                <div>
                  <span className="text-white/60 text-xs block">Slots Locked:</span>
                  <span className="text-white font-semibold">
                    {vault.slotsLocked} ({(BigInt(vault.slotsLocked) / BigInt(432000)).toString()} epochs)
                  </span>
                </div>
              </div>

              <div className="pt-2">
                <span className="text-white/60 text-xs block">Start Slot:</span>
                <span className="text-white">{vault.startSlot}</span>
              </div>

              <div className="pt-1">
                <span className="text-white/60 text-xs block">Vault Token Account:</span>
                <span className="text-white font-mono text-xs break-all">{vault.vaultToken}</span>
              </div>
            </div>
          ) : (
            <div className="text-center py-4">
              <p className="text-white/80 mb-4">No vault found</p>
              <p className="text-white/60 text-sm mb-4">Create a vault to lock your SOLCAT tokens</p>
              <button
                onClick={handleCreateVault}
                disabled={creatingVault || !solcatBalance || BigInt(solcatBalance) === BigInt(0)}
                className="bg-green-500 hover:bg-green-600 disabled:bg-gray-500 disabled:cursor-not-allowed text-white px-6 py-3 rounded-lg font-semibold transition-colors"
              >
                {creatingVault ? 'Creating Vault...' : 'Create Vault'}
              </button>
              {solcatBalance && BigInt(solcatBalance) === BigInt(0) && (
                <p className="text-white/60 text-xs mt-2">You need SOLCAT tokens to create a vault</p>
              )}
            </div>
          )}
        </div>
      </div>
    );
  };

  // ============================================================================
  // Main
  // ============================================================================

  return (
    <div className="flex flex-col items-center justify-center min-h-screen gap-6 sm:gap-8 p-4 sm:p-8 bg-[#38beff]">
      {renderPicture()}
      {renderConnectButton()}
      {renderVaults()}
    </div>
  );
}
