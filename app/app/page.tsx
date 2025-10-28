'use client';

import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import Image from "next/image";
import dynamic from 'next/dynamic';
import { getBalance, getVault, getTokenBalance, submitTransaction, getRecentBlockhash, getMintInfo, getEpochInfo } from './actions/solana';
import { VaultJSON, SOLCAT_MINT, lockVaultIx, emptyVaultIx, vaultAddress } from './controllers/solcat';
import { Transaction } from '@solana/web3.js';

// Dynamically import WalletMultiButton with no SSR
const WalletMultiButton = dynamic(
  async () => (await import('@solana/wallet-adapter-react-ui')).WalletMultiButton,
  { ssr: false }
);

// Add a client-side only wrapper
const ClientOnly = ({ children }: { children: React.ReactNode }) => {
  const [hasMounted, setHasMounted] = useState(false);

  useEffect(() => {
    setHasMounted(true);
  }, []);

  if (!hasMounted) {
    return null;
  }

  return <>{children}</>;
};

interface VaultData extends VaultJSON {
  address: string;
}

interface MintInfo {
  decimals: number;
  supply: bigint;
}

interface EpochInfo {
  slot: number;
  epoch: number;
  slotIndex: number;
}

export default function Home() {
  // ============================================================================
  // State
  // ============================================================================
  const { publicKey, signTransaction } = useWallet();
  const [balance, setBalance] = useState<number | null>(null);
  const [epochInfo, setEpochInfo] = useState<EpochInfo | null>(null);
  const [solcatBalance, setSolcatBalance] = useState<string | null>(null);
  const [solcatVaultBalance, setSolcatVaultBalance] = useState<string | null>(null);
  const [vault, setVault] = useState<VaultData | null>(null);
  const [mint, setMint] = useState<MintInfo | null>(null);
  const [loadingMint, setLoadingMint] = useState(false);
  const [loadingVault, setLoadingVault] = useState(false);
  const [loadingSolcatBalance, setLoadingSolcatBalance] = useState(false);
  const [loadingSolcatVaultBalance, setLoadingSolcatVaultBalance] = useState(false);
  const [creatingVault, setCreatingVault] = useState(false);
  const [emptyingVault, setEmptyingVault] = useState(false);
  const [slotsToLock, setSlotsToLock] = useState<string>('10');

  // ============================================================================
  // Effects
  // ============================================================================
  useEffect(() => {
    if (publicKey) {
      loadBalance();
      loadVault();
      loadSolcatBalance();
      loadSolcatVaultBalance();
      loadMint();
      loadEpochInfo();
    } else {
      setBalance(null);
      setVault(null);
      setSolcatBalance(null);
      setSolcatVaultBalance(null);
      setMint(null);
    }
  }, [publicKey]);

  // Auto-refresh epoch info when vault exists
  useEffect(() => {
    if (!publicKey || !vault) return;

    const interval = setInterval(() => {
      loadEpochInfo();
    }, 1000);

    return () => clearInterval(interval);
  }, [publicKey, vault]);

  const loadMint = async () => {
    setLoadingMint(true);
    const result = await getMintInfo(SOLCAT_MINT.toString());
    if (result.success) {
      setMint(result.data as MintInfo);
    } else {
      console.error('Failed to load mint:', result.error);
      setMint(null);
    }
    setLoadingMint(false);
  };

  const loadEpochInfo = async () => {
    if (!publicKey) return;

    const result = await getEpochInfo();
    if (result.success) {
      setEpochInfo(result.data);
    } else {
      console.error('Failed to load EpochInfo:', result.error);
      setBalance(null);
    }
  };

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

    setLoadingSolcatBalance(true);
    const result = await getTokenBalance(publicKey.toString(), SOLCAT_MINT.toString());

    if (result.success) {
      setSolcatBalance(result.data.balance);
    } else {
      console.error('Failed to load SOLCAT balance:', result.error);
      setSolcatBalance(null);
    }
    setLoadingSolcatBalance(false);
  };

  const loadSolcatVaultBalance = async () => {
    if (!publicKey) return;

    setLoadingSolcatVaultBalance(true);
    const [vault] = vaultAddress(publicKey, SOLCAT_MINT);
    const result = await getTokenBalance(vault.toString(), SOLCAT_MINT.toString(), true);

    if (result.success) {
      setSolcatVaultBalance(result.data.balance);
    } else {
      console.error('Failed to load SOLCAT vault balance:', result.error);
      setSolcatVaultBalance(null);
    }
    setLoadingSolcatVaultBalance(false);
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

  const executeTransaction = async (
    instructions: ReturnType<typeof lockVaultIx> | ReturnType<typeof emptyVaultIx>,
    actionName: string
  ) => {
    if (!publicKey || !signTransaction) {
      throw new Error('Wallet not connected');
    }

    const blockhashResult = await getRecentBlockhash();
    if (!blockhashResult.success) {
      throw new Error('Failed to get blockhash');
    }

    const transaction = new Transaction();
    transaction.recentBlockhash = blockhashResult.blockhash;
    transaction.feePayer = publicKey;

    const instructionsArray = Array.isArray(instructions) ? instructions : [instructions];
    instructionsArray.forEach(ix => transaction.add(ix));

    const signedTransaction = await signTransaction(transaction);
    const serializedTransaction = signedTransaction.serialize();
    const base64Transaction = Buffer.from(serializedTransaction).toString('base64');

    const result = await submitTransaction(base64Transaction);

    if (!result.success) {
      throw new Error(result.error || 'Transaction failed');
    }

    console.log(`${actionName} successful! Signature:`, result.signature);
    return result.signature;
  };

  const handleCreateVault = async () => {
    if (!publicKey || !signTransaction) {
      console.error('Wallet not connected');
      return;
    }

    try {
      setCreatingVault(true);

      const slotsToLockBigInt = BigInt(slotsToLock);
      const tokensToLock = solcatBalance ? BigInt(solcatBalance) : null;

      const instructions = lockVaultIx(publicKey, SOLCAT_MINT, slotsToLockBigInt, tokensToLock);

      await executeTransaction(instructions, 'Vault created');

      await loadVault();
      await loadSolcatBalance();
      await loadSolcatVaultBalance();
      await loadEpochInfo();
    } catch (error) {
      console.error('Error creating vault:', error);
      alert(`Failed to create vault: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setCreatingVault(false);
    }
  };

  const handleEmptyVault = async () => {
    if (!publicKey || !signTransaction) {
      console.error('Wallet not connected');
      return;
    }

    try {
      setEmptyingVault(true);

      const instruction = emptyVaultIx(publicKey, SOLCAT_MINT);

      await executeTransaction(instruction, 'Vault emptied');

      await loadVault();
      await loadSolcatBalance();
    } catch (error) {
      console.error('Error emptying vault:', error);
      alert(`Failed to empty vault: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setEmptyingVault(false);
    }
  };

  // ============================================================================
  // Helpers
  // ============================================================================

  const formatBalance = (balance: string, decimals: number = 9): string => {
    const balanceBigInt = BigInt(balance);
    const divisor = BigInt(10 ** decimals);
    return (balanceBigInt / divisor).toString();
  };

  const getSlotsLeft = (startSlot: number, slotsLocked: number, currentSlot: number) => {
    return Math.max(0, (startSlot + slotsLocked) - currentSlot);
  }

  const estimatedTimeLeft = (startSlot: number, slotsLocked: number, currentSlot: number) => {
    const slotsLeft = getSlotsLeft(startSlot, slotsLocked, currentSlot);
    const timePerSlotMS = 500;
    const timeLeftMS = slotsLeft * timePerSlotMS;
    const duration = {
      milliseconds: timeLeftMS,
      seconds: Math.floor(timeLeftMS / 1000),
      minutes: Math.floor(timeLeftMS / (1000 * 60)),
      hours: Math.floor(timeLeftMS / (1000 * 60 * 60)),
    };
    return duration;
  }

  const formatDuration = (duration: ReturnType<typeof estimatedTimeLeft>) => {
    if (duration.hours > 0) {
      return `${duration.hours}h ${duration.minutes % 60}m`;
    } else if (duration.minutes > 0) {
      return `${duration.minutes}m ${duration.seconds % 60}s`;
    } else {
      return `${duration.seconds}s`;
    }
  }

  const estimatedDurationFromSlots = (slots: string) => {
    try {
      const slotsNum = parseInt(slots);
      if (isNaN(slotsNum) || slotsNum <= 0) return 'Invalid';

      const timePerSlotMS = 500;
      const timeLeftMS = slotsNum * timePerSlotMS;
      const duration = {
        milliseconds: timeLeftMS,
        seconds: Math.floor(timeLeftMS / 1000),
        minutes: Math.floor(timeLeftMS / (1000 * 60)),
        hours: Math.floor(timeLeftMS / (1000 * 60 * 60)),
      };
      return formatDuration(duration);
    } catch {
      return 'Invalid';
    }
  }

  const getEstimatedUnlockDate = (slots: string): string => {
    try {
      const slotsNum = parseInt(slots);
      if (isNaN(slotsNum) || slotsNum <= 0) return 'Invalid';

      const timePerSlotMS = 500;
      const timeLeftMS = slotsNum * timePerSlotMS;

      const unlockDate = new Date(Date.now() + timeLeftMS);

      const month = String(unlockDate.getMonth() + 1).padStart(2, '0');
      const day = String(unlockDate.getDate()).padStart(2, '0');
      const year = unlockDate.getFullYear();
      const hours = String(unlockDate.getHours()).padStart(2, '0');
      const minutes = String(unlockDate.getMinutes()).padStart(2, '0');

      return `${month}.${day}.${year} ${hours}:${minutes}`;
    } catch {
      return 'Invalid';
    }
  }

  // ============================================================================
  // Main Render
  // ============================================================================

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#38beff] via-[#4a9eff] to-[#5c7eff] flex items-center justify-center p-8">
      <div className="max-w-4xl w-full space-y-8">

        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Image
              src="/cat.png"
              alt="SOLCAT"
              width={60}
              height={60}
              className="rounded-lg"
              style={{ imageRendering: 'pixelated' }}
              priority
            />
            <div>
              <h1 className="text-3xl font-bold text-white">SOLCAT</h1>
              <p className="text-white/70">Diamond Hands Vault</p>
            </div>
          </div>
          <ClientOnly>
            <WalletMultiButton className="!bg-purple-600 hover:!bg-purple-700 !rounded-lg" />
          </ClientOnly>
        </div>

        {publicKey ? (
          <>
            {/* Balances */}
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-white/10 backdrop-blur-sm rounded-lg p-8 border border-white/20">
                <div className="text-white/70 text-sm mb-2">SOL Balance</div>
                <div className="text-2xl font-bold text-white">
                  {balance !== null ? (balance / 1e9).toFixed(4) : '---'}
                </div>
              </div>
              <div className="bg-white/10 backdrop-blur-sm rounded-lg p-8 border border-white/20">
                <div className="text-white/70 text-sm mb-2">SOLCAT Balance</div>
                <div className="text-2xl font-bold text-white">
                  {loadingSolcatBalance ? (
                    'Loading...'
                  ) : solcatBalance !== null && mint?.decimals ? (
                    formatBalance(solcatBalance, mint.decimals)
                  ) : (
                    '---'
                  )}
                </div>
              </div>
            </div>

            {/* Vault */}
            <div className="bg-white/10 backdrop-blur-sm rounded-lg border border-white/20">
              <div className="p-8 border-b border-white/20">
                <h2 className="text-xl font-bold text-white">Vault</h2>
              </div>

              <div className="p-8">
                {loadingVault ? (
                  <div className="text-center py-12">
                    <div className="animate-spin rounded-full h-10 w-10 border-2 border-white/30 border-t-white mx-auto"></div>
                  </div>
                ) : vault ? (
                  <div className="space-y-4">
                    {/* Vault Stats */}
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <div className="text-white/70 text-sm mb-2">Tokens Locked</div>
                        <div className="text-xl font-bold text-white">
                          {(BigInt(solcatVaultBalance ?? 0) / BigInt(10 ** vault.mintDecimals)).toString()} SOLCAT
                        </div>
                      </div>
                      <div>
                        <div className="text-white/70 text-sm mb-2">Time Remaining</div>
                        <div className="text-xl font-bold text-white">
                          {formatDuration(estimatedTimeLeft(
                            Number(vault.startSlot),
                            Number(vault.slotsLocked),
                            epochInfo?.slot ?? 0
                          ))}
                        </div>
                      </div>
                    </div>

                    {/* Vault Details */}
                    <div className="space-y-4 pt-4">
                      <div>
                        <div className="text-white/70 text-xs mb-2">Vault Address</div>
                        <div className="flex items-center gap-2">
                          <code className="text-white text-sm font-mono flex-1 truncate">{vault.address}</code>
                        </div>
                      </div>

                      <div className="grid grid-cols-3 gap-4">
                        <div>
                          <div className="text-white/70 text-xs mb-2">Slots Locked</div>
                          <div className="text-white text-sm">{vault.slotsLocked}</div>
                        </div>
                        <div>
                          <div className="text-white/70 text-xs mb-2">Current Slot</div>
                          <div className="text-white text-sm">{epochInfo?.slot ?? 'N/A'}</div>
                        </div>
                        <div>
                          <div className="text-white/70 text-xs mb-2">End Slot</div>
                          <div className="text-white text-sm">{Number(vault.startSlot) + Number(vault.slotsLocked)}</div>
                        </div>
                      </div>
                    </div>

                    {/* Empty Vault Button */}
                    <button
                      onClick={handleEmptyVault}
                      disabled={emptyingVault || !epochInfo || epochInfo.slot <= Number(vault.startSlot) + Number(vault.slotsLocked)}
                      className="w-full mt-4 bg-red-600 hover:bg-red-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white px-6 py-3 rounded-lg font-semibold transition-colors"
                    >
                      {emptyingVault ? 'Emptying Vault...' :
                       (!epochInfo || epochInfo.slot <= Number(vault.startSlot) + Number(vault.slotsLocked)) ?
                       'Vault Locked' : 'Empty Vault'}
                    </button>
                  </div>
                ) : (
                  <div className="space-y-4">
                    <div className="text-center py-6">
                      <div className="text-white/70 mb-2">No vault found</div>
                      <div className="text-white text-sm">Create a vault to lock your SOLCAT tokens</div>
                    </div>

                    {/* Create Vault Form */}
                    <div>
                      <label className="text-white/70 text-sm mb-2 block">Slots to Lock</label>
                      <input
                        type="number"
                        min="1"
                        value={slotsToLock}
                        onChange={(e) => setSlotsToLock(e.target.value)}
                        className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
                        placeholder="Enter number of slots"
                      />
                      <div className="text-white text-sm mt-2 bg-white/10 rounded px-3 py-2 border border-white/20">
                        ≈ {estimatedDurationFromSlots(slotsToLock)} (Est Unlock: {getEstimatedUnlockDate(slotsToLock)})
                      </div>
                    </div>

                    <button
                      onClick={handleCreateVault}
                      disabled={creatingVault || !solcatBalance || BigInt(solcatBalance) === BigInt(0) || !slotsToLock || parseInt(slotsToLock) <= 0}
                      className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white px-6 py-3 rounded-lg font-semibold transition-colors"
                    >
                      {creatingVault ? 'Creating Vault...' : 'Create Vault'}
                    </button>

                    {solcatBalance && BigInt(solcatBalance) === BigInt(0) && (
                      <div className="text-white/50 text-sm text-center">
                        You need SOLCAT tokens to create a vault
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          </>
        ) : (
          <div className="bg-white/10 backdrop-blur-sm rounded-lg p-12 border border-white/20 text-center">
            <div className="text-white/70 mb-4">Connect your wallet to get started</div>
            <ClientOnly>
              <WalletMultiButton className="!bg-purple-600 hover:!bg-purple-700 !rounded-lg mx-auto" />
            </ClientOnly>
          </div>
        )}
      </div>
    </div>
  );
}
