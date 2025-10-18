import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountIdempotentInstruction,
} from '@solana/spl-token';
import { Buffer } from 'buffer';

// ----------------------- PROGRAM ID -----------------------
export const PROGRAM_ID = new PublicKey('CATvuZTNuyeBkoo5Tpeqtxcn51NDLNMExWPZ5vzQxkEg');
// export const SOLCAT_MINT = new PublicKey('84Y6h6XoaLAD1zxoQ2CDhcZYRpNsSBKsXULCnpjXpump');
export const SOLCAT_MINT = new PublicKey('Fvg3HU47cXfmnys3tKSn6ZFnMgL7Rti61ZdcJcPX89h5');

export function id(): PublicKey {
  return PROGRAM_ID;
}

// ----------------------- CONSTANTS -----------------------
export const VAULT_SEED = Buffer.from('VAULT');
export const VAULT_DISCRIMINATOR = 1;

export const LOCK_VAULT_IX_DISCRIMINATOR = 1;
export const EMPTY_VAULT_IX_DISCRIMINATOR = 2;

// ----------------------- VAULT -----------------------
export function vaultAddress(admin: PublicKey, mint: PublicKey): [PublicKey, number] {
  const [address, bump] = PublicKey.findProgramAddressSync(
    [VAULT_SEED, admin.toBuffer(), mint.toBuffer()],
    PROGRAM_ID
  );
  return [address, bump];
}

// ----------------------- VAULT ACCOUNT STRUCTURE -----------------------
export interface Vault {
  discriminator: number;
  bump: number;
  admin: PublicKey;
  mint: PublicKey;
  mintDecimals: number;
  vaultToken: PublicKey;
  startSlot: bigint;
  slotsLocked: bigint;
  tokensLocked: bigint;
  reserved: Uint8Array;
}

export interface VaultJSON {
  discriminator: number;
  bump: number;
  admin: string;
  mint: string;
  mintDecimals: number;
  vaultToken: string;
  startSlot: string;
  slotsLocked: string;
  tokensLocked: string;
  reserved: number[];
}

export function vaultToJSON(vault: Vault): VaultJSON {
  return {
    discriminator: vault.discriminator,
    bump: vault.bump,
    admin: vault.admin.toString(),
    mint: vault.mint.toString(),
    mintDecimals: vault.mintDecimals,
    vaultToken: vault.vaultToken.toString(),
    startSlot: vault.startSlot.toString(),
    slotsLocked: vault.slotsLocked.toString(),
    tokensLocked: vault.tokensLocked.toString(),
    reserved: Array.from(vault.reserved),
  };
}

export function vaultFromJSON(json: VaultJSON): Vault {
  return {
    discriminator: json.discriminator,
    bump: json.bump,
    admin: new PublicKey(json.admin),
    mint: new PublicKey(json.mint),
    mintDecimals: json.mintDecimals,
    vaultToken: new PublicKey(json.vaultToken),
    startSlot: BigInt(json.startSlot),
    slotsLocked: BigInt(json.slotsLocked),
    tokensLocked: BigInt(json.tokensLocked),
    reserved: new Uint8Array(json.reserved),
  };
}

export function deserializeVault(data: Buffer): Vault {
  let offset = 0;

  // Read discriminator (PodOption<u8>)
  const _hasDiscriminator = data.readUInt8(offset);
  offset += 1;
  const discriminator = data.readUInt8(offset);
  offset += 1;

  // Read bump
  const bump = data.readUInt8(offset);
  offset += 1;

  // Read admin (32 bytes)
  const admin = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;

  // Read mint (32 bytes)
  const mint = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;

  // Read mint_decimals
  const mintDecimals = data.readUInt8(offset);
  offset += 1;

  // Read vault_token (32 bytes)
  const vaultToken = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;

  // Read start_slot (PodU64 - 8 bytes)
  const startSlot = data.readBigUInt64LE(offset);
  offset += 8;

  // Read slots_locked (PodU64 - 8 bytes)
  const slotsLocked = data.readBigUInt64LE(offset);
  offset += 8;

  // Read tokens_locked (PodU64 - 8 bytes)
  const tokensLocked = data.readBigUInt64LE(offset);
  offset += 8;

  // Read reserved (32 bytes)
  const reserved = data.slice(offset, offset + 32);

  return {
    discriminator,
    bump,
    admin,
    mint,
    mintDecimals,
    vaultToken,
    startSlot,
    slotsLocked,
    tokensLocked,
    reserved,
  };
}

