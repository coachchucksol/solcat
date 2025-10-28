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


  const loadMint = async () => {
    setLoadingMint(true);
    const result = await getMintInfo(SOLCAT_MINT.toString());
    if (result.success) {
      setMint(result.data as MintInfo);
    } else {
      console.error('Failed to load mint:', result.error);
      setMint(null);
    }

    setLoadingMint(true);
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

    // Get recent blockhash
    const blockhashResult = await getRecentBlockhash();
    if (!blockhashResult.success) {
      throw new Error('Failed to get blockhash');
    }

    // Create and populate transaction
    const transaction = new Transaction();
    transaction.recentBlockhash = blockhashResult.blockhash;
    transaction.feePayer = publicKey;

    // Add instructions (handle both array and single instruction)
    const instructionsArray = Array.isArray(instructions) ? instructions : [instructions];
    instructionsArray.forEach(ix => transaction.add(ix));

    // Sign transaction
    const signedTransaction = await signTransaction(transaction);

    // Submit transaction
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

      const slotsToLock = BigInt(10);
      const tokensToLock = solcatBalance ? BigInt(solcatBalance) : null;

      const instructions = lockVaultIx(publicKey, SOLCAT_MINT, slotsToLock, tokensToLock);

      await executeTransaction(instructions, 'Vault created');

      // Reload vault data
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

      // Reload vault data
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

  const formatAddress = (address: string) => {
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
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

  // ============================================================================
  // Renders
  // ============================================================================

  const renderHero = () => {
    return (
      <div className="text-center space-y-6 animate-fade-in">
        <div className="relative group">
          <div className="absolute inset-0 bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 rounded-3xl blur-2xl opacity-30 group-hover:opacity-50 transition-opacity duration-500"></div>
          <Image
            src="/cat.png"
            alt="SOLCAT"
            width={320}
            height={320}
            className="relative w-64 h-64 sm:w-80 sm:h-80 rounded-3xl shadow-2xl transform group-hover:scale-105 transition-transform duration-300"
            style={{ imageRendering: 'pixelated' }}
            priority
          />
        </div>
        <div className="space-y-2">
          <h1 className="text-5xl sm:text-6xl font-black text-white tracking-tight">
            SOLCAT
          </h1>
          <p className="text-xl text-white/80 font-medium">Diamond Hands Vault</p>
        </div>
      </div>
    );
  };

  const renderWalletSection = () => {
    return (
      <div className="w-full max-w-2xl">
        <div className="flex justify-center">
          <WalletMultiButton className="!bg-gradient-to-r !from-purple-600 !to-pink-600 hover:!from-purple-700 hover:!to-pink-700 !rounded-xl !px-8 !py-3 !font-bold !text-base !shadow-lg !transition-all hover:!shadow-xl hover:!scale-105" />
        </div>
      </div>
    );
  };

  const renderBalances = () => {
    if (!publicKey) return null;

    return (
      <div className="w-full max-w-2xl px-4 sm:px-0 grid grid-cols-1 sm:grid-cols-2 gap-4 animate-slide-up">
        {/* SOL Balance Card */}
        <div className="bg-gradient-to-br from-indigo-500/20 to-purple-500/20 backdrop-blur-xl rounded-2xl p-6 sm:p-7 border border-white/10 shadow-xl hover:shadow-2xl transition-all hover:scale-105">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center shadow-lg">
              <span className="text-white font-bold text-lg">◎</span>
            </div>
            <h3 className="text-white/60 text-sm font-semibold uppercase tracking-wider">SOL Balance</h3>
          </div>
          <p className="text-3xl font-black text-white">
            {balance !== null ? `${(balance / 1e9).toFixed(4)}` : '---'}
          </p>
        </div>

        {/* SOLCAT Balance Card */}
        <div className="bg-gradient-to-br from-pink-500/20 to-orange-500/20 backdrop-blur-xl rounded-2xl p-6 sm:p-7 border border-white/10 shadow-xl hover:shadow-2xl transition-all hover:scale-105">
          <div className="flex items-center gap-3 mb-2">
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-pink-500 to-orange-500 flex items-center justify-center shadow-lg">
              <span className="text-white font-bold text-lg">🐱</span>
            </div>
            <h3 className="text-white/60 text-sm font-semibold uppercase tracking-wider">SOLCAT Balance</h3>
          </div>
          <p className="text-3xl font-black text-white">
            {loadingSolcatBalance ? (
              <span className="animate-pulse">Loading...</span>
            ) : solcatBalance !== null && mint?.decimals ? (
              formatBalance(solcatBalance, mint.decimals)
            ) : (
              '---'
            )}
          </p>
        </div>
      </div>
    );
  };

  const renderVault = () => {
    if (!publicKey) return null;

    return (
      <div className="w-full max-w-2xl px-4 sm:px-0 animate-slide-up">
        <div className="bg-white/5 backdrop-blur-2xl rounded-3xl border border-white/10 shadow-2xl overflow-hidden">
          {/* Header */}
          <div className="bg-gradient-to-r from-purple-600/30 to-pink-600/30 px-6 sm:px-8 py-5 sm:py-6 border-b border-white/10">
            <h2 className="text-xl sm:text-2xl font-black text-white flex items-center gap-3">
              <span className="w-8 h-8 rounded-lg bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center shadow-lg">
                🔒
              </span>
              Diamond Hands Vault
            </h2>
          </div>

          {/* Content */}
          <div className="p-6 sm:p-8">
            {loadingVault ? (
              <div className="flex items-center justify-center py-12">
                <div className="animate-spin rounded-full h-12 w-12 border-4 border-white/20 border-t-white"></div>
              </div>
            ) : vault ? (
              <div className="space-y-6">
                {/* Vault Address */}
                <div className="bg-white/5 rounded-xl p-4 sm:p-5 border border-white/10">
                  <p className="text-white/60 text-xs font-semibold uppercase tracking-wider mb-2">Vault Address</p>
                  <div className="flex items-center gap-2">
                    <code className="text-white font-mono text-xs sm:text-sm flex-1 break-all">{vault.address}</code>
                    <button
                      onClick={() => copyToClipboard(vault.address)}
                      className="text-white/60 hover:text-white transition-colors p-2 hover:bg-white/10 rounded-lg flex-shrink-0"
                      title="Copy address"
                    >
                      📋
                    </button>
                  </div>
                </div>

                {/* Stats Grid */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div className="bg-gradient-to-br from-green-500/10 to-emerald-500/10 rounded-xl p-4 sm:p-5 border border-green-500/20">
                    <p className="text-green-400 text-xs font-semibold uppercase tracking-wider mb-2">Tokens Locked</p>
                    <p className="text-2xl font-bold text-white">
                      {(BigInt(solcatVaultBalance ?? 0) / BigInt(10 ** vault.mintDecimals)).toString()}
                    </p>
                    <p className="text-green-400/60 text-xs mt-1">SOLCAT</p>
                  </div>

                  <div className="bg-gradient-to-br from-blue-500/10 to-cyan-500/10 rounded-xl p-4 sm:p-5 border border-blue-500/20">
                    <p className="text-blue-400 text-xs font-semibold uppercase tracking-wider mb-2">Lock Duration</p>
                    <p className="text-2xl font-bold text-white">
                      {(BigInt(vault.slotsLocked)).toString()} Slots
                    </p>
                    <p className="text-blue-400/60 text-xs mt-1">epochs ({vault.slotsLocked} slots)</p>
                  </div>
                </div>

                {/* Additional Info */}
                <div className="space-y-3">
                  <div className="bg-white/5 rounded-xl p-4 sm:p-5 border border-white/10">
                    <p className="text-white/60 text-xs font-semibold uppercase tracking-wider mb-2">Estimated Lock Up</p>
                      <p className="text-white font-mono text-sm">
                        {formatDuration(estimatedTimeLeft(
                          Number(vault.startSlot),
                          Number(vault.slotsLocked),
                          epochInfo?.slot ?? 0
                        ))}
                      </p>
                  </div>

                  <div className="bg-white/5 rounded-xl p-4 sm:p-5 border border-white/10">
                    <p className="text-white/60 text-xs font-semibold uppercase tracking-wider mb-2">Current Slot</p>
                      <p className="text-white font-mono text-sm">{ epochInfo?.slot ?? 'N/A' }</p>
                  </div>

                  <div className="bg-white/5 rounded-xl p-4 sm:p-5 border border-white/10">
                    <p className="text-white/60 text-xs font-semibold uppercase tracking-wider mb-2">Start/End Slot</p>
                      <p className="text-white font-mono text-sm">{vault.startSlot}-{vault.startSlot + vault.slotsLocked}</p>
                  </div>

                  <div className="bg-white/5 rounded-xl p-4 sm:p-5 border border-white/10">
                    <p className="text-white/60 text-xs font-semibold uppercase tracking-wider mb-2">Vault Token Account</p>
                    <div className="flex items-center gap-2">
                      <code className="text-white font-mono text-xs flex-1 break-all">{vault.vaultToken}</code>
                      <button
                        onClick={() => copyToClipboard(vault.vaultToken)}
                        className="text-white/60 hover:text-white transition-colors p-2 hover:bg-white/10 rounded-lg flex-shrink-0"
                        title="Copy address"
                      >
                        📋
                      </button>
                    </div>
                  </div>
                </div>

                {/* Empty Vault Button */}
                <button
                  onClick={handleEmptyVault}
                  disabled={emptyingVault}
                  className="w-full bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-700 hover:to-rose-700 disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed text-white px-6 py-4 rounded-xl font-bold text-base sm:text-lg shadow-lg hover:shadow-xl transition-all hover:scale-105 disabled:hover:scale-100"
                >
                  {emptyingVault ? (
                    <span className="flex items-center justify-center gap-2">
                      <span className="animate-spin">⏳</span>
                      Emptying Vault...
                    </span>
                  ) : (
                    '🔓 Empty Vault'
                  )}
                </button>
              </div>
            ) : (
              <div className="text-center py-12 space-y-6">
                <div className="w-20 h-20 mx-auto rounded-full bg-gradient-to-br from-purple-500/20 to-pink-500/20 flex items-center justify-center border border-white/10">
                  <span className="text-4xl">🔒</span>
                </div>
                <div className="space-y-2 px-4">
                  <h3 className="text-xl font-bold text-white">No Vault Found</h3>
                  <p className="text-white/60 text-sm sm:text-base">Create a vault to lock your SOLCAT tokens and become a diamond hands holder</p>
                </div>
                <button
                  onClick={handleCreateVault}
                  disabled={creatingVault || !solcatBalance || BigInt(solcatBalance) === BigInt(0)}
                  className="bg-gradient-to-r from-green-600 to-emerald-600 hover:from-green-700 hover:to-emerald-700 disabled:from-gray-600 disabled:to-gray-700 disabled:cursor-not-allowed text-white px-8 py-4 rounded-xl font-bold text-base sm:text-lg shadow-lg hover:shadow-xl transition-all hover:scale-105 disabled:hover:scale-100"
                >
                  {creatingVault ? (
                    <span className="flex items-center justify-center gap-2">
                      <span className="animate-spin">⏳</span>
                      Creating Vault...
                    </span>
                  ) : (
                    '✨ Create Diamond Hands Vault'
                  )}
                </button>
                {solcatBalance && BigInt(solcatBalance) === BigInt(0) && (
                  <p className="text-white/40 text-sm px-4">You need SOLCAT tokens to create a vault</p>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

  // ============================================================================
  // Main
  // ============================================================================

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#38beff] via-[#4a9eff] to-[#5c7eff] relative overflow-hidden">
      {/* Animated Background Elements */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-10 w-72 h-72 bg-purple-500/10 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute bottom-20 right-10 w-96 h-96 bg-pink-500/10 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }}></div>
        <div className="absolute top-1/2 left-1/2 w-80 h-80 bg-orange-500/10 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '2s' }}></div>
      </div>

      {/* Content */}
      <div className="relative flex flex-col items-center justify-center min-h-screen gap-8 sm:gap-10 px-6 sm:px-8 md:px-12 lg:px-16 py-16 sm:py-20">
        {renderHero()}
        {renderWalletSection()}
        {renderBalances()}
        {renderVault()}
      </div>

      {/* Custom Animations */}
      <style jsx>{`
        @keyframes fade-in {
          from {
            opacity: 0;
            transform: translateY(-20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        @keyframes slide-up {
          from {
            opacity: 0;
            transform: translateY(30px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        .animate-fade-in {
          animation: fade-in 0.6s ease-out;
        }

        .animate-slide-up {
          animation: slide-up 0.6s ease-out;
        }
      `}</style>
    </div>
  );
}
