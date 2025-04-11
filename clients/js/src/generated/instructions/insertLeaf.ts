/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Context,
  Pda,
  PublicKey,
  Signer,
  TransactionBuilder,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  bytes,
  mapSerializer,
  struct,
  u8,
} from '@metaplex-foundation/umi/serializers';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  getAccountMetasAndSigners,
} from '../shared';

// Accounts.
export type InsertLeafInstructionAccounts = {
  /** The account paying for the storage fees */
  payer?: Signer;
  /** The address of the new account */
  tree: PublicKey | Pda;
};

// Data.
export type InsertLeafInstructionData = {
  discriminator: number;
  leaf: Uint8Array;
};

export type InsertLeafInstructionDataArgs = { leaf: Uint8Array };

export function getInsertLeafInstructionDataSerializer(): Serializer<
  InsertLeafInstructionDataArgs,
  InsertLeafInstructionData
> {
  return mapSerializer<
    InsertLeafInstructionDataArgs,
    any,
    InsertLeafInstructionData
  >(
    struct<InsertLeafInstructionData>(
      [
        ['discriminator', u8()],
        ['leaf', bytes({ size: 32 })],
      ],
      { description: 'InsertLeafInstructionData' }
    ),
    (value) => ({ ...value, discriminator: 1 })
  ) as Serializer<InsertLeafInstructionDataArgs, InsertLeafInstructionData>;
}

// Args.
export type InsertLeafInstructionArgs = InsertLeafInstructionDataArgs;

// Instruction.
export function insertLeaf(
  context: Pick<Context, 'payer' | 'programs'>,
  input: InsertLeafInstructionAccounts & InsertLeafInstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'merkleTreeStorage',
    'TREEZwpvqQN6HVAAPjqhJAr8BuoGhXSx34jm9YV5DPB'
  );

  // Accounts.
  const resolvedAccounts = {
    payer: {
      index: 0,
      isWritable: true as boolean,
      value: input.payer ?? null,
    },
    tree: { index: 1, isWritable: true as boolean, value: input.tree ?? null },
  } satisfies ResolvedAccountsWithIndices;

  // Arguments.
  const resolvedArgs: InsertLeafInstructionArgs = { ...input };

  // Default values.
  if (!resolvedAccounts.payer.value) {
    resolvedAccounts.payer.value = context.payer;
  }

  // Accounts in order.
  const orderedAccounts: ResolvedAccount[] = Object.values(
    resolvedAccounts
  ).sort((a, b) => a.index - b.index);

  // Keys and Signers.
  const [keys, signers] = getAccountMetasAndSigners(
    orderedAccounts,
    'programId',
    programId
  );

  // Data.
  const data = getInsertLeafInstructionDataSerializer().serialize(
    resolvedArgs as InsertLeafInstructionDataArgs
  );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