// ----------------------- INSTRUCTION DATA -----------------------
interface LockVaultIxData {
  vaultBump: number;
  slotsToLock: bigint;
  tokensToLock: bigint | null;
}

function serializeLockVaultIxData(data: LockVaultIxData): Buffer {
  // Rust's Option<u64> with #[repr(C)] has specific layout:
  // - discriminant (u8): 1 byte
  // - padding: 7 bytes (to align u64)
  // - value (u64): 8 bytes
  // Total Option<u64> = 16 bytes

  const buffer = Buffer.alloc(1 + 1 + 8 + 16); // discriminator + bump + slots + Option<u64>
  let offset = 0;

  // Write discriminator
  buffer.writeUInt8(LOCK_VAULT_IX_DISCRIMINATOR, offset);
  offset += 1;

  // Write vault_bump
  buffer.writeUInt8(data.vaultBump, offset);
  offset += 1;

  // Write slots_to_lock
  buffer.writeBigUInt64LE(data.slotsToLock, offset);
  offset += 8;

  // Write tokens_to_lock (Option<u64>)
  if (data.tokensToLock !== null) {
    buffer.writeUInt8(1, offset); // Some discriminant
    offset += 1;
    // 7 bytes padding
    offset += 7;
    buffer.writeBigUInt64LE(data.tokensToLock, offset); // value
  } else {
    buffer.writeUInt8(0, offset); // None discriminant
    // Rest is already zeroed
  }

  return buffer;
}

interface EmptyVaultIxData {}

function serializeEmptyVaultIxData(): Buffer {
  const buffer = Buffer.alloc(1);
  let offset = 0;

  // Write discriminator
  buffer.writeUInt8(EMPTY_VAULT_IX_DISCRIMINATOR, offset);

  return buffer;
}

// ----------------------- INSTRUCTIONS -----------------------
export function lockVaultIx(
  admin: PublicKey,
  mint: PublicKey,
  slotsToLock: bigint,
  tokensToLock: bigint | null = null
): TransactionInstruction[] {
  const [vault, vaultBump] = vaultAddress(admin, mint);

  const adminToken = getAssociatedTokenAddressSync(mint, admin);
  const vaultToken = getAssociatedTokenAddressSync(mint, vault, true);

  // Create vault ATA instruction
  const vaultAtaIx = createAssociatedTokenAccountIdempotentInstruction(
    admin,
    vaultToken,
    vault,
    mint,
    TOKEN_PROGRAM_ID
  );

  // Create lock vault instruction
  const keys = [
    { pubkey: vault, isSigner: false, isWritable: true },
    { pubkey: admin, isSigner: true, isWritable: true },
    { pubkey: mint, isSigner: false, isWritable: false },
    { pubkey: adminToken, isSigner: false, isWritable: true },
    { pubkey: vaultToken, isSigner: false, isWritable: true },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ];

  const ixData: LockVaultIxData = {
    vaultBump,
    slotsToLock,
    tokensToLock,
  };

  const data = serializeLockVaultIxData(ixData);

  const lockVaultIx = new TransactionInstruction({
    keys,
    programId: PROGRAM_ID,
    data,
  });

  return [vaultAtaIx, lockVaultIx];
}

export function emptyVaultIx(admin: PublicKey, mint: PublicKey): TransactionInstruction {
  const [vault] = vaultAddress(admin, mint);

  const adminToken = getAssociatedTokenAddressSync(mint, admin);
  const vaultToken = getAssociatedTokenAddressSync(mint, vault, true);

  const keys = [
    { pubkey: vault, isSigner: false, isWritable: true },
    { pubkey: admin, isSigner: true, isWritable: true },
    { pubkey: mint, isSigner: false, isWritable: false },
    { pubkey: adminToken, isSigner: false, isWritable: true },
    { pubkey: vaultToken, isSigner: false, isWritable: true },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ];

  const data = serializeEmptyVaultIxData();

  return new TransactionInstruction({
    keys,
    programId: PROGRAM_ID,
    data,
  });
}
