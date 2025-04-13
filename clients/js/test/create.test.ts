import {describe, test, expect} from '@jest/globals';
import { Connection, Transaction, TransactionInstruction, PublicKey, Keypair, sendAndConfirmTransaction, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import sk from './program.json';
const crypto = require('crypto');

const sha256 = (data: any) => crypto.createHash('sha256').update(data).digest();

describe('Merkle tree program', () => {
  const programId = new PublicKey("TREEZwpvqQN6HVAAPjqhJAr8BuoGhXSx34jm9YV5DPB");
  const wallet = Keypair.fromSecretKey(Uint8Array.from(sk));
  const port = process.env['RPC_PORT'];
  const connection = new Connection(`http://127.0.0.1:${port}`, 'confirmed');
  const SEED = 'tree';

  test("Insert leaf", async () => {
    console.log("Program ID:", programId.toBase58());

    const [pda, bump] = await PublicKey.findProgramAddressSync(
      [Buffer.from(SEED), wallet.publicKey.toBuffer()], // seeds
      programId
    );

    console.log("Derived PDA:", pda.toBase58());
    console.log("Bump:", bump);

    //create account
    const initializeTreeInstruction = new TransactionInstruction({
      keys: [
        { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: pda, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
      ],
      programId: new PublicKey(programId),
      data: Buffer.from([0, 3])
    });

    console.log("Tree account sent to instruction ", pda.toBase58());

    const leaf = sha256(Buffer.from('LeafA'));
    const addLeafInstruction = new TransactionInstruction({
      keys: [
        { pubkey: wallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: pda, isSigner: false, isWritable: true },
      ],
      programId: new PublicKey(programId),
      data: Buffer.concat([Buffer.from([1]), leaf])
    });

    const transaction = new Transaction()
      .add(initializeTreeInstruction)
      .add(addLeafInstruction)

    const sig = await sendAndConfirmTransaction(connection, transaction, [wallet]);

    const tx = await connection.getParsedTransaction(sig);
    const logs = tx?.meta?.logMessages || [];
    const selectedEvents = logs.filter(line => ['event:CreateTree', 'event:LeafInserted'].some(event => line.includes(event)));
    expect(selectedEvents).toMatchSnapshot();
  });
});
